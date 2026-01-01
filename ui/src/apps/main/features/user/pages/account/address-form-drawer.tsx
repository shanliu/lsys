import {
  addressAdd,
  addressEdit,
  mobileListData,
  type AddressAddParamType,
  type AddressEditParamType
} from '@shared/apis/user/profile';
import { AutocompleteInput } from '@shared/components/custom/input/autocomplete-input';
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer';
import { LoadingButton } from '@apps/main/components/local/sender-config/loading-button';
import type { AddressSelection } from '@apps/main/components/selector/address-selector';
import { AddressSelector } from '@apps/main/components/selector/address-selector';
import { Button } from '@shared/components/ui/button';
import { Input } from '@shared/components/ui/input';
import { Label } from '@shared/components/ui/label';
import { useToast } from '@shared/contexts/toast-context';
import { cn } from '@shared/lib/utils';
import { formatServerError } from '@shared/lib/utils/format-utils';
import { useMutation, useQuery } from '@tanstack/react-query';
import React from 'react';
import { addressFormSchema, type AddressData, type AddressFormData } from './address-schema';

interface AddressFormDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  editingAddress?: AddressData | null;
  onSuccess: () => void;
}

export const AddressFormDrawer: React.FC<AddressFormDrawerProps> = ({
  isOpen,
  onClose,
  editingAddress,
  onSuccess,
}) => {
  const { success, error } = useToast();

  // 表单数据状态
  const [formData, setFormData] = React.useState<AddressFormData>({
    name: '',
    mobile: '',
    code: '',
    info: '',
    detail: '',
  });

  // 初始化表单数据
  React.useEffect(() => {
    if (isOpen && editingAddress) {
      setFormData({
        name: editingAddress.name,
        mobile: editingAddress.mobile,
        code: editingAddress.address_code,
        info: editingAddress.address_info,
        detail: editingAddress.address_detail
      });
    } else if (isOpen && !editingAddress) {
      setFormData({
        name: '',
        mobile: '',
        code: '',
        info: '',
        detail: '',
      });
    }
  }, [isOpen, editingAddress]);

  // 获取手机号列表数据
  const { data: mobileListRes, isLoading: mobileLoading } = useQuery({
    queryKey: ['mobileList'],
    queryFn: () => mobileListData({}),
    enabled: isOpen, // 只有在抽屉打开时才加载数据
  });

  // 提取手机号选项
  const mobileOptions = React.useMemo(() => {
    if (!mobileListRes?.response?.data) return [];
    return mobileListRes.response.data
      .filter(mobile => mobile.status === 2) // 只显示已验证的手机号
      .map(mobile => mobile.mobile);
  }, [mobileListRes]);

  // 添加地址
  const addAddressMutation = useMutation({
    mutationFn: (param: AddressAddParamType) => addressAdd(param),
    onSuccess: () => {
      success('地址添加成功');
      onSuccess();
      onClose();
    },
    onError: (err: any) => {
      const errorMessage = formatServerError(err);
      error(errorMessage);
    },
  });

  // 编辑地址
  const editAddressMutation = useMutation({
    mutationFn: (param: AddressEditParamType) => addressEdit(param),
    onSuccess: () => {
      success('地址修改成功');
      onSuccess();
      onClose();
    },
    onError: (err: any) => {
      const errorMessage = formatServerError(err);
      error(errorMessage);
    },
  });

  // 处理地址选择
  const handleAddressSelect = (selection: AddressSelection | null) => {
    if (selection) {
      setFormData({
        ...formData,
        code: selection.code,
        info: selection.displayText
      });
    }
  };

  // 处理输入框变化
  const handleInputChange = (field: keyof AddressFormData) => (value: string) => {
    setFormData({
      ...formData,
      [field]: value
    });
  };

  // 处理表单提交
  const handleSubmit = () => {
    // 使用 zod 校验表单数据
    const result = addressFormSchema.safeParse(formData);

    if (!result.success) {
      const firstError = result.error.errors[0];
      error(firstError.message);
      return;
    }

    const validatedData = result.data;

    if (editingAddress) {
      editAddressMutation.mutate({
        address_id: editingAddress.id,
        name: validatedData.name.trim(),
        mobile: validatedData.mobile.trim(),
        code: validatedData.code,
        info: validatedData.info,
        detail: validatedData.detail.trim()
      });
    } else {
      addAddressMutation.mutate({
        name: validatedData.name.trim(),
        mobile: validatedData.mobile.trim(),
        code: validatedData.code,
        info: validatedData.info,
        detail: validatedData.detail.trim()
      });
    }
  };

  const isSubmitting = addAddressMutation.isPending || editAddressMutation.isPending;

  return (
    <Drawer open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DrawerContent className={cn("bg-background text-foreground border-border ")}>
        <DrawerHeader>
          <DrawerTitle className={cn("text-foreground")}>
            {editingAddress ? '编辑地址' : '添加地址'}
          </DrawerTitle>
          <DrawerDescription className={cn("text-muted-foreground")}>
            {editingAddress ? '修改地址信息' : '填写收货地址信息'}
          </DrawerDescription>
        </DrawerHeader>

        <div className="space-y-6 mt-6 overflow-y-auto flex-1">
          <div className="space-y-2">
            <Label htmlFor="name" className={cn("text-foreground")}>收件人姓名 *</Label>
            <Input
              id="name"
              placeholder="请输入收件人姓名"
              value={formData.name}
              onChange={(e) => handleInputChange('name')(e.target.value)}
              disabled={isSubmitting}
              className={cn("bg-background text-foreground border-border placeholder:text-muted-foreground")}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="mobile" className={cn("text-foreground")}>手机号码 *</Label>
            <AutocompleteInput
              id="mobile"
              placeholder="请输入手机号码"
              value={formData.mobile}
              onChange={handleInputChange('mobile')}
              options={mobileOptions}
              loading={mobileLoading}
              disabled={isSubmitting}
              className={cn("bg-background text-foreground border-border placeholder:text-muted-foreground")}
            />
          </div>

          <div className="space-y-2">
            <Label className={cn("text-foreground")}>所在地区 *</Label>
            <AddressSelector
              key={`${isOpen}-${editingAddress?.id || 'new'}`}
              value={formData.code}
              onChange={handleAddressSelect}
              placeholder="请选择省市区"
              disabled={isSubmitting}
              className={cn("bg-background border-border")}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="detail" className={cn("text-foreground")}>详细地址 *</Label>
            <Input
              id="detail"
              placeholder="请输入详细地址"
              value={formData.detail}
              onChange={(e) => handleInputChange('detail')(e.target.value)}
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
              {editingAddress ? '保存修改' : '添加地址'}
            </LoadingButton>
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  );
};
