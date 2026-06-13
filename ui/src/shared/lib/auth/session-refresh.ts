import { accountTokenRefresh } from "@shared/apis/user/account";
import { userStore } from ".";
import { handleLoginResponse } from "./utils";

// 客户端主动刷新 token（非 cookie）：
// 当窗口重新可见 / 获得焦点，且当前会话「临近过期」时，调用 /api/auth/token_refresh
// 轮换出全新 token 并写回 store。旧 token 在服务端轮换后立即失效，因此这里做：
//   - 串行化（同一时刻只允许一个刷新在途，避免并发竞态导致用新旧 token 互相打架）；
//   - 节流（两次刷新之间保留最小间隔）。

// 临近过期阈值（秒）：剩余有效期 <= 该值时触发刷新。
const REFRESH_BEFORE_SECS = 300;
// 两次刷新之间的最小间隔（毫秒），防抖动 / 频繁可见切换。
const MIN_REFRESH_INTERVAL_MS = 60_000;

let lastRefreshAt = 0;
let inFlight: Promise<void> | null = null;
let started = false;

// 当前会话剩余有效期（秒）。`null` 表示无会话或永不过期。
function secondsLeft(): number | null {
    const current = userStore.getState().current();
    if (!current) return null;
    if (current.timeOut <= 0) return null; // 0 表示永不过期
    return current.timeOut - Math.floor(Date.now() / 1000);
}

function shouldRefresh(): boolean {
    if (!userStore.getState().isLoggedIn()) return false;
    const left = secondsLeft();
    if (left === null) return false;
    // 已过期交给正常的 not_login 流程处理，这里只处理「临近过期」。
    if (left <= 0) return false;
    return left <= REFRESH_BEFORE_SECS;
}

async function doRefresh(): Promise<void> {
    // 串行化：同一时刻只允许一个刷新在途。
    if (inFlight) return inFlight;
    if (Date.now() - lastRefreshAt < MIN_REFRESH_INTERVAL_MS) return;
    if (!shouldRefresh()) return;

    lastRefreshAt = Date.now();
    inFlight = (async () => {
        try {
            // auth=true 以取回新的 auth_data（含新 time_out）+ 新 token。
            const res = await accountTokenRefresh({ auth: true });
            if (res.status && res.response) {
                handleLoginResponse(res.response);
            }
        } catch {
            // 刷新失败不阻断使用；真正过期后由正常 not_login 流程处理。
        } finally {
            inFlight = null;
        }
    })();
    return inFlight;
}

// 安装监听：窗口可见 / 获得焦点时检查并按需刷新。幂等，可重复调用。
export function initSessionAutoRefresh(): void {
    if (started) return;
    if (typeof window === "undefined" || typeof document === "undefined") return;
    started = true;

    const onVisible = () => {
        if (document.visibilityState === "visible") {
            void doRefresh();
        }
    };
    document.addEventListener("visibilitychange", onVisible);
    window.addEventListener("focus", onVisible);

    // 首次挂载时也检查一次（例如刚打开页面、token 已临近过期）。
    void doRefresh();
}
