/**
 * Web Worker 管理工具，用于后台计算文件 MD5
 * 避免阻塞 UI 主线程
 */

export interface MD5WorkerOptions {
  onProgress?: (progress: number) => void;
}

let workerInstance: Worker | null = null;

/**
 * 检测浏览器是否支持 Web Worker
 */
export function isWorkerSupported(): boolean {
  // 检查 Worker 构造函数是否存在
  if (typeof Worker === 'undefined') {
    return false;
  }

  // 检查是否在有效的环境中（非 SharedArrayBuffer 限制等）
  try {
    // 尝试创建一个简单的 blob worker 来验证
    const blob = new Blob(['console.log("test")'], { type: 'application/javascript' });
    const workerUrl = URL.createObjectURL(blob);
    const testWorker = new Worker(workerUrl);
    testWorker.terminate();
    URL.revokeObjectURL(workerUrl);
    return true;
  } catch (e) {
    console.warn('Web Worker 支持检测失败:', e);
    return false;
  }
}

/**
 * 获取或创建 MD5 Worker 实例
 */
function getWorker(): Worker {
  if (!workerInstance) {
    // 动态导入 worker 模块
    workerInstance = new Worker(
      new URL('../../workers/md5-worker.ts', import.meta.url),
      { type: 'module' }
    );
  }
  return workerInstance;
}

/**
 * 使用 Web Worker 计算文件 MD5
 * @param file 要计算的文件
 * @param options 配置选项
 * @returns Promise<string> 文件的 MD5 哈希值
 */
export function calculateFileMd5WithWorker(
  file: File,
  options: MD5WorkerOptions = {}
): Promise<string> {
  return new Promise((resolve, reject) => {
    const worker = getWorker();
    const { onProgress } = options;
    let timeoutId: NodeJS.Timeout;

    const handleMessage = (event: MessageEvent) => {
      const { type, progress, md5, error } = event.data;

      if (type === 'progress') {
        onProgress?.(progress);
      } else if (type === 'result') {
        cleanup();
        resolve(md5);
      } else if (type === 'error') {
        cleanup();
        reject(new Error(error));
      }
    };

    const handleError = (error: ErrorEvent) => {
      cleanup();
      reject(new Error(`Worker 错误: ${error.message}`));
    };

    const cleanup = () => {
      clearTimeout(timeoutId);
      worker.removeEventListener('message', handleMessage);
      worker.removeEventListener('error', handleError);
    };

    worker.addEventListener('message', handleMessage);
    worker.addEventListener('error', handleError);

    // 发送文件到 Worker 进行计算
    worker.postMessage({
      type: 'calculate',
      file,
      chunkSize: 2 * 1024 * 1024, // 2MB
    });

    // 添加超时保护（如果计算超过 30 分钟就失败）
    timeoutId = setTimeout(() => {
      cleanup();
      reject(new Error('MD5 计算超时'));
    }, 30 * 60 * 1000);
  });
}

/**
 * 取消正在进行的 MD5 计算
 */
export function cancelMD5Calculation(): void {
  if (workerInstance) {
    workerInstance.postMessage({ type: 'cancel' });
  }
}

/**
 * 终止 Worker 并释放资源
 */
export function terminateWorker(): void {
  if (workerInstance) {
    workerInstance.terminate();
    workerInstance = null;
  }
}
