import { TimeoutInput } from '@shared/components/custom/input/timeout-input';
import { Alert, AlertDescription } from "@shared/components/ui/alert";
import { Input } from "@shared/components/ui/input";
import { Label } from "@shared/components/ui/label";
import { cn } from "@shared/lib/utils";
import { Clock, FileText, Globe, Hash, InfoIcon, Mail, Smartphone } from "lucide-react";
import { Controller, UseFormReturn } from "react-hook-form";

/**
 * Config Type 1: 关闭功能
 * 不需要任何输入，config_data 为空字符串
 */
export function SenderRuleConfigType1Add() {
    return (
        <Alert className={cn("alert-icon-centered")}>
            <InfoIcon className="h-4 w-4" />
            <AlertDescription>
                将关闭功能
            </AlertDescription>
        </Alert>
    );
}

/**
 * Config Type 2: 频率限制
 * config_data 格式: { range_time: number, max_send: number }
 * range_time: 时间范围（秒）
 * max_send: 最大发送数量
 */
interface SenderRuleConfigType2AddProps {
    form: UseFormReturn<any>;
    fieldPrefix?: string;
}

export function SenderRuleConfigType2Add({ form, fieldPrefix = "config_data" }: SenderRuleConfigType2AddProps) {
    const rangeTimeField = `${fieldPrefix}.range_time`;
    const maxSendField = `${fieldPrefix}.max_send`;

    return (
        <div className="space-y-4">
            <div className="space-y-2">
                <Label>时间范围（秒）</Label>
                <div className="flex items-center gap-2">
                    <Clock className="h-4 w-4 text-muted-foreground" />
                    <Controller
                        control={form.control}
                        name={rangeTimeField}
                        render={({ field }) => (
                            <TimeoutInput
                                value={field.value}
                                onChange={field.onChange}
                                placeholder="例如: 3600"
                            />
                        )}
                    />
                </div>
                <p className="text-[0.8rem] text-muted-foreground">
                    限制的时间范围，单位为秒（例如：3600 = 1小时）
                </p>
            </div>

            <div className="space-y-2">
                <Label>最大发送数量</Label>
                <div className="flex items-center gap-2">
                    <Mail className="h-4 w-4 text-muted-foreground" />
                    <Input
                        type="number"
                        placeholder="例如: 100"
                        min={1}
                        {...form.register(maxSendField, { valueAsNumber: true })}
                    />
                </div>
                <p className="text-[0.8rem] text-muted-foreground">
                    在指定时间范围内最多可发送的数量
                </p>
            </div>
        </div>
    );
}

/**
 * Config Type 3: 每次最大发送数量
 * config_data 格式: number
 * 表示每次最多可发送的邮件数量
 */
interface SenderRuleConfigType3AddProps {
    form: UseFormReturn<any>;
    fieldName?: string;
}

export function SenderRuleConfigType3Add({ form, fieldName = "config_data" }: SenderRuleConfigType3AddProps) {
    return (
        <div className="space-y-2">
            <Label>每次最大发送数量</Label>
            <div className="flex items-center gap-2">
                <Hash className="h-4 w-4 text-muted-foreground" />
                <Input
                    type="number"
                    placeholder="例如: 50"
                    min={1}
                    {...form.register(fieldName, { valueAsNumber: true })}
                />
            </div>
            <p className="text-[0.8rem] text-muted-foreground">
                单次批量发送时的最大数量限制，必须大于0
            </p>
        </div>
    );
}

/**
 * Config Type 4: 指定模板不检测限制
 * config_data 格式: string
 * 模板键值，该模板发送邮件时不进行频率限制检测
 */
interface SenderRuleConfigType4AddProps {
    form: UseFormReturn<any>;
    fieldName?: string;
}

export function SenderRuleConfigType4Add({ form, fieldName = "config_data" }: SenderRuleConfigType4AddProps) {
    return (
        <div className="space-y-2">
            <Label>模板键值</Label>
            <div className="flex items-center gap-2">
                <FileText className="h-4 w-4 text-muted-foreground" />
                <Input
                    type="text"
                    placeholder="例如: welcome_email"
                    {...form.register(fieldName)}
                />
            </div>
            <p className="text-[0.8rem] text-muted-foreground">
                指定的模板键值，使用该模板发送时将跳过频率限制检测
            </p>
        </div>
    );
}

/**
 * Config Type 10: 屏蔽指定手机号
 */
interface SenderRuleConfigType10AddProps {
    form: UseFormReturn<any>;
    fieldName?: string;
}

export function SenderRuleConfigType10Add({ form, fieldName = "config_data" }: SenderRuleConfigType10AddProps) {
    const areaField = `${fieldName}.area`;
    const mobileField = `${fieldName}.mobile`;

    return (
        <div className="space-y-4">
            <div className="space-y-2">
                <Label>区号</Label>
                <div className="flex items-center gap-2">
                    <Smartphone className="h-4 w-4 text-muted-foreground" />
                    <Input
                        type="text"
                        placeholder="例如: 86"
                        {...form.register(areaField)}
                    />
                </div>
                <p className="text-[0.8rem] text-muted-foreground">国家或地区区号（不含+号）</p>
            </div>

            <div className="space-y-2">
                <Label>手机号</Label>
                <div className="flex items-center gap-2">
                    <Smartphone className="h-4 w-4 text-muted-foreground" />
                    <Input
                        type="text"
                        placeholder="例如: 13800138000"
                        {...form.register(mobileField)}
                    />
                </div>
                <p className="text-[0.8rem] text-muted-foreground">填写完整手机号（不含区号）</p>
            </div>
        </div>
    );
}

/**
 * Config Type 20: 指定邮箱屏蔽
 * config_data 格式: string
 * 屏蔽的邮箱地址，该邮箱将无法接收邮件
 */
interface SenderRuleConfigType20AddProps {
    form: UseFormReturn<any>;
    fieldName?: string;
}

export function SenderRuleConfigType20Add({ form, fieldName = "config_data" }: SenderRuleConfigType20AddProps) {
    return (
        <div className="space-y-2">
            <Label>屏蔽邮箱地址</Label>
            <div className="flex items-center gap-2">
                <Mail className="h-4 w-4 text-muted-foreground" />
                <Input
                    type="email"
                    placeholder="例如: blocked@example.com"
                    {...form.register(fieldName)}
                />
            </div>
            <p className="text-[0.8rem] text-muted-foreground">
                指定需要屏蔽的邮箱地址，向该地址发送的邮件将被拒绝
            </p>
        </div>
    );
}

/**
 * Config Type 21: 指定域名屏蔽
 * config_data 格式: string
 * 屏蔽的域名，该域名下的所有邮箱将无法接收邮件
 */
interface SenderRuleConfigType21AddProps {
    form: UseFormReturn<any>;
    fieldName?: string;
}

export function SenderRuleConfigType21Add({ form, fieldName = "config_data" }: SenderRuleConfigType21AddProps) {
    return (
        <div className="space-y-2">
            <Label>屏蔽域名</Label>
            <div className="flex items-center gap-2">
                <Globe className="h-4 w-4 text-muted-foreground" />
                <Input
                    type="text"
                    placeholder="例如: spam-domain.com"
                    {...form.register(fieldName)}
                />
            </div>
            <p className="text-[0.8rem] text-muted-foreground">
                指定需要屏蔽的域名，该域名下所有邮箱地址都将被拒绝发送
            </p>
        </div>
    );
}

/**
 * 统一的配置数据查看组件
 * 根据 config_type 自动选择对应的查看组件
 */
interface SenderRuleConfigFormProps {
    form: UseFormReturn<any>;
    configType: number;
    fieldPrefix?: string;
}

export function SenderRuleConfigForm({ form, configType, fieldPrefix }: SenderRuleConfigFormProps) {
    switch (configType) {
        case 1:
            return <SenderRuleConfigType1Add />;
        case 2:
            return <SenderRuleConfigType2Add form={form} fieldPrefix={fieldPrefix} />;
        case 3:
            return <SenderRuleConfigType3Add form={form} />;
        case 4:
            return <SenderRuleConfigType4Add form={form} />;
        case 10:
            return <SenderRuleConfigType10Add form={form} />;
        case 20:
            return <SenderRuleConfigType20Add form={form} />;
        case 21:
            return <SenderRuleConfigType21Add form={form} />;
        default:
            return;
    }
}

