import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@shared/components/ui/tooltip";
import { formatSeconds } from '@shared/lib/utils/format-utils';
import { ShieldX } from "lucide-react";

/**
 * Config Type 1: 关闭功能
 * 不需要任何输入，config_data 为空字符串
 */
interface SenderRuleConfigType1ViewProps {
    data: any;
}

export function SenderRuleConfigType1View({ data }: SenderRuleConfigType1ViewProps) {
    return <span className="text-muted-foreground">已关闭</span>;
}

/**
 * Config Type 2: 频率限制
 * config_data 格式: { range_time: number, max_send: number }
 * range_time: 时间范围（秒）
 * max_send: 最大发送数量
 */
interface SenderRuleConfigType2ViewProps {
    data: {
        range_time: number;
        max_send: number;
    };
}

export function SenderRuleConfigType2View({ data }: SenderRuleConfigType2ViewProps) {
    return (
        <TooltipProvider>
            <Tooltip>
                <TooltipTrigger asChild>
                    <span className="cursor-help">
                        {formatSeconds(data.range_time)} / {data.max_send}封
                    </span>
                </TooltipTrigger>
                <TooltipContent>
                    <div className="space-y-1">
                        <div>时间范围: {formatSeconds(data.range_time)}</div>
                        <div>最大发送: {data.max_send}条</div>
                    </div>
                </TooltipContent>
            </Tooltip>
        </TooltipProvider>
    );
}

/**
 * Config Type 3: 每次最大发送数量
 * config_data 格式: number
 * 表示每次最多可发送的邮件数量
 */
interface SenderRuleConfigType3ViewProps {
    data: number;
}

export function SenderRuleConfigType3View({ data }: SenderRuleConfigType3ViewProps) {
    return (
        <TooltipProvider>
            <Tooltip>
                <TooltipTrigger asChild>
                    <span className="cursor-help">{data}条/次</span>
                </TooltipTrigger>
                <TooltipContent>
                    <div>每次最多发送 {data} 条</div>
                </TooltipContent>
            </Tooltip>
        </TooltipProvider>
    );
}

/**
 * Config Type 4: 指定模板不检测限制
 * config_data 格式: string
 * 模板键值，该模板发送邮件时不进行频率限制检测
 */
interface SenderRuleConfigType4ViewProps {
    data: string;
}

export function SenderRuleConfigType4View({ data }: SenderRuleConfigType4ViewProps) {
    const displayText = data.length > 20 ? `${data.substring(0, 20)}...` : data;

    return (
        <TooltipProvider>
            <Tooltip>
                <TooltipTrigger asChild>
                    <span className="cursor-help font-mono text-sm">{displayText}</span>
                </TooltipTrigger>
                <TooltipContent>
                    <div className="max-w-xs break-all">
                        <div className="font-semibold mb-1">模板键值:</div>
                        <div className="font-mono">{data}</div>
                    </div>
                </TooltipContent>
            </Tooltip>
        </TooltipProvider>
    );
}

/**
 * Config Type 10: 屏蔽指定手机号
 */
interface SenderRuleConfigType10ViewProps {
    data: {
        area?: string;
        mobile?: string;
    } | string | null;
}

function maskMobile(mobile?: string) {
    if (!mobile) return "-";
    // mask middle digits, keep first 3 and last 4 if length >=7
    const len = mobile.length;
    if (len <= 4) return mobile;
    if (len <= 7) return `${mobile.slice(0, 2)}***${mobile.slice(-2)}`;
    return `${mobile.slice(0, 3)}****${mobile.slice(-4)}`;
}

export function SenderRuleConfigType10View({ data }: SenderRuleConfigType10ViewProps) {
    let area = "";
    let mobile = "";

    if (!data) {
        return <span className="text-muted-foreground">未配置</span>;
    }

    if (typeof data === "string") {
        try {
            const parsed = JSON.parse(data);
            area = parsed?.area ?? "";
            mobile = parsed?.mobile ?? "";
        } catch (e) {
            // if it's a plain string phone
            mobile = data;
        }
    } else {
        area = data.area ?? "";
        mobile = data.mobile ?? "";
    }

    const display = `${area ? `+${area} ` : ""}${maskMobile(mobile)}`;

    return (
        <TooltipProvider>
            <Tooltip>
                <TooltipTrigger asChild>
                    <span className="cursor-help">{display}</span>
                </TooltipTrigger>
                <TooltipContent>
                    <div className="space-y-1">
                        <div>区号: {area || "-"}</div>
                        <div>手机号: {mobile || "-"}</div>
                    </div>
                </TooltipContent>
            </Tooltip>
        </TooltipProvider>
    );
}

/**
 * Config Type 20: 指定邮箱屏蔽
 * config_data 格式: string
 * 屏蔽的邮箱地址，该邮箱将无法接收邮件
 */
interface SenderRuleConfigType20ViewProps {
    data: string;
}

export function SenderRuleConfigType20View({ data }: SenderRuleConfigType20ViewProps) {
    const displayText = data.length > 30 ? `${data.substring(0, 30)}...` : data;

    return (
        <TooltipProvider>
            <Tooltip>
                <TooltipTrigger asChild>
                    <span className="cursor-help flex items-center gap-1">
                        <ShieldX className="h-3 w-3 text-destructive" />
                        <span className="font-mono text-sm">{displayText}</span>
                    </span>
                </TooltipTrigger>
                <TooltipContent>
                    <div className="max-w-xs break-all">
                        <div className="font-semibold mb-1">屏蔽邮箱:</div>
                        <div className="font-mono">{data}</div>
                    </div>
                </TooltipContent>
            </Tooltip>
        </TooltipProvider>
    );
}

/**
 * Config Type 21: 指定域名屏蔽
 * config_data 格式: string
 * 屏蔽的域名，该域名下的所有邮箱将无法接收邮件
 */
interface SenderRuleConfigType21ViewProps {
    data: string;
}

export function SenderRuleConfigType21View({ data }: SenderRuleConfigType21ViewProps) {
    const displayText = data.length > 30 ? `${data.substring(0, 30)}...` : data;

    return (
        <TooltipProvider>
            <Tooltip>
                <TooltipTrigger asChild>
                    <span className="cursor-help flex items-center gap-1">
                        <ShieldX className="h-3 w-3 text-destructive" />
                        <span className="font-mono text-sm">@{displayText}</span>
                    </span>
                </TooltipTrigger>
                <TooltipContent>
                    <div className="max-w-xs break-all">
                        <div className="font-semibold mb-1">屏蔽域名:</div>
                        <div className="font-mono">@{data}</div>
                        <div className="text-xs text-muted-foreground mt-1">
                            所有 @{data} 的邮箱都将被屏蔽
                        </div>
                    </div>
                </TooltipContent>
            </Tooltip>
        </TooltipProvider>
    );
}

/**
 * 解析 configData，如果是 JSON 字符串则解析为对象
 */
function parseConfigData(configData: any): any {
    if (typeof configData === 'string') {
        try {
            return JSON.parse(configData);
        } catch {
            return configData;
        }
    }
    return configData;
}

/**
 * 统一的配置数据查看组件
 * 根据 config_type 自动选择对应的查看组件
 */
interface SenderRuleConfigViewProps {
    configType: number;
    configData: any;
    displayType: string;
}

export function SenderRuleConfigView({ configType, configData, displayType }: SenderRuleConfigViewProps) {
    const parsedData = parseConfigData(configData);
    
    switch (configType) {
        case 1:
            return <SenderRuleConfigType1View data={parsedData} />;
        case 2:
            return <SenderRuleConfigType2View data={parsedData} />;
        case 3:
            return <SenderRuleConfigType3View data={parsedData} />;
        case 4:
            return <SenderRuleConfigType4View data={parsedData} />;
        case 10:
            return <SenderRuleConfigType10View data={parsedData} />;
        case 20:
            return <SenderRuleConfigType20View data={parsedData} />;
        case 21:
            return <SenderRuleConfigType21View data={parsedData} />;
        default:
            return <span className="text-muted-foreground">未知类型</span>;
    }
}

