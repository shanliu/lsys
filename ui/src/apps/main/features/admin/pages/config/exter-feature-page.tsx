import {
    getExterFeatureList,
    delExterFeature,
    type ExterFeatureItemType,
} from '@shared/apis/admin/config'
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { Button } from '@shared/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@shared/components/ui/card'
import { useToast } from '@shared/contexts/toast-context'
import { ConfigNavContainer } from '@apps/main/features/admin/components/ui/config-nav'
import { cn, formatServerError, getQueryResponseData } from '@shared/lib/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Edit, Plus, Puzzle, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { configModuleConfig } from '../nav-info'
import { ExterFeatureDrawer } from './exter-feature-drawer'

export function ExterFeaturePage() {
    const toast = useToast()
    const queryClient = useQueryClient()

    // Drawer 状态
    const [drawerOpen, setDrawerOpen] = useState(false)
    const [editingItem, setEditingItem] = useState<ExterFeatureItemType | undefined>()

    // 获取列表
    const {
        data: listData,
        isLoading,
        isError,
        error,
    } = useQuery({
        queryKey: ['exter-feature-list'],
        queryFn: async ({ signal }) => {
            const result = await getExterFeatureList({ page: { page: 1, limit: 100 } }, { signal })
            return result
        },
    })

    const list = getQueryResponseData<ExterFeatureItemType[]>(listData, [])

    // 删除
    const deleteMutation = useMutation({
        mutationFn: (id: number) => delExterFeature({ id }),
        onSuccess: () => {
            toast.success('删除成功')
            queryClient.invalidateQueries({ queryKey: ['exter-feature-list'] })
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        },
    })

    // 打开新增抽屉
    const handleAdd = () => {
        setEditingItem(undefined)
        setDrawerOpen(true)
    }

    // 打开编辑抽屉
    const handleEdit = (item: ExterFeatureItemType) => {
        setEditingItem(item)
        setDrawerOpen(true)
    }

    // 刷新数据
    const refreshData = () => {
        queryClient.refetchQueries({ queryKey: ['exter-feature-list'] })
    }

    if (isLoading) {
        return (
            <ConfigNavContainer {...configModuleConfig}>
                <CenteredLoading variant="content" />
            </ConfigNavContainer>
        )
    }

    if (isError) {
        return (
            <ConfigNavContainer {...configModuleConfig}>
                <CenteredError variant="content" error={error} onReset={refreshData} />
            </ConfigNavContainer>
        )
    }

    return (
        <ConfigNavContainer {...configModuleConfig}>
            <div className="space-y-6">
                <Card>
                    <CardHeader>
                        <div className="flex items-center justify-between">
                            <div>
                                <CardTitle className={cn('flex items-center gap-2')}>
                                    <Puzzle className="h-5 w-5" />
                                    外部扩展能力
                                </CardTitle>
                                <CardDescription>管理系统中的外部扩展能力定义（如短信、邮件等）</CardDescription>
                            </div>
                            <Button variant="outline" size="sm" onClick={handleAdd}>
                                <Plus className={cn('mr-2 h-4 w-4')} />
                                新增
                            </Button>
                        </div>
                    </CardHeader>
                    <CardContent>
                        {list.length === 0 ? (
                            <div className="text-center py-8 text-muted-foreground">
                                暂无扩展能力，点击右上角"新增"按钮添加
                            </div>
                        ) : (
                            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                                {list.map((item) => (
                                    <div
                                        key={item.id}
                                        className="group relative flex items-center justify-between rounded-lg border bg-card p-4 transition-colors hover:bg-accent/50"
                                    >
                                        <div className="min-w-0 flex-1">
                                            <div className="font-medium text-foreground">{item.title}</div>
                                            <code className="text-xs text-muted-foreground bg-muted px-1.5 py-0.5 rounded">
                                                {item.key}
                                            </code>
                                        </div>
                                        <div className="flex items-center gap-1 ml-2">
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                className="h-8 w-8"
                                                onClick={() => handleEdit(item)}
                                            >
                                                <Edit className="h-4 w-4" />
                                            </Button>
                                            <ConfirmDialog
                                                title="确认删除"
                                                description={`确定要删除扩展能力 "${item.title}" 吗？此操作不可撤销。`}
                                                onConfirm={async () => {
                                                    await deleteMutation.mutateAsync(item.id)
                                                }}
                                            >
                                                <Button
                                                    variant="ghost"
                                                    size="icon"
                                                    className="h-8 w-8 text-destructive hover:text-destructive"
                                                >
                                                    <Trash2 className="h-4 w-4" />
                                                </Button>
                                            </ConfirmDialog>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </CardContent>
                </Card>

                {/* 新增/编辑抽屉 */}
                <ExterFeatureDrawer
                    feature={editingItem}
                    open={drawerOpen}
                    onOpenChange={setDrawerOpen}
                />
            </div>
        </ConfigNavContainer>
    )
}
