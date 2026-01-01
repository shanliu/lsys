import { emailAdd } from '@shared/apis/user/profile';
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer';
import { LoadingButton } from '@apps/main/components/local/sender-config/loading-button';
import { Button } from '@shared/components/ui/button';
import { Input } from '@shared/components/ui/input';
import { Label } from '@shared/components/ui/label';
import { useToast } from '@shared/contexts/toast-context';
import { cn } from '@shared/lib/utils';
import { formatServerError } from '@shared/lib/utils/format-utils';
import { useMutation } from '@tanstack/react-query';
import React from 'react';
import { emailFormSchema, type EmailFormType } from './email-schema';

interface EmailFormDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export const EmailFormDrawer: React.FC<EmailFormDrawerProps> = ({
  isOpen,
  onClose,
  onSuccess,
}) => {
  const { success, error } = useToast();

  // 表单数据状态
  const [formData, setFormData] = React.useState<EmailFormType>({ email: '' });

  // 初始化表单数据
  React.useEffect(() => {
    if (isOpen) {
      setFormData({ email: '' });
    }
  }, [isOpen]);

  // 添加邮箱
  const addEmailMutation = useMutation({
    mutationFn: (data: { email: string }) => emailAdd(data),
    onSuccess: () => {
      success('邮箱添加成功');
      onSuccess();
      onClose();
    },
    onError: (err: any) => {
      const errorMessage = formatServerError(err);
      error(errorMessage);
    },
  });

  // 处理输入框变化
  const handleInputChange = (field: keyof EmailFormType) => (value: string) => {
    setFormData({
      ...formData,
      [field]: value
    });
  };

  // 处理表单提交
  const handleSubmit = () => {
    const result = emailFormSchema.safeParse(formData);
    if (!result.success) {
      const firstError = result.error.errors[0];
      error(firstError.message);
      return;
    }
    addEmailMutation.mutate(result.data);
  };

  const isSubmitting = addEmailMutation.isPending;

  return (
    <Drawer open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DrawerContent className={cn("bg-background text-foreground border-border ")}>
        <DrawerHeader>
          <DrawerTitle className={cn("text-foreground")}>
            添加邮箱
          </DrawerTitle>
          <DrawerDescription className={cn("text-muted-foreground")}>
            填写邮箱地址信息，添加后需要验证
          </DrawerDescription>
        </DrawerHeader>

        <div className="space-y-6 mt-6 overflow-y-auto flex-1">
          <div className="space-y-2">
            <Label htmlFor="email" className={cn("text-foreground")}>邮箱地址 *</Label>
            <Input
              id="email"
              type="email"
              placeholder="请输入邮箱地址"
              value={formData.email}
              onChange={(e) => handleInputChange('email')(e.target.value)}
              className={cn("bg-background text-foreground border-border placeholder:text-muted-foreground")}
            />
          </div>

          <div className="flex gap-3 pt-4">
            <Button
              onClick={onClose}
              variant="outline"
              className={cn("flex-1 border-border text-foreground hover:bg-muted")}
            >
              取消
            </Button>
            <LoadingButton
              onClick={handleSubmit}
              className={cn("flex-1 bg-primary text-primary-foreground hover:bg-primary/90")}
              loading={isSubmitting}
            >
              添加邮箱
            </LoadingButton>
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  );
};
