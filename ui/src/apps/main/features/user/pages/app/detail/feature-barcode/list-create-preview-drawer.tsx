import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { type TypedDictData } from "@apps/main/hooks/use-dict-data";
import { UserBarcodeCreateConfigItemType } from '@shared/apis/user/barcode';
import { ContentDialog } from "@shared/components/custom/dialog/content-dialog";
import { Button } from "@shared/components/ui/button";
import { Input } from "@shared/components/ui/input";
import { Label } from "@shared/components/ui/label";
import { Config } from "@shared/lib/config";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@shared/components/ui/select";
import { cn } from "@shared/lib/utils";
import React from "react";

interface BarcodeCreateConfigPreviewDrawerProps {
    config: UserBarcodeCreateConfigItemType;
    dictData: TypedDictData<["user_barcode"]>;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function BarcodeCreateConfigPreviewDrawer({
    config,
    dictData,
    open,
    onOpenChange,
}: BarcodeCreateConfigPreviewDrawerProps) {
    const [previewContent, setPreviewContent] = React.useState("Hello World");
    const [contentType, setContentType] = React.useState<"text" | "base64">("text");

    const previewTransformedContent = React.useMemo(() => {
        if (!previewContent) {
            return "";
        }
        return contentType === "text"
            ? previewContent
            : encodeToBase64(previewContent);
    }, [contentType, previewContent]);

    const previewUrl = React.useMemo(() => {
        if (!previewTransformedContent) {
            return "";
        }
        const encodedContent = contentType === "text"
            ? encodeURIComponent(previewTransformedContent)
            : previewTransformedContent;
        return `${Config.apiBaseUrl}/barcode/${contentType}/${config.id}/${encodedContent}`;
    }, [contentType, config.id, previewTransformedContent]);

    const previewContentLabel = contentType === "text" ? "文本内容" : "Base64 内容";

    const barcodeTypeLabel = dictData.barcode_type?.getLabel(config.barcode_type) ?? config.barcode_type;

    return (
        <>
            <Drawer open={open} onOpenChange={onOpenChange}>
                <DrawerContent>
                    <DrawerHeader>
                        <DrawerTitle>条码预览</DrawerTitle>
                        <DrawerDescription>配置 ID: {config.id}</DrawerDescription>
                    </DrawerHeader>

                    <div className="space-y-4 mt-6">
                        {/* 配置信息 */}
                        <div className="rounded-lg border p-4 space-y-3 bg-muted/50">
                            <h3 className="font-semibold text-sm">配置信息</h3>
                            <div className="grid grid-cols-2 gap-3 text-sm">
                                <div className="flex items-center">
                                    <span className="text-muted-foreground">条码类型:</span>
                                    <span className="ml-2 font-mono">{barcodeTypeLabel}</span>
                                </div>
                                <div className="flex items-center">
                                    <span className="text-muted-foreground">图片格式:</span>
                                    <span className="ml-2 font-mono">{config.image_format}</span>
                                </div>
                                <div className="flex items-center">
                                    <span className="text-muted-foreground">尺寸:</span>
                                    <span className="ml-2 font-mono">
                                        {config.image_width} × {config.image_height}
                                    </span>
                                </div>
                                <div className="flex items-center">
                                    <span className="text-muted-foreground">边距:</span>
                                    <span className="ml-2 font-mono">{config.margin}px</span>
                                </div>
                                <div className="flex items-center">
                                    <span className="text-muted-foreground">前景色:</span>
                                    <div
                                        className="w-4 h-4 rounded border ml-2"
                                        style={{ backgroundColor: config.image_color }}
                                    />
                                    <span className="ml-2 font-mono text-xs">{config.image_color}</span>
                                </div>
                                <div className="flex items-center">
                                    <span className="text-muted-foreground">背景色:</span>
                                    <div
                                        className="w-4 h-4 rounded border ml-2"
                                        style={{ backgroundColor: config.image_background }}
                                    />
                                    <span className="ml-2 font-mono text-xs">{config.image_background}</span>
                                </div>
                            </div>
                        </div>

                        {/* 内容类型选择 */}
                        <div>
                            <Label htmlFor="contentType">内容类型</Label>
                            <Select
                                value={contentType}
                                onValueChange={(value) => setContentType(value as "text" | "base64")}
                            >
                                <SelectTrigger id="contentType" className="w-full mt-1.5">
                                    <SelectValue placeholder="选择内容类型" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="text">文本 (text)</SelectItem>
                                    <SelectItem value="base64">Base64 (base64)</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>

                        {/* 内容输入 */}
                        <div>
                            <Label htmlFor="previewContent">
                                {contentType === "text" ? "预览文本" : "Base64 内容"}
                            </Label>
                            <Input
                                id="previewContent"
                                placeholder={
                                    contentType === "text"
                                        ? "输入要生成条码的文本内容"
                                        : "输入 Base64 编码内容"
                                }
                                value={previewContent}
                                onChange={(e) => setPreviewContent(e.target.value)}
                                className={cn("mt-1.5")}
                            />
                        </div>

                        {/* 预览按钮 */}
                        <div className="flex justify-end pt-2">
                            <ContentDialog
                                title="条码预览"
                                className={cn("max-w-2xl")}
                                content={(
                                    <>
                                        <p className="text-sm text-muted-foreground">
                                            {previewContentLabel}: {previewTransformedContent}
                                        </p>
                                        <div className="flex justify-center items-center p-6 bg-muted/30 rounded-lg min-h-[300px] mt-4">
                                            <img
                                                src={previewUrl || undefined}
                                                alt="Barcode Preview"
                                                className="max-w-full max-h-[400px] object-contain"
                                                onError={(e) => {
                                                    const target = e.target as HTMLImageElement;
                                                    target.src = "";
                                                    target.alt = "预览加载失败";
                                                }}
                                            />
                                        </div>
                                        <div className="text-xs text-muted-foreground break-all bg-muted p-3 rounded mt-4">
                                            <strong>预览 URL:</strong>
                                            <div className="mt-1 font-mono">{previewUrl}</div>
                                        </div>
                                    </>
                                )}
                                footer={(closeDialog) => (
                                    <Button variant="outline" onClick={closeDialog}>
                                        关闭
                                    </Button>
                                )}
                            >
                                <Button disabled={!previewContent.trim()}>
                                    生成预览
                                </Button>
                            </ContentDialog>
                        </div>
                    </div>
                </DrawerContent>
            </Drawer>
        </>
    );
}

function encodeToBase64(value: string): string {
    if (!value) {
        return "";
    }

    try {
        if (typeof window !== "undefined" && typeof window.btoa === "function") {
            const bytes = new TextEncoder().encode(value);
            let binary = "";
            bytes.forEach((byte) => {
                binary += String.fromCharCode(byte);
            });
            return window.btoa(binary);
        }
    } catch (error) {
        // ignore and fallback to Buffer branch below
    }

    try {
        if (typeof globalThis !== "undefined") {
            const BufferClass = (globalThis as { Buffer?: { from: (input: string, encoding: string) => { toString: (encoding: string) => string } } }).Buffer;
            if (BufferClass) {
                return BufferClass.from(value, "utf-8").toString("base64");
            }
        }
    } catch (error) {
        // ignore and fall through to empty string
    }

    return "";
}
