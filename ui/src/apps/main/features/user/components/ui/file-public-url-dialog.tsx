import { ContentDialog } from '@shared/components/custom/dialog/content-dialog';
import { Button } from '@shared/components/ui/button';
import { Input } from '@shared/components/ui/input';
import { useToast } from '@shared/contexts/toast-context';
import React from 'react';

interface FilePublicUrlDialogProps {
    /** 触发弹窗的子元素 */
    children: React.ReactNode;
    /** 公开访问的完整 URL */
    url: string;
}

export function FilePublicUrlDialog({ children, url }: FilePublicUrlDialogProps) {
    const { success: showSuccess } = useToast();

    return (
        <ContentDialog
            title="公开链接"
            content={
                <div className="space-y-3">
                    <p className="text-sm text-muted-foreground">
                        该文件为公开文件，任何人可通过以下链接直接访问：
                    </p>
                    <div className="flex gap-2">
                        <Input readOnly value={url} className="text-xs" />
                        <Button
                            size="sm"
                            variant="outline"
                            onClick={() => {
                                navigator.clipboard.writeText(url);
                                showSuccess('链接已复制');
                            }}
                        >
                            复制
                        </Button>
                    </div>
                </div>
            }
        >
            {children}
        </ContentDialog>
    );
}
