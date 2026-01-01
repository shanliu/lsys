import { mobileAdd } from '@shared/apis/user/profile';
import { LoadingButton } from '@apps/main/components/local/sender-config/loading-button';
import { Button } from '@shared/components/ui/button';
import { Input } from '@shared/components/ui/input';
import { Label } from '@shared/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@shared/components/ui/select';
import { Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer';
import { useToast } from '@shared/contexts/toast-context';
import { cn } from '@shared/lib/utils';
import { formatServerError } from '@shared/lib/utils/format-utils';
import { useMutation } from '@tanstack/react-query';
import React, { useEffect, useState } from 'react';
import { mobileFormSchema, type MobileFormType } from './mobile-schema';

// 区号选项
const areaCodeOptions = [
  { value: '+86', label: '+86 中国大陆' },
];

interface MobileFormDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export const MobileFormDrawer: React.FC<MobileFormDrawerProps> = ({
  isOpen,
  onClose,
  onSuccess,
}) => {
  const { success, error } = useToast();

  // 表单数据
  const [formData, setFormData] = useState<MobileFormType>({ area_code: '+86', mobile: '' });

  // 添加手机号
  const addMobileMutation = useMutation({
    mutationFn: (data: { area_code: string; mobile: string }) => mobileAdd(data),
    onSuccess: () => {
      success('手机号添加成功');
      onClose();
      resetForm();
      onSuccess();
    },
    onError: (err: any) => {
      const errorMessage = formatServerError(err);
      error(errorMessage);
    },
  });

  // 重置表单
  const resetForm = () => {
    setFormData({ area_code: '+86', mobile: '' });
  };

  // 当抽屉打开时重置表单
  useEffect(() => {
    if (isOpen) {
      resetForm();
    }
  }, [isOpen]);

  // 处理输入框变化
  const handleInputChange = (field: keyof MobileFormType) => (value: string) => {
    setFormData({
      ...formData,
      [field]: value
    });
  };

  // 处理表单提交
  const handleSubmit = () => {
    const result = mobileFormSchema.safeParse(formData);
    if (!result.success) {
      const firstError = result.error.errors[0];
      error(firstError.message);
      return;
    }
    addMobileMutation.mutate(result.data);
  };

  // 关闭抽屉
  const handleClose = () => {
    onClose();
    resetForm();
  };

  return (
    <Drawer open={isOpen} onOpenChange={(open) => !open && handleClose()}>
      <DrawerContent className={cn("bg-background text-foreground border-border ")}>
        <DrawerHeader>
          <DrawerTitle className={cn("text-foreground")}>
            添加手机号
          </DrawerTitle>
          <DrawerDescription className={cn("text-muted-foreground")}>
            填写手机号信息，添加后需要验证
          </DrawerDescription>
        </DrawerHeader>

        <div className="space-y-6 mt-6 overflow-y-auto flex-1">
          <div className="space-y-2">
            <Label htmlFor="area_code" className={cn("text-foreground")}>区号 *</Label>
            <Select value={formData.area_code} onValueChange={handleInputChange('area_code')} disabled={addMobileMutation.isPending}>
              <SelectTrigger className={cn("bg-background text-foreground border-border")}>
                <SelectValue placeholder="选择区号" />
              </SelectTrigger>
              <SelectContent className={cn("bg-background border-border max-h-[300px]")}>
                {areaCodeOptions.map((option) => (
                  <SelectItem
                    key={option.value}
                    value={option.value}
                    className={cn("text-foreground hover:bg-muted")}
                  >
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="mobile" className={cn("text-foreground")}>手机号码 *</Label>
            <Input
              id="mobile"
              type="tel"
              placeholder="请输入手机号码"
              value={formData.mobile}
              onChange={(e) => handleInputChange('mobile')(e.target.value)}
              disabled={addMobileMutation.isPending}
              className={cn("bg-background text-foreground border-border placeholder:text-muted-foreground")}
            />
          </div>

          <div className="flex gap-3 pt-4">
            <Button
              onClick={handleClose}
              variant="outline"
              className={cn("flex-1 border-border text-foreground hover:bg-muted")}
            >
              取消
            </Button>
            <LoadingButton
              onClick={handleSubmit}
              className={cn("flex-1 bg-primary text-primary-foreground hover:bg-primary/90")}
              loading={addMobileMutation.isPending}
            >
              添加手机号
            </LoadingButton>
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  );
};
