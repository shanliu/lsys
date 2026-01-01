import { emailConfirm, emailSendCode } from '@shared/apis/user/profile';
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
import { emailVerifyFormSchema, type EmailData, type EmailVerifyType } from './email-schema';

interface EmailVerifyDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  email: EmailData | null;
  onSuccess: () => void;
}

export const EmailVerifyDrawer: React.FC<EmailVerifyDrawerProps> = ({
  isOpen,
  onClose,
  email,
  onSuccess,
}) => {
  const { success, error } = useToast();
  const [captchaData, setCaptchaData] = useState<CaptchaData>();
  const [countdown, setCountdown] = useState(0);
  const [formData, setFormData] = useState<EmailVerifyType>({ code: '' });

  // 初始化表单数据
  useEffect(() => {
    if (isOpen) {
      setFormData({ code: '' });
      setCaptchaData(undefined);
      setCountdown(0);
    }
  }, [isOpen]);

  // 倒计时处理
  useEffect(() => {
    if (countdown > 0) {
      const timer = setTimeout(() => setCountdown(countdown - 1), 1000);
      return () => clearTimeout(timer);
    }
  }, [countdown]);

  // 发送验证码
  const sendCodeMutation = useMutation({
    mutationFn: async () => {
      if (!email?.email || !captchaData) throw new Error('缺少必要参数');
      return emailSendCode({
        email: email.email,
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

  // 确认邮箱验证
  const confirmEmailMutation = useMutation({
    mutationFn: (param: { email_id: number; code: string }) => emailConfirm(param),
    onSuccess: () => {
      success('邮箱验证成功');
      onSuccess();
      onClose();
    },
    onError: (err: any) => {
      const errorMessage = formatServerError(err);
      error(errorMessage);
    },
  });

  // 处理发送验证码
  const handleSendCode = () => {
    if (captchaData?.code && captchaData?.key) {
      sendCodeMutation.mutate();
    }
  };

  // 处理输入框变化
  const handleInputChange = (field: keyof EmailVerifyType) => (value: string) => {
    setFormData({
      ...formData,
      [field]: value
    });
  };

  // 处理验证表单提交
  const handleSubmit = () => {
    const result = emailVerifyFormSchema.safeParse(formData);
    if (!result.success) {
      const firstError = result.error.errors[0];
      error(firstError.message);
      return;
    }

    if (!email) return;

    confirmEmailMutation.mutate({
      email_id: email.id,
      code: result.data.code,
    });
  };

  const isSubmitting = confirmEmailMutation.isPending;

  return (
    <Drawer open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DrawerContent className={cn("bg-background text-foreground border-border ")}>
        <DrawerHeader>
          <DrawerTitle className={cn("text-foreground")}>验证邮箱</DrawerTitle>
          <DrawerDescription className={cn("text-muted-foreground")}>
            我们已向 {email?.email} 发送验证码，请查收邮件
          </DrawerDescription>
        </DrawerHeader>

        <div className="space-y-6 mt-6 overflow-y-auto flex-1">
          <div className="space-y-2">
            <Label className={cn("text-foreground")}>发送验证码</Label>
            <div className="flex flex-col sm:flex-row gap-2">
              <CaptchaInput
                captchaType="add-email"
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
              disabled={isSubmitting}
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
              验证邮箱
            </LoadingButton>
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  );
};
