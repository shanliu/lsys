

interface SenderTplConfigViewProps {
    config_data: any;
    setting_key: string;
    variant?: "simple" | "detail";
}

export function SenderTplConfigView({ config_data, setting_key, variant = "simple" }: SenderTplConfigViewProps) {
    const data = config_data || {};

    switch (setting_key) {
        case "smtp-config":
            if (variant === "simple") {
                return (
                    <div className="space-y-1 text-xs">
                        <div className="truncate max-w-[200px]" title={data.from_email}>
                            <span className="text-muted-foreground">发件人:</span> {data.from_email || "-"}
                        </div>
                        <div className="truncate max-w-[200px]" title={data.subject_tpl_id}>
                            <span className="text-muted-foreground">主题:</span> {data.subject_tpl_id || "-"}
                        </div>
                    </div>
                );
            }

            return (
                <div className="space-y-3 text-sm">
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">发件人邮箱</span>
                            <span className="font-medium">{data.from_email || "-"}</span>
                        </div>
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">回复邮箱</span>
                            <span className="font-medium">{data.reply_email || "-"}</span>
                        </div>
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">主题模板ID</span>
                            <span className="font-medium font-mono">{data.subject_tpl_id || "-"}</span>
                        </div>
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">正文模板ID</span>
                            <span className="font-medium font-mono">{data.body_tpl_id || "-"}</span>
                        </div>
                    </div>
                </div>
            );

        case "tenyun-sms-config":
            if (variant === "simple") {
                return (
                    <div className="space-y-1 text-xs">
                        <div className="truncate max-w-[200px]" title={data.sign_name}>
                            <span className="text-muted-foreground">签名:</span> {data.sign_name || "-"}
                        </div>
                        <div className="truncate max-w-[200px]" title={data.template_id}>
                            <span className="text-muted-foreground">模板ID:</span> {data.template_id || "-"}
                        </div>
                    </div>
                );
            }
            return (
                <div className="space-y-3 text-sm">
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">签名内容</span>
                            <span className="font-medium">{data.sign_name || "-"}</span>
                        </div>
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">模板ID</span>
                            <span className="font-medium font-mono">{data.template_id || "-"}</span>
                        </div>
                        <div className="flex flex-col space-y-1 sm:col-span-2">
                            <span className="text-xs text-muted-foreground">模板变量映射</span>
                            <span className="font-medium font-mono break-all">{data.template_map || "-"}</span>
                        </div>
                    </div>
                </div>
            );

        case "163-sms-config":
        case "col-sms-config":
            if (variant === "simple") {
                return (
                    <div className="space-y-1 text-xs">
                        <div className="truncate max-w-[200px]" title={data.template_id}>
                            <span className="text-muted-foreground">模板ID:</span> {data.template_id || "-"}
                        </div>
                    </div>
                );
            }
            return (
                <div className="space-y-3 text-sm">
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">模板ID</span>
                            <span className="font-medium font-mono">{data.template_id || "-"}</span>
                        </div>
                        <div className="flex flex-col space-y-1 sm:col-span-2">
                            <span className="text-xs text-muted-foreground">模板变量映射</span>
                            <span className="font-medium font-mono break-all">{data.template_map || "-"}</span>
                        </div>
                    </div>
                </div>
            );

        case "jd-cloud-sms-config":
            if (variant === "simple") {
                return (
                    <div className="space-y-1 text-xs">
                        <div className="truncate max-w-[200px]" title={data.sign_id}>
                            <span className="text-muted-foreground">签名ID:</span> {data.sign_id || "-"}
                        </div>
                        <div className="truncate max-w-[200px]" title={data.template_id}>
                            <span className="text-muted-foreground">模板ID:</span> {data.template_id || "-"}
                        </div>
                    </div>
                );
            }
            return (
                <div className="space-y-3 text-sm">
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">签名ID</span>
                            <span className="font-medium">{data.sign_id || "-"}</span>
                        </div>
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">模板ID</span>
                            <span className="font-medium font-mono">{data.template_id || "-"}</span>
                        </div>
                        <div className="flex flex-col space-y-1 sm:col-span-2">
                            <span className="text-xs text-muted-foreground">模板变量映射</span>
                            <span className="font-medium font-mono break-all">{data.template_map || "-"}</span>
                        </div>
                    </div>
                </div>
            );

        case "hwyun-sms-config":
            if (variant === "simple") {
                return (
                    <div className="space-y-1 text-xs">
                        <div className="truncate max-w-[200px]" title={data.sender}>
                            <span className="text-muted-foreground">发送通道:</span> {data.sender || "-"}
                        </div>
                        <div className="truncate max-w-[200px]" title={data.template_id}>
                            <span className="text-muted-foreground">模板ID:</span> {data.template_id || "-"}
                        </div>
                    </div>
                );
            }
            return (
                <div className="space-y-3 text-sm">
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">发送通道号</span>
                            <span className="font-medium">{data.sender || "-"}</span>
                        </div>
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">签名名称</span>
                            <span className="font-medium">{data.signature || "-"}</span>
                        </div>
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">模板ID</span>
                            <span className="font-medium font-mono">{data.template_id || "-"}</span>
                        </div>
                        <div className="flex flex-col space-y-1 sm:col-span-2">
                            <span className="text-xs text-muted-foreground">模板变量映射</span>
                            <span className="font-medium font-mono break-all">{data.template_map || "-"}</span>
                        </div>
                    </div>
                </div>
            );

        case "ali-sms-config":
            if (variant === "simple") {
                return (
                    <div className="space-y-1 text-xs">
                        <div className="truncate max-w-[200px]" title={data.aliyun_sign_name}>
                            <span className="text-muted-foreground">签名:</span> {data.aliyun_sign_name || "-"}
                        </div>
                        <div className="truncate max-w-[200px]" title={data.aliyun_sms_tpl}>
                            <span className="text-muted-foreground">模板Code:</span> {data.aliyun_sms_tpl || "-"}
                        </div>
                    </div>
                );
            }
            return (
                <div className="space-y-3 text-sm">
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">签名名称</span>
                            <span className="font-medium">{data.aliyun_sign_name || "-"}</span>
                        </div>
                        <div className="flex flex-col space-y-1">
                            <span className="text-xs text-muted-foreground">模板Code</span>
                            <span className="font-medium font-mono">{data.aliyun_sms_tpl || "-"}</span>
                        </div>
                    </div>
                </div>
            );

        default:
            return <div className="text-sm text-muted-foreground">不支持的配置类型: {setting_key}</div>;
    }
}

