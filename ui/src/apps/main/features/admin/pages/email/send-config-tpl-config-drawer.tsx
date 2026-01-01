import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@shared/components/ui/select";
import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle
} from "@apps/main/components/local/drawer";
import { useState } from "react";
import { EmailSendConfigTplConfigDrawerSmtpForm } from "./send-config-tpl-config-drawer-smtp-form";
import {
    EMAIL_CHANNEL_OPTIONS,
    EmailChannelType
} from "./send-config-tpl-config-schema";

interface EmailSendConfigTplConfigDrawerProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function EmailSendConfigTplConfigDrawer({
    open,
    onOpenChange,
}: EmailSendConfigTplConfigDrawerProps) {
    const [selectedChannel, setSelectedChannel] = useState<string>(EmailChannelType.SMTP);
    const handleClose = () => {
        setSelectedChannel(EmailChannelType.SMTP);
        onOpenChange(false);
    };



    const handleChannelChange = (value: string) => {
        setSelectedChannel(value);
    };


    // 渠道特定表单，通过 switch 统一生成
    const renderChannelForm = () => {
        switch (selectedChannel) {
            case EmailChannelType.SMTP:
                return (
                    <EmailSendConfigTplConfigDrawerSmtpForm
                        key="smtp-form"
                        onClose={handleClose}
                    />
                );
            default:
                return <div>暂不支持该邮件渠道的配置</div>;
        }
    };

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>新增邮件模板配置</DrawerTitle>
                    <DrawerDescription>
                        选择邮件渠道并填写配置信息
                    </DrawerDescription>
                </DrawerHeader>

                <div className="space-y-4 mt-6">
                    {/* 邮件渠道选择 */}
                    <div className="space-y-2">
                        <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                            邮件渠道
                        </label>
                        <Select
                            onValueChange={handleChannelChange}
                            value={selectedChannel}
                        >
                            <SelectTrigger>
                                <SelectValue placeholder="选择邮件渠道" />
                            </SelectTrigger>
                            <SelectContent className="max-h-[300px]">
                                {EMAIL_CHANNEL_OPTIONS.map((option) => (
                                    <SelectItem key={option.value} value={option.value}>
                                        {option.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    {/* 渠道特定表单 */}
                    {renderChannelForm()}
                </div>
            </DrawerContent>
        </Drawer>
    );
}
