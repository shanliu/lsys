import { siteInfo } from '@shared/apis/public/site';
import { queryClient } from '@shared/lib/query-client';
import '@shared/styles/globals.css';
import { QueryClientProvider, useQuery } from '@tanstack/react-query';
import { StrictMode } from 'react';
import * as ReactDOM from 'react-dom/client';
import './styles/home.css';

export function HomePage() {
    const { data: siteTips } = useQuery({
        queryKey: ['siteInfo'],
        queryFn: async () => {
            const res = await siteInfo();
            if (res.status && res.response?.site_tips) {
                return res.response.site_tips;
            }
            return '';
        },
    });

    return (
        <div
            className="home-container"
        >
            {/* The Sloppy Doc */}
            <div className="draft-doc">

                {/* Header Info */}
                <div className="meta-header">
                    <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                        <a href="https://github.com/shanliu/lsys" target="_blank" rel="noreferrer" style={{ color: 'inherit', display: 'flex', alignItems: 'center' }}>
                            <svg style={{ flexShrink: 0 }} width="14" height="14" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                                <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38
                                0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52
                                -.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64
                                -.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18
                                1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56
                                .82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2
                                0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8z"/>
                            </svg>
                        </a>
                        <span className="github-text">: <a href="https://github.com/shanliu/lsys" target="_blank" rel="noreferrer" style={{ color: 'inherit', textDecoration: 'underline' }}>lsys</a></span>
                    </div>
                    <div>Status: WIP (Maybe)</div>
                    <div>Date: 2025-12-14</div>
                </div>

                {/* Intro Spec */}
                <div className="section-block" style={{ borderTop: 'none' }}>
                    <h1 className="draft-title">还在开发... <span className="caret" /></h1>
                    <p className="draft-text">
                        这是Rust+React的实现后台管理系统，反正该有的都有。
                        也没什么好介绍的，自己看吧。
                    </p>
                </div>

                {/* Site Tips - Dynamic */}
                {siteTips && (
                    <div className="section-block">
                        <h2 className="draft-heading">Tips：</h2>
                        <p className="draft-text" style={{ whiteSpace: 'pre-line' }}>
                            {siteTips}
                        </p>
                    </div>
                )}

                {/* Feature List */}
                <div className="section-block">
                    <p className="draft-text">
                        大概有些什么功能：
                    </p>

                    <ul className="spec-list">
                        <div className="spec-item">
                            <span className="spec-key">应用 (App)</span>
                            <span className="spec-val">管些应用的密钥啥的。</span>
                        </div>
                        <div className="spec-item">
                            <span className="spec-key">消息 (Mail or SMS)</span>
                            <span className="spec-val">也就是管理下发送的邮件和短信</span>
                        </div>
                        <div className="spec-item">
                            <span className="spec-key">权限 (RBAC)</span>
                            <span className="spec-val">防闲杂人等的，挺复杂的</span>
                        </div>
                        <div className="spec-item">
                            <span className="spec-key">用户 (User or Account)</span>
                            <span className="spec-val">可能你不需要，自己系统的更好用，可选</span>
                        </div>
                    </ul>
                </div>

                {/* Footer / EOF */}
                <div className="section-block footer-section">
                    <p className="draft-text footer-text">
                        -- 差不多得了 / EOF --
                    </p>
                    <div className="button-row">
                        <button
                            onClick={() => { window.location.href = "/user" }}
                            className="btn-enter-system"
                        >
                            进入系统
                        </button>
                        <button
                            onClick={() => { window.location.href = "/docs" }}
                            className="btn-enter-system"
                            aria-label="查看文档"
                        >
                            查看文档
                        </button>
                        <button
                            onClick={() => { window.open("https://github.com/shanliu/lsys/tree/dev/sdk", "_blank") }}
                            className="btn-enter-system"
                            aria-label="SDK示例"
                        >
                            SDK示例
                        </button>
                    </div>

                    <div className="qr-block" aria-label="二维码">
                        <img
                            className="qr-image"
                            src="https://www.lsys.cc/barcode/base64/1/aHR0cHM6Ly9sc3lzLmNjLw=="
                            alt="手机浏览二维码"
                            loading="lazy"
                        />
                        <div className="qr-caption">手机扫码浏览</div>
                    </div>

                    {/* Signature */}
                    <div className="signature">
                        DESIGNED_BY::GEMINI 3
                    </div>
                    <div style={{ fontSize: 'small', textAlign: 'center', marginTop: '10px' }}>
                        <a href="http://www.lsys.cc:8088" target="_blank" rel="noreferrer" style={{ color: 'gray', textDecoration: 'none' }}>old分支版本示例(旧版,仅支持PC)</a>
                    </div>


                </div>

            </div>
        </div >
    );
}

// 创建 root 元素（如果不存在）
let rootElement = document.getElementById('root')
if (!rootElement) {
    rootElement = document.createElement('div')
    rootElement.id = 'root'
    document.body.appendChild(rootElement)
}

if (rootElement) {
    const root = ReactDOM.createRoot(rootElement)
    root.render(
        <StrictMode>
            <QueryClientProvider client={queryClient}>
                <HomePage />
            </QueryClientProvider>
        </StrictMode>,
    )
}
