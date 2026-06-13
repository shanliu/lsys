// 文件下载进度跟踪器
// 设计：
//   - record_bytes()         由下载循环在 1s 节拍里调用，内部计算速度并写入 Redis HASH + PUBLISH
//   - get_progress_batch()   批量单次查询多个文件的进度（从 Redis HASH 读取，无数据返回 None）
//   - subscribe_progress()   返回 mpsc::Receiver，第一条消息为当前快照，后续消息来自 Redis PUB/SUB
//   - clear_progress()       下载完成或失败时清理 Redis key
//
// Redis key / channel：file:progress:{file_id}
// Redis HASH fields：c{chunk_id}:dl, c{chunk_id}:sz, c{chunk_id}:spd

use crate::common::FileError;
use deadpool_redis::redis::{AsyncCommands, pipe};
use futures_util::StreamExt;
use lsys_core::app_core::AppCore;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::{OnceCell, Semaphore, mpsc};
use tracing::{info, warn};

const PROGRESS_KEY_PREFIX: &str = "file:progress:";

// ──────────────────────────────────────────
// 公开数据结构
// ──────────────────────────────────────────

/// 下载状态
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    #[default]
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChunkProgressInfo {
    pub chunk_id: u64,
    pub downloaded: u64,
    pub total_size: u64,
    pub speed_bps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileProgressInfo {
    pub file_id: u64,
    pub total_downloaded: u64,
    pub total_size: u64,
    /// 0.0 ~ 100.0
    pub percent: f32,
    /// 所有分片速度之和 (bytes/sec)
    pub speed_bps: u64,
    pub chunks: Vec<ChunkProgressInfo>,
    /// 下载状态
    pub status: DownloadStatus,
}

// ──────────────────────────────────────────
// 内部辅助
// ──────────────────────────────────────────

struct SpeedSample {
    last_bytes: u64,
    last_instant: Instant,
}

/// write_worker 的消息类型
enum WorkerMsg {
    /// 进度更新（record_bytes 产生）
    Update {
        file_id: u64,
        chunk_id: u64,
        current_downloaded: u64,
        chunk_total_size: u64,
        file_total_size: u64,
        speed_bps: u64,
    },
    /// 终态清理（clear_progress 产生）：PUBLISH 终态信号 + DEL key
    Clear {
        file_id: u64,
        status: DownloadStatus,
    },
}

/// 从 Redis HGETALL 返回的 HashMap 解析出 FileProgressInfo
fn parse_progress_from_map(file_id: u64, map: HashMap<String, String>) -> FileProgressInfo {
    let mut chunk_map: HashMap<u64, ChunkProgressInfo> = HashMap::new();
    // file:sz — 文件总大小（由 record_bytes 写入，优先于分片大小求和）
    let mut file_total_size: u64 = 0;

    for (key, val) in &map {
        // 文件级总大小
        if key == "file:sz" {
            file_total_size = val.parse().unwrap_or(0);
            continue;
        }
        // key 格式: c{chunk_id}:dl | c{chunk_id}:sz | c{chunk_id}:spd
        let Some(rest) = key.strip_prefix('c') else {
            continue;
        };
        let parts: Vec<&str> = rest.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }
        let chunk_id: u64 = parts[0].parse().unwrap_or(0);
        let entry = chunk_map.entry(chunk_id).or_insert(ChunkProgressInfo {
            chunk_id,
            downloaded: 0,
            total_size: 0,
            speed_bps: 0,
        });
        match parts[1] {
            "dl" => entry.downloaded = val.parse().unwrap_or(0),
            "sz" => entry.total_size = val.parse().unwrap_or(0),
            "spd" => entry.speed_bps = val.parse().unwrap_or(0),
            _ => {}
        }
    }

    let mut chunks: Vec<ChunkProgressInfo> = chunk_map.into_values().collect();
    chunks.sort_by_key(|c| c.chunk_id);

    let total_downloaded: u64 = chunks.iter().map(|c| c.downloaded).sum();
    // 优先用明确写入的文件总大小；无则从分片大小求和（分片下载场景）
    let total_size: u64 = if file_total_size > 0 {
        file_total_size
    } else {
        chunks.iter().map(|c| c.total_size).sum()
    };
    let speed_bps: u64 = chunks.iter().map(|c| c.speed_bps).sum();
    let percent = if total_size > 0 {
        (total_downloaded as f32 / total_size as f32 * 100.0).min(100.0)
    } else {
        0.0
    };

    FileProgressInfo {
        file_id,
        total_downloaded,
        total_size,
        percent,
        speed_bps,
        chunks,
        status: DownloadStatus::InProgress,
    }
}

// ──────────────────────────────────────────
// FileProgressTracker
// ──────────────────────────────────────────

pub struct FileProgressTracker {
    /// deadpool 连接池：用于 HSET / HGETALL / DEL / PUBLISH
    redis: deadpool_redis::Pool,
    /// 懒建立的 pub/sub 专用客户端（首次订阅时创建）
    app_core: Arc<AppCore>,
    redis_client: Arc<OnceCell<redis::Client>>,
    /// Redis key TTL（秒）：取任务最大超时 + 30s 余量
    progress_ttl: i64,
    /// 速度采样状态：(file_id, chunk_id) → SpeedSample
    speed_state: Mutex<HashMap<(u64, u64), SpeedSample>>,
    /// 限制同时存活的 pub/sub 连接数，防止打爆 Redis
    subscribe_sem: Arc<Semaphore>,
    /// 统一写 Redis 的 channel 发送端；record_bytes / clear_progress 投递，write_worker 消费
    write_tx: mpsc::Sender<WorkerMsg>,
    /// write_worker 的接收端，仅 run_write_worker() 调用时取出一次
    write_rx: Mutex<Option<mpsc::Receiver<WorkerMsg>>>,
}

impl FileProgressTracker {
    /// `task_timeout_secs`：下载任务的最大超时时间（秒），TTL = task_timeout + 30s 余量
    /// `max_subscribe_conns`：允许同时存活的 pub/sub 连接数上限（超出时 Receiver 立即关闭）
    /// `write_channel_cap`：写入 channel 容量上限，防止生产速度远超消费时内存膨胀
    pub fn new(
        redis: deadpool_redis::Pool,
        app_core: Arc<AppCore>,
        task_timeout_secs: u64,
        max_subscribe_conns: usize,
        write_channel_cap: usize,
    ) -> Self {
        let progress_ttl = (task_timeout_secs as i64).saturating_add(30);
        let (write_tx, write_rx) = mpsc::channel::<WorkerMsg>(write_channel_cap);
        Self {
            redis,
            app_core,
            redis_client: Arc::new(OnceCell::new()),
            progress_ttl,
            speed_state: Mutex::new(HashMap::new()),
            subscribe_sem: Arc::new(Semaphore::new(max_subscribe_conns)),
            write_tx,
            write_rx: Mutex::new(Some(write_rx)),
        }
    }

    // ──────────────────────────────────────
    // 写入端（供下载/上传循环调用）
    // ──────────────────────────────────────

    /// 运行批量写入后台循环，通常通过 `tokio::spawn` 调用。
    /// 只能被调用一次；重复调用会立即返回并打印警告。
    pub async fn run_write_worker(&self) {
        info!("progress_tracker write_worker: starting");
        let rx = self
            .write_rx
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        let Some(rx) = rx else {
            warn!("progress_tracker: run_write_worker called more than once, ignoring");
            return;
        };
        // 复用已有的 redis::Client（与 subscribe 共享同一 Client 实例）
        let client = match self
            .redis_client
            .get_or_try_init(|| lsys_core::app_core::create_redis_client(&self.app_core))
            .await
        {
            Ok(c) => c.clone(),
            Err(e) => {
                warn!(
                    "progress_tracker run_write_worker: redis client init failed: {:?}",
                    e
                );
                return;
            }
        };
        Self::write_worker(client, rx, self.progress_ttl).await;
    }

    /// 后台 write_worker：建立专属连接（不占用连接池），批量取消息去重合并后一次 pipeline 写 Redis。
    /// - 外层循环负责连接 / 重连（参考 listen_notify 模式）
    /// - 内层循环负责收消息、批处理、执行 pipeline
    /// - pipeline 失败即 break 内层循环，外层重建连接
    async fn write_worker(
        client: redis::Client,
        mut rx: mpsc::Receiver<WorkerMsg>,
        progress_ttl: i64,
    ) {
        const MAX_BATCH: usize = 32;
        const SUMMARY_LOG_EVERY_BATCHES: u64 = 60;
        let mut reconnect_attempt: u64 = 0;

        // 外层循环：连接失败 / 断开后重建连接
        'reconnect: loop {
            reconnect_attempt = reconnect_attempt.saturating_add(1);
            let mut conn = match client.get_multiplexed_async_connection().await {
                Ok(c) => {
                    info!(
                        "progress_tracker write_worker: redis connection established, attempt={}",
                        reconnect_attempt
                    );
                    c
                }
                Err(e) => {
                    warn!(
                        "progress_tracker write_worker: connection failed, attempt={}: {}",
                        reconnect_attempt, e
                    );
                    // 等待下一条消息再重试，自然退避紧循环
                    match rx.recv().await {
                        None => {
                            info!(
                                "progress_tracker write_worker: channel closed while reconnecting, exiting"
                            );
                            return;
                        }
                        Some(_) => continue 'reconnect,
                    }
                }
            };
            let mut batch_count_on_conn: u64 = 0;

            // 内层循环：消息处理
            loop {
                // 阻塞等待第一条消息；Sender 全部 drop 后退出
                let first = match rx.recv().await {
                    Some(m) => m,
                    None => {
                        info!("progress_tracker write_worker: channel closed, exiting");
                        return;
                    }
                };

                // 非阻塞地再取最多 MAX_BATCH-1 条，凑成一批
                let mut batch = vec![first];
                while batch.len() < MAX_BATCH {
                    match rx.try_recv() {
                        Ok(m) => batch.push(m),
                        Err(_) => break,
                    }
                }
                let batch_msg_count = batch.len();

                // 拆分 Update / Clear，同一批次内 Clear 优先级更高
                // 若某 file_id 同时有 Update 和 Clear，丢弃其 Update（Clear 会 DEL key）
                struct UpdateMsg {
                    file_id: u64,
                    chunk_id: u64,
                    current_downloaded: u64,
                    chunk_total_size: u64,
                    file_total_size: u64,
                    speed_bps: u64,
                }
                let mut raw_updates: Vec<UpdateMsg> = Vec::new();
                let mut clears: HashMap<u64, DownloadStatus> = HashMap::new();
                for msg in batch {
                    match msg {
                        WorkerMsg::Update {
                            file_id, chunk_id, current_downloaded,
                            chunk_total_size, file_total_size, speed_bps,
                        } => raw_updates.push(UpdateMsg {
                            file_id, chunk_id, current_downloaded,
                            chunk_total_size, file_total_size, speed_bps,
                        }),
                        WorkerMsg::Clear { file_id, status } => {
                            clears.insert(file_id, status);
                        }
                    }
                }

                // 去重 Update：同 (file_id, chunk_id) 只保留最新；跳过已被 Clear 的 file_id
                let mut seen: HashSet<(u64, u64)> = HashSet::new();
                let deduped: Vec<UpdateMsg> = raw_updates
                    .into_iter()
                    .rev()
                    .filter(|m| !clears.contains_key(&m.file_id) && seen.insert((m.file_id, m.chunk_id)))
                    .collect();

                // 按 file_id 分组，构建字段列表
                struct FileUpdate {
                    progress_key: String,
                    fields: Vec<(String, String)>,
                    payload: String,
                }
                let mut by_file: HashMap<u64, (Vec<UpdateMsg>, u64)> = HashMap::new();
                for m in deduped {
                    let fts = m.file_total_size;
                    let (msgs, max_fts) = by_file.entry(m.file_id).or_default();
                    if fts > *max_fts {
                        *max_fts = fts;
                    }
                    msgs.push(m);
                }
                let mut file_updates: Vec<FileUpdate> = Vec::with_capacity(by_file.len());
                for (file_id, (msgs, file_total_size)) in by_file {
                    let progress_key = format!("{}{}", PROGRESS_KEY_PREFIX, file_id);
                    let mut fields: Vec<(String, String)> = Vec::new();
                    let mut payload_updates: Vec<String> = Vec::new();
                    for m in &msgs {
                        fields.push((format!("c{}:dl", m.chunk_id), m.current_downloaded.to_string()));
                        fields.push((format!("c{}:sz", m.chunk_id), m.chunk_total_size.to_string()));
                        fields.push((format!("c{}:spd", m.chunk_id), m.speed_bps.to_string()));
                        payload_updates.push(format!(
                            "{},{},{},{}",
                            m.chunk_id, m.current_downloaded, m.chunk_total_size, m.speed_bps
                        ));
                    }
                    if file_total_size > 0 {
                        fields.push(("file:sz".to_string(), file_total_size.to_string()));
                    }
                    // payload 结构：u|{file_id}|{chunk_id,downloaded,total_size,speed;...}|{file_total_size}
                    let payload = format!(
                        "u|{}|{}|{}",
                        file_id,
                        payload_updates.join(";"),
                        file_total_size
                    );
                    file_updates.push(FileUpdate {
                        progress_key,
                        fields,
                        payload,
                    });
                }

                if file_updates.is_empty() && clears.is_empty() {
                    continue;
                }

                // 一次 pipeline：先写所有 Update（HSET + EXPIRE + PUBLISH "1"），
                // 再写所有 Clear（PUBLISH 终态 + DEL），保证有序
                let clear_keys: Vec<(String, &'static str)> = clears
                    .iter()
                    .map(|(file_id, status)| {
                        let key = format!("{}{}", PROGRESS_KEY_PREFIX, file_id);
                        let payload = match status {
                            DownloadStatus::Completed => "done",
                            DownloadStatus::Failed => "fail",
                            DownloadStatus::InProgress => "1",
                        };
                        (key, payload)
                    })
                    .collect();
                let mut p = pipe();
                for upd in &file_updates {
                    p.hset_multiple(&upd.progress_key, &upd.fields)
                        .expire(&upd.progress_key, progress_ttl)
                        .publish::<_, _>(&upd.progress_key, &upd.payload);
                }
                for (key, payload) in &clear_keys {
                    p.publish::<_, _>(key, *payload).del(key);
                }
                let update_file_count = file_updates.len();
                let clear_file_count = clear_keys.len();
                if let Err(e) = p.query_async::<()>(&mut conn).await {
                    warn!(
                        "progress_tracker write_worker: redis pipeline failed, attempt={}, queued_msgs={}, update_files={}, clear_files={}: {}",
                        reconnect_attempt,
                        batch_msg_count,
                        update_file_count,
                        clear_file_count,
                        e
                    );
                    // 连接可能已断开，break 内层循环回到外层重连
                    break;
                } else {
                    batch_count_on_conn = batch_count_on_conn.saturating_add(1);
                    if clear_file_count > 0
                        || batch_count_on_conn % SUMMARY_LOG_EVERY_BATCHES == 0
                    {
                        info!(
                            "progress_tracker write_worker: batch flushed, attempt={}, batch_no={}, queued_msgs={}, update_files={}, clear_files={}",
                            reconnect_attempt,
                            batch_count_on_conn,
                            batch_msg_count,
                            update_file_count,
                            clear_file_count
                        );
                    }
                }
            }
        }
    }

    /// 记录已传输字节数；内部计算瞬时速度，投递到写 channel，由 write_worker 批量写 Redis。
    ///
    /// - `chunk_id = 0`：非分片传输（single file）
    /// - `chunk_total_size`：本分片的预期大小（非分片时与 `file_total_size` 相同）
    /// - `file_total_size`：整个文件的总大小；0 表示未知（仅分片场景可能未知）
    /// - 调用频率与状态检查节拍一致（约 1s），fire-and-forget，不阻塞下载循环
    pub fn record_bytes(
        &self,
        file_id: u64,
        chunk_id: u64,
        current_downloaded: u64,
        chunk_total_size: u64,
        file_total_size: u64,
    ) {
        info!(
            "progress_tracker record_bytes: recv file_id={}, chunk_id={}, downloaded={}, chunk_total_size={}, file_total_size={}",
            file_id,
            chunk_id,
            current_downloaded,
            chunk_total_size,
            file_total_size
        );
        // 计算瞬时速度（bytes/sec）
        let speed_bps = {
            let mut state = self.speed_state.lock().unwrap_or_else(|e| e.into_inner());
            let key = (file_id, chunk_id);
            let now = Instant::now();
            let speed = match state.get(&key) {
                Some(prev) => {
                    let elapsed = prev.last_instant.elapsed().as_secs_f64();
                    if elapsed > 0.0 {
                        ((current_downloaded.saturating_sub(prev.last_bytes)) as f64 / elapsed)
                            as u64
                    } else {
                        0
                    }
                }
                None => 0,
            };
            state.insert(
                key,
                SpeedSample {
                    last_bytes: current_downloaded,
                    last_instant: now,
                },
            );
            speed
        };

        // channel 满时直接丢弃（进度更新可接受少量丢失，绝不阻塞下载循环）
        if let Err(e) = self.write_tx.try_send(WorkerMsg::Update {
            file_id,
            chunk_id,
            current_downloaded,
            chunk_total_size,
            file_total_size,
            speed_bps,
        }) {
            warn!(
                "progress_tracker: write channel full or closed, dropping update file_id={}: {}",
                file_id, e
            );
        } else {
            info!(
                "progress_tracker record_bytes: queued file_id={}, chunk_id={}, downloaded={}, speed_bps={}",
                file_id,
                chunk_id,
                current_downloaded,
                speed_bps
            );
        }
    }

    /// 下载完成或失败后清理 Redis 进度数据及速度采样状态
    pub fn clear_progress(&self, file_id: u64, status: DownloadStatus) {
        {
            let mut state = self.speed_state.lock().unwrap_or_else(|e| e.into_inner());
            state.retain(|(fid, _), _| *fid != file_id);
        }
        // 通过统一 channel 投递，由 write_worker 串行处理（保证在后续 Update 之后执行）
        if let Err(e) = self.write_tx.try_send(WorkerMsg::Clear { file_id, status }) {
            warn!(
                "progress_tracker: write channel full or closed, dropping clear file_id={}: {}",
                file_id, e
            );
        }
    }

    // ──────────────────────────────────────
    // 读取端（供外部查询/订阅）
    // ──────────────────────────────────────

    /// 批量单次查询多个文件的进度。
    /// 用 pipeline 一次取所有 HGETALL，无进度数据的文件 ID 不出现在结果中。
    pub async fn get_progress_batch(&self, file_ids: &[u64]) -> HashMap<u64, FileProgressInfo> {
        let mut result = HashMap::new();
        if file_ids.is_empty() {
            return result;
        }
        let mut conn = match self.redis.get().await {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    "progress_tracker: redis get conn failed on get_progress_batch: {}",
                    e
                );
                return result;
            }
        };
        let keys: Vec<String> = file_ids
            .iter()
            .map(|id| format!("{}{}", PROGRESS_KEY_PREFIX, id))
            .collect();
        let mut p = pipe();
        for key in &keys {
            p.hgetall(key);
        }
        let maps: Vec<HashMap<String, String>> = match p.query_async(&mut conn).await {
            Ok(m) => m,
            Err(e) => {
                warn!(
                    "progress_tracker: redis pipeline HGETALL failed on get_progress_batch: {}",
                    e
                );
                return result;
            }
        };
        for (file_id, map) in file_ids.iter().zip(maps) {
            if !map.is_empty() {
                result.insert(*file_id, parse_progress_from_map(*file_id, map));
            }
        }
        result
    }

    /// 订阅多个文件进度实时推送，一条 Redis 连接同时 SUBSCRIBE 所有 channel。
    /// 返回 `mpsc::Receiver<FileProgressInfo>`，每条消息对应某个文件 ID 的最新进度。
    ///
    /// - 建立连接后先批量发快照（pipeline HGETALL），有数据的文件逐条推送
    /// - 后续任意文件收到 PUBLISH 时，HGETALL 对应 key 后推送
    /// - Receiver drop 时后台任务自动退出
    /// - 多节点：任意节点的 record_bytes PUBLISH 均可被任意节点的订阅者收到
    pub async fn subscribe_progress_batch(
        &self,
        file_ids: &[u64],
    ) -> Result<mpsc::Receiver<FileProgressInfo>, FileError> {
        let (tx, rx) = mpsc::channel::<FileProgressInfo>(64);
        info!("[sse] subscribe_progress_batch called, file_ids={:?}", file_ids);
        if file_ids.is_empty() {
            info!("[sse] file_ids empty, returning immediately");
            return Ok(rx);
        }

        // 建立 channel 名 <-> file_id 的映射
        let channel_map: HashMap<String, u64> = file_ids
            .iter()
            .map(|id| (format!("{}{}", PROGRESS_KEY_PREFIX, id), *id))
            .collect();

        let redis_pool = self.redis.clone();
        let app_core = self.app_core.clone();
        let redis_client_cell = self.redis_client.clone();
        let sem = self.subscribe_sem.clone();

        // 超出连接上限时立即返回错误，调用方可降级为轮询
        let _permit = match sem.try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                return Err(FileError::System(lsys_core::fluent_message!(
                    "file-error",
                    "subscribe_progress_batch rejected: too many concurrent subscribers"
                )));
            }
        };

        // 懒建立 redis::Client（仅首次订阅时创建一次）
        let redis_client = redis_client_cell
            .get_or_try_init(|| async {
                lsys_core::app_core::create_redis_client(&app_core)
                    .await
                    .map_err(FileError::AppCore)
            })
            .await
            .map_err(|e| FileError::System(lsys_core::fluent_message!("file-error", e)))?
            .clone();

        tokio::spawn(async move {
            let channels: Vec<&str> = channel_map.keys().map(|s| s.as_str()).collect();
            info!("[sse] spawn task started, channels={:?}", channels);
            // 本连接内的进度缓存：优先用 payload 增量更新，必要时再回退 HGETALL
            let mut progress_cache: HashMap<u64, FileProgressInfo> = HashMap::new();

            // 先发批量快照（pipeline HGETALL）
            match redis_pool.get().await {
                Err(e) => warn!("progress_tracker: redis get conn failed on snapshot: {}", e),
                Ok(mut conn) => {
                    let mut p = pipe();
                    for ch in &channels {
                        p.hgetall(*ch);
                    }
                    match p
                        .query_async::<Vec<HashMap<String, String>>>(&mut conn)
                        .await
                    {
                        Err(e) => {
                            warn!("progress_tracker: redis pipeline failed on snapshot: {}", e)
                        }
                        Ok(maps) => {
                            for (ch, map) in channels.iter().zip(maps) {
                                info!("[sse] snapshot ch={} map_len={}", ch, map.len());
                                if !map.is_empty()
                                    && let Some(&file_id) = channel_map.get(*ch) {
                                        info!("[sse] snapshot sending file_id={}", file_id);
                                        let info = parse_progress_from_map(file_id, map);
                                        progress_cache.insert(file_id, info.clone());
                                        if let Err(e) = tx.try_send(info) {
                                            warn!(
                                                "[sse] snapshot send failed file_id={}: {}",
                                                file_id, e
                                            );
                                        }
                                    }
                            }
                        }
                    }
                }
            }

            // 一条连接同时订阅所有 channel
            let mut pubsub = match redis_client.get_async_pubsub().await {
                Ok(p) => p,
                Err(e) => {
                    warn!("progress_tracker: get_async_pubsub failed: {}", e);
                    return;
                }
            };
            if let Err(e) = pubsub.subscribe(channels.as_slice()).await {
                warn!("progress_tracker: subscribe failed: {}", e);
                return;
            }
            info!("[sse] subscribed to channels, entering loop");
            let mut stream = pubsub.on_message();
            // 跟踪尚未到达终态的 file_id，所有文件结束后主动退出
            let mut pending: HashSet<u64> = channel_map.values().copied().collect();

            loop {
                // Receiver 已 drop 时提前退出，不等下一条消息
                if tx.is_closed() {
                    break;
                }
                // 同时监听 pub/sub 消息和 Receiver 关闭，任意一个触发就响应
                let msg = tokio::select! {
                    msg = stream.next() => msg,
                    _ = tx.closed() => break,
                };
                let Some(msg) = msg else { break };
                {
                    // 从消息的 channel 名反解 file_id
                    let ch: String = msg.get_channel_name().to_string();
                    let Some(&file_id) = channel_map.get(&ch) else {
                        continue;
                    };

                    let payload = msg.get_payload::<String>().unwrap_or_default();
                    info!("[sse] recv msg ch={} payload={}", ch, payload);
                    match payload.as_str() {
                        "done" | "fail" => {
                            // 终态消息：发终态通知，从 pending 移除，全部结束则退出
                            let (status, percent) = if payload == "done" {
                                (DownloadStatus::Completed, 100.0f32)
                            } else {
                                (DownloadStatus::Failed, 0.0f32)
                            };
                            progress_cache.remove(&file_id);
                            if let Err(e) = tx
                                .send(FileProgressInfo {
                                    file_id,
                                    percent,
                                    status,
                                    ..Default::default()
                                })
                                .await
                            {
                                warn!(
                                    "[sse] send terminal state failed file_id={}: {}",
                                    file_id, e
                                );
                            }
                            pending.remove(&file_id);
                            if pending.is_empty() {
                                break;
                            }
                        }
                        p if p.starts_with("u|") => {
                            // 增量 payload：u|{file_id}|{chunk_id,downloaded,total_size,speed;...}|{file_total_size}
                            let parts: Vec<&str> = p.splitn(4, '|').collect();
                            if parts.len() != 4 {
                                warn!("[sse] invalid update payload format, ch={}", ch);
                                continue;
                            }
                            let payload_file_id = parts[1].parse::<u64>().unwrap_or(0);
                            if payload_file_id != file_id {
                                warn!(
                                    "[sse] payload file_id mismatch, ch_file_id={}, payload_file_id={}",
                                    file_id, payload_file_id
                                );
                                continue;
                            }
                            let payload_file_total_size = parts[3].parse::<u64>().unwrap_or(0);

                            let entry = progress_cache.entry(file_id).or_insert_with(|| FileProgressInfo {
                                file_id,
                                status: DownloadStatus::InProgress,
                                ..Default::default()
                            });

                            if !parts[2].is_empty() {
                                for item in parts[2].split(';') {
                                    if item.is_empty() {
                                        continue;
                                    }
                                    let vals: Vec<&str> = item.split(',').collect();
                                    if vals.len() != 4 {
                                        continue;
                                    }
                                    let chunk_id = vals[0].parse::<u64>().unwrap_or(0);
                                    let downloaded = vals[1].parse::<u64>().unwrap_or(0);
                                    let total_size = vals[2].parse::<u64>().unwrap_or(0);
                                    let speed_bps = vals[3].parse::<u64>().unwrap_or(0);
                                    if let Some(c) = entry.chunks.iter_mut().find(|c| c.chunk_id == chunk_id) {
                                        c.downloaded = downloaded;
                                        c.total_size = total_size;
                                        c.speed_bps = speed_bps;
                                    } else {
                                        entry.chunks.push(ChunkProgressInfo {
                                            chunk_id,
                                            downloaded,
                                            total_size,
                                            speed_bps,
                                        });
                                    }
                                }
                            }

                            entry.chunks.sort_by_key(|c| c.chunk_id);
                            entry.total_downloaded = entry.chunks.iter().map(|c| c.downloaded).sum();
                            if payload_file_total_size > 0 {
                                entry.total_size = payload_file_total_size;
                            } else if entry.total_size == 0 {
                                entry.total_size = entry.chunks.iter().map(|c| c.total_size).sum();
                            }
                            entry.speed_bps = entry.chunks.iter().map(|c| c.speed_bps).sum();
                            entry.percent = if entry.total_size > 0 {
                                (entry.total_downloaded as f32 / entry.total_size as f32 * 100.0)
                                    .min(100.0)
                            } else {
                                0.0
                            };
                            entry.status = DownloadStatus::InProgress;

                            if let Err(e) = tx.send(entry.clone()).await {
                                warn!("[sse] send progress failed file_id={}: {}", file_id, e);
                                break;
                            }
                        }
                        _ => {
                            // 兼容旧 payload（如 "1"）：回退 HGETALL 取最新完整状态
                            let info = match redis_pool.get().await {
                                Err(e) => {
                                    warn!(
                                        "progress_tracker: redis get conn failed in subscribe loop, file_id={}: {}",
                                        file_id, e
                                    );
                                    None
                                }
                                Ok(mut conn) => {
                                    match conn.hgetall::<_, HashMap<String, String>>(&ch).await {
                                        Err(e) => {
                                            warn!(
                                                "progress_tracker: HGETALL failed in subscribe loop, file_id={}: {}",
                                                file_id, e
                                            );
                                            None
                                        }
                                        Ok(m) if m.is_empty() => None,
                                        Ok(m) => Some(parse_progress_from_map(file_id, m)),
                                    }
                                }
                            };
                            if let Some(info) = info {
                                progress_cache.insert(file_id, info.clone());
                                info!("[sse] sending progress file_id={} dl={} pct={:.1}", info.file_id, info.total_downloaded, info.percent);
                                if let Err(e) = tx.send(info).await {
                                    warn!("[sse] send progress failed file_id={}: {}", file_id, e);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}
