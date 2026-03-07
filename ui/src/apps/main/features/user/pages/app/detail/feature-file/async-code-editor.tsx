import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading";
import { useTheme } from "@shared/contexts/theme-context";
import React, { Suspense, useCallback, useMemo, useRef } from "react";

const LazyCodeEditor = React.lazy(() => import("@uiw/react-textarea-code-editor"));

export interface AsyncCodeEditorProps {
    value: string;
    language?: string;
    placeholder?: string;
    onChange?: (value: string) => void;
    readOnly?: boolean;
    minHeight?: number;
    className?: string;
}

const LINE_HEIGHT = 19.5; // 13px * 1.5
const PADDING = 15;

export function AsyncCodeEditor({
    value,
    language = "javascript",
    placeholder = "请输入代码...",
    onChange,
    readOnly = false,
    minHeight = 300,
    className,
}: AsyncCodeEditorProps) {
    const { theme } = useTheme();
    const lineNumbersRef = useRef<HTMLDivElement>(null);

    const colorMode = useMemo(() => {
        if (theme === "system") {
            return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
        }
        return theme;
    }, [theme]);

    const lineCount = useMemo(() => Math.max((value || "").split("\n").length, 1), [value]);

    // React 19 callback ref with cleanup — runs when textarea actually mounts (after lazy load)
    const textareaCallbackRef = useCallback((textarea: HTMLTextAreaElement | null) => {
        if (!textarea) return;
        const onScroll = () => {
            if (lineNumbersRef.current) {
                lineNumbersRef.current.scrollTop = textarea.scrollTop;
            }
        };
        textarea.addEventListener("scroll", onScroll);
        return () => textarea.removeEventListener("scroll", onScroll);
    }, []);

    const isDark = colorMode === "dark";
    const gutterBg = isDark ? "#161b22" : "#f6f8fa";
    const gutterColor = isDark ? "#6e7681" : "#8c959f";
    const gutterBorder = isDark ? "#30363d" : "#d0d7de";

    return (
        <Suspense fallback={<CenteredLoading variant="content" iconSize="md" />}>
            <div
                className="border rounded-md overflow-hidden flex items-stretch"
                data-color-mode={colorMode}
            >
                {/* Line numbers gutter */}
                <div
                    ref={lineNumbersRef}
                    className="select-none shrink-0 overflow-hidden"
                    style={{
                        fontSize: 13,
                        fontFamily:
                            "ui-monospace,SFMono-Regular,SF Mono,Consolas,Liberation Mono,Menlo,monospace",
                        lineHeight: `${LINE_HEIGHT}px`,
                        paddingTop: PADDING,
                        paddingBottom: PADDING,
                        paddingLeft: 10,
                        paddingRight: 10,
                        minHeight,
                        background: gutterBg,
                        color: gutterColor,
                        borderRight: `1px solid ${gutterBorder}`,
                        textAlign: "right",
                    }}
                >
                    {Array.from({ length: lineCount }, (_, i) => (
                        <div key={i + 1} style={{ height: LINE_HEIGHT }}>
                            {i + 1}
                        </div>
                    ))}
                </div>

                {/* Code editor */}
                <div className="flex-1 min-w-0">
                    <LazyCodeEditor
                        ref={textareaCallbackRef}
                        value={value}
                        language={language}
                        placeholder={placeholder}
                        onChange={(evn) => onChange?.(evn.target.value)}
                        padding={PADDING}
                        readOnly={readOnly}
                        minHeight={minHeight}
                        style={{
                            fontSize: 13,
                            fontFamily:
                                "ui-monospace,SFMono-Regular,SF Mono,Consolas,Liberation Mono,Menlo,monospace",
                            lineHeight: `${LINE_HEIGHT}px`,
                            width: "100%",
                        }}
                        className={className}
                    />
                </div>
            </div>
        </Suspense>
    );
}
