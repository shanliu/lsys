import { mobileConfirm, mobileSendCode } from '@shared/apis/user/profile';
import { LoadingButton } from '@apps/main/components/local/sender-config/loading-button';
import { CaptchaInput, type CaptchaData } from '@shared/components/custom/input/captcha-input';
import { Button } from '@shared/components/ui/button';
import { Input } from '@shared/components/ui/input';
import { Label } from '@shared/components/ui/label';
import { Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer';
import { useToast } from '@shared/contexts/toast-context';
import { cn } from '@shared/lib/utils';
import { formatServerError } from '@shared/lib/utils/format-utils';
import { useMutation } from '@tanstack/react-query';
import React, { useEffect, useState } from 'react';
import { mobileVerifyFormSchema, type MobileData, type MobileVerifyType } from './mobile-schema';

interface MobileVerifyDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  mobile: MobileData;
  onSuccess: () => void;
}

export const MobileVerifyDrawer: React.FC<MobileVerifyDrawerProps> = ({
  isOpen,
  onClose,
  mobile,
  onSuccess,
}) => {
  const { success, error } = useToast();
  const [captchaData, setCaptchaData] = useState<CaptchaData>();
  const [countdown, setCountdown] = useState(0);
  const [formData, setFormData] = useState<MobileVerifyType>({ code: '' });

  // 倒计时处理
  useEffect(() => {
    if (countdown > 0) {
      const timer = setTimeout(() => setCountdown(countdown - 1), 1000);
      return () => clearTimeout(timer);
    }
  }, [countdown]);

  // 当抽屉打开时重置表单
  useEffect(() => {
    if (isOpen) {
      setFormData({ code: '' });
      setCaptchaData(undefined);
      setCountdown(0);
    }
  }, [isOpen]);

  // 发送验证码
  const sendCodeMutation = useMutation({
    mutationFn: async () => {
      if (!mobile || !captchaData) throw new Error('缺少必要参数');
      return mobileSendCode({
        area_code: mobile.area_code,
        mobile: mobile.mobile,
        captcha: {
          code: captchaData.code,
          key: captchaData.key,
        },
      });
    },
    onSuccess: () => {
      success('验证码已发送');
      setCountdown(60);
    },
    onError: (err: any) => {
      const errorMessage = formatServerError(err);
      error(errorMessage);
    },
  });

  // 确认验证
  const confirmMutation = useMutation({
    mutationFn: (param: { mobile_id: number; code: string }) => mobileConfirm(param),
    onSuccess: () => {
      success('手机号验证成功');
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
    setFormData({ code: '' });
    setCaptchaData(undefined);
    setCountdown(0);
  };

  // 处理发送验证码
  const handleSendCode = () => {
    if (captchaData?.code && captchaData?.key) {
      sendCodeMutation.mutate();
    }
  };

  // 处理输入框变化
  const handleInputChange = (field: keyof MobileVerifyType) => (value: string) => {
    setFormData({
      ...formData,
      [field]: value
    });
  };

  // 处理表单提交
  const handleSubmit = () => {
    const result = mobileVerifyFormSchema.safeParse(formData);
    if (!result.success) {
      const firstError = result.error.errors[0];
      error(firstError.message);
      return;
    }

    confirmMutation.mutate({
      mobile_id: mobile.id,
      code: result.data.code,
    });
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
          <DrawerTitle className={cn("text-foreground")}>验证手机号</DrawerTitle>
          <DrawerDescription className={cn("text-muted-foreground")}>
            我们将向 {mobile.area_code} {mobile.mobile} 发送验证码，请注意查收短信
          </DrawerDescription>
        </DrawerHeader>

        <div className="space-y-6 mt-6 overflow-y-auto flex-1">
          <div className="space-y-2">
            <Label className={cn("text-foreground")}>发送验证码</Label>
            <div className="flex flex-col sm:flex-row gap-2">
              <CaptchaInput
                captchaType="add-sms"
                value={captchaData}
                onChange={setCaptchaData}
                className={cn("flex-1 bg-background border-border")}
              />
              <LoadingButton
                onClick={handleSendCode}
                loading={sendCodeMutation.isPending}
                disabled={countdown > 0 || !captchaData?.code}
                className={cn("shrink-0 w-full sm:w-auto")}
              >
                {countdown > 0 ? `${countdown}秒后重发` : '发送验证码'}
              </LoadingButton>
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="code" className={cn("text-foreground")}>验证码 *</Label>
            <Input
              id="code"
              placeholder="请输入6位验证码"
              value={formData.code}
              onChange={(e) => handleInputChange('code')(e.target.value)}
              maxLength={6}
              disabled={confirmMutation.isPending}
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
              loading={confirmMutation.isPending}
            >
              验证手机号
            </LoadingButton>
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  );
};
