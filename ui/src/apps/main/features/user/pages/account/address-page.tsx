import {
  addressDelete,
  addressListData,
} from '@shared/apis/user/profile';
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog';
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { PageSkeletonTable } from '@shared/components/custom/page-placeholder/skeleton-table';
import { Button } from '@shared/components/ui/button';
import { useToast } from '@shared/contexts/toast-context';
import { cn } from '@shared/lib/utils';
import { formatServerError, formatTime, TIME_STYLE } from '@shared/lib/utils/format-utils';
import { getQueryResponseData } from '@shared/lib/utils';
import { Route } from '@apps/main/routes/_main/user/account/address';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Clock, Edit, MapPin, Phone, Plus, Trash2, User } from 'lucide-react';
import React, { useMemo } from 'react';
import { AddressFormDrawer } from './address-form-drawer';
import { type AddressData } from './address-schema';

export const ProfileAddressPage: React.FC = () => {
  const navigate = Route.useNavigate();
  const search = Route.useSearch();
  const { error } = useToast();
  const queryClient = useQueryClient();

  // 获取地址列表
  const { data: addressData, isLoading, error: queryError } = useQuery({
    queryKey: ['addressList'],
    queryFn: () => addressListData()
  });
  const addressList = getQueryResponseData<AddressData[]>(addressData, []);

  // 路由状态
  const isEditing = search.action === 'edit' && !!search.id;
  const isSheetOpen = search.action === 'add' || search.action === 'edit';

  // 正在编辑的地址
  const editingAddress = useMemo(() => {
    if (!isEditing || addressList.length === 0) return null;
    return addressList.find((addr: AddressData) => addr.id === search.id) || null;
  }, [isEditing, search.id, addressList]);

  // 删除地址
  const deleteAddressMutation = useMutation({
    mutationFn: (addressId: number) => addressDelete({ address_id: addressId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['addressList'] });
    },
    onError: (err: any) => {
      const errorMessage = formatServerError(err);
      error(errorMessage);
    },
  });

  // 打开添加地址抽屉
  const openAddSheet = () => {
    navigate({ search: { action: 'add' } });
  };

  // 打开编辑地址抽屉
  const openEditSheet = (address: AddressData) => {
    navigate({ search: { action: 'edit', id: address.id } });
  };

  // 关闭抽屉
  const closeSheet = () => {
    navigate({ search: {} });
  };

  // 表单提交成功后回调
  const handleSuccess = () => {
    queryClient.invalidateQueries({ queryKey: ['addressList'] });
  };

  // 处理删除地址
  const handleDeleteAddress = (addressId: number) => {
    deleteAddressMutation.mutate(addressId);
  };

  return (
    <div className="px-0 py-6 space-y-6">
      {/* 页面标题和添加按钮 */}
      <div className="flex items-center justify-between px-4 md:px-6">
        <h1 className="text-lg md:text-2xl font-semibold text-foreground">地址管理</h1>
        <Button onClick={openAddSheet} size="sm" variant="outline">
          <Plus className="w-4 h-4 mr-2" />
          添加地址
        </Button>
      </div>

      {/* 地址列表内容 */}
      {isLoading ? (
        <PageSkeletonTable variant="content" rows={3} className={cn('m-4 md:px-6')} />
      ) : queryError ? (
        <CenteredError
          variant="card"
          error={queryError}
          onReset={() => queryClient.refetchQueries({ queryKey: ['addressList'] })}
          className={cn('m-4  md:m-6')}
        />
      ) : addressList.length > 0 ? (
        <div className={cn("px-4 md:px-6", addressList.length === 1 ? "space-y-4" : "grid grid-cols-1 2xl:grid-cols-2 gap-4")}>
          {addressList.map((address: AddressData) => (
            <div key={address.id} className="border rounded-lg p-4 hover:shadow-md transition-shadow border-border">
              <div className="flex flex-col md:flex-row md:justify-between md:items-start">
                <div className="flex-1 space-y-2">
                  <div className="flex items-center gap-2">
                    <User className="w-4 h-4 text-muted-foreground" />
                    <span className="font-medium text-foreground">{address.name}</span>
                    <Phone className="w-4 h-4 text-muted-foreground ml-4" />
                    <span className="text-muted-foreground">{address.mobile}</span>
                  </div>
                  <div className="flex items-start gap-2">
                    <MapPin className="w-4 h-4 text-muted-foreground mt-0.5" />
                    <div className="flex-1">
                      <p className="text-sm text-muted-foreground">{address.address_info}</p>
                      <p className="text-sm text-muted-foreground">{address.address_detail}</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-1 ml-6">
                    <Clock className="w-3 h-3 text-muted-foreground" />
                    <span className="text-xs text-muted-foreground">
                      {formatTime(address.change_time, TIME_STYLE.RELATIVE_TEXT)}
                    </span>
                  </div>
                </div>
                <div className="flex items-center justify-end gap-2 mt-4 md:mt-0 md:ml-4">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => openEditSheet(address)}
                    className={cn("border-border text-foreground hover:bg-muted")}
                  >
                    <Edit className="w-4 h-4 mr-2" />
                    编辑
                  </Button>
                  <ConfirmDialog
                    title="确认删除地址"
                    description={`您确定要删除收件人“${address.name}”的地址吗？此操作不可撤销。`}
                    onConfirm={() => handleDeleteAddress(address.id)}
                  >
                    <Button variant="outline" size="sm" className={cn("border-border text-foreground hover:bg-muted")}>
                      <Trash2 className="w-4 h-4 mr-2" />
                      删除
                    </Button>
                  </ConfirmDialog>
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="text-center py-8 px-4 md:px-6">
          <MapPin className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
          <p className="text-muted-foreground mb-4">暂无地址信息</p>
          <Button onClick={openAddSheet} variant="outline">
            <Plus className="w-4 h-4 mr-2" />
            添加地址
          </Button>
        </div>
      )}

      <AddressFormDrawer
        isOpen={isSheetOpen}
        onClose={closeSheet}
        editingAddress={editingAddress}
        onSuccess={handleSuccess}
      />
    </div>
  );
};
