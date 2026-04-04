import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import {
    userFileTagAdd,
    userFileTagRemove,
    userFileTags,
    type UserFileItemType,
    type UserFileTagItemType,
} from "@shared/apis/user/file";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { Input } from "@shared/components/ui/input";
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from "@shared/components/ui/tooltip";
import { useToast } from "@shared/contexts/toast-context";
import { cn, formatTime, TIME_STYLE } from "@shared/lib/utils";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Loader2, Plus, X } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";

interface FileTagsDrawerProps {
    file: UserFileItemType;
    isOpen: boolean;
    onOpenChange: (open: boolean) => void;
    /** 标签变更后的回调（用于刷新列表） */
    onTagsChanged?: () => void;
}

export function FileTagsDrawer({
    file,
    isOpen,
    onOpenChange,
    onTagsChanged,
}: FileTagsDrawerProps) {
    const { success: showSuccess, error: showError } = useToast();
    const [newTagName, setNewTagName] = useState("");

    // 当文件切换时重置输入
    useEffect(() => {
        setNewTagName("");
    }, [file.id]);

    // 获取标签列表
    const {
        data: tagsData,
        isLoading,
        isError,
        error,
        refetch,
    } = useQuery({
        queryKey: ["userFileTags", file.id],
        queryFn: ({ signal }) =>
            userFileTags(
                { id: file.id },
                { signal }
            ),
        enabled: isOpen,
    });

    const tags: UserFileTagItemType[] = useMemo(
        () => tagsData?.response?.data ?? [],
        [tagsData]
    );

    // 添加标签
    const addTagMutation = useMutation({
        mutationFn: (tagName: string) =>
            userFileTagAdd({
                id: file.id,
                tag_name: tagName,
            }),
        onSuccess: () => {
            showSuccess("标签已添加");
            setNewTagName("");
            refetch();
            onTagsChanged?.();
        },
        onError: (err: any) => {
            showError(
                err?.data?.message || err?.message || "添加标签失败"
            );
        },
    });

    // 移除标签
    const removeTagMutation = useMutation({
        mutationFn: (tagName: string) =>
            userFileTagRemove({
                id: file.id,
                tag_name: tagName,
            }),
        onSuccess: () => {
            showSuccess("标签已移除");
            refetch();
            onTagsChanged?.();
        },
        onError: (err: any) => {
            showError(
                err?.data?.message || err?.message || "移除标签失败"
            );
        },
    });

    const handleAddTag = useCallback(() => {
        const trimmed = newTagName.trim();
        if (!trimmed) return;
        // 检查是否已存在
        if (tags.some((t) => t.tag_name.toLowerCase() === trimmed.toLowerCase())) {
            showError("标签已存在");
            return;
        }
        addTagMutation.mutate(trimmed);
    }, [newTagName, tags, addTagMutation, showError]);

    const handleKeyDown = useCallback(
        (e: React.KeyboardEvent<HTMLInputElement>) => {
            if (e.key === "Enter") {
                e.preventDefault();
                handleAddTag();
            }
        },
        [handleAddTag]
    );

    const handleRemoveTag = useCallback(
        (tagName: string) => {
            removeTagMutation.mutate(tagName);
        },
        [removeTagMutation]
    );

    const handleOpenChange = (open: boolean) => {
        onOpenChange(open);
        if (!open) {
            setNewTagName("");
        }
    };

    return (
        <Drawer open={isOpen} onOpenChange={handleOpenChange}>
            <DrawerContent>
                <DrawerHeader className={cn("pb-4")}>
                    <DrawerTitle>文件标签管理</DrawerTitle>
                    <DrawerDescription className={cn("space-y-1")}>
                        <div className="flex items-center gap-1.5">
                            <span>文件名:</span>
                            <span className="font-medium truncate max-w-[280px]" title={file.file_name}>
                                {file.file_name || "-"}
                            </span>
                        </div>
                        <div className="text-xs text-muted-foreground">
                            文件ID: {file.id}
                        </div>
                    </DrawerDescription>
                </DrawerHeader>

                {/* 添加标签区域 */}
                <div className="px-0 pb-4 border-b">
                    <div className="flex items-center gap-2">
                        <Input
                            value={newTagName}
                            onChange={(e) => setNewTagName(e.target.value)}
                            onKeyDown={handleKeyDown}
                            placeholder="输入标签名称..."
                            className="flex-1 h-8 text-sm"
                            disabled={addTagMutation.isPending}
                            maxLength={64}
                        />
                        <Button
                            size="sm"
                            variant="default"
                            className="h-8 px-3"
                            onClick={handleAddTag}
                            disabled={
                                !newTagName.trim() || addTagMutation.isPending
                            }
                        >
                            {addTagMutation.isPending ? (
                                <Loader2 className="h-3.5 w-3.5 animate-spin" />
                            ) : (
                                <Plus className="h-3.5 w-3.5" />
                            )}
                            <span className="ml-1">添加</span>
                        </Button>
                    </div>
                </div>

                {/* 标签列表区域 */}
                <div className="pt-4" data-vaul-no-drag>
                    {isLoading && (
                        <CenteredLoading className="py-8" />
                    )}

                    {isError && (
                        <CenteredError
                            variant="content"
                            error={error}
                            onReset={() => refetch()}
                        />
                    )}

                    {!isLoading && !isError && tags.length === 0 && (
                        <div className="text-center text-sm text-muted-foreground py-8">
                            暂无标签，请添加标签
                        </div>
                    )}

                    {!isLoading && !isError && tags.length > 0 && (
                        <div className="flex flex-wrap gap-2">
                            {tags.map((tag) => (
                                <TooltipProvider
                                    key={tag.id}
                                    delayDuration={300}
                                >
                                    <Tooltip>
                                        <TooltipTrigger asChild>
                                            <Badge
                                                variant="secondary"
                                                className={cn(
                                                    "text-sm px-2.5 py-1 gap-1.5 cursor-default",
                                                    "hover:bg-secondary/60 transition-colors",
                                                    "group"
                                                )}
                                            >
                                                <span className="select-all cursor-text">
                                                    {tag.tag_name}
                                                </span>
                                                <button
                                                    type="button"
                                                    className={cn(
                                                        "inline-flex items-center justify-center",
                                                        "h-4 w-4 rounded-full flex-shrink-0",
                                                        "text-muted-foreground/60 hover:text-destructive hover:bg-destructive/10",
                                                        "transition-colors",
                                                        removeTagMutation.isPending
                                                            ? "opacity-50 pointer-events-none"
                                                            : ""
                                                    )}
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        handleRemoveTag(tag.tag_name);
                                                    }}
                                                    title="移除标签"
                                                >
                                                    <X className="h-3 w-3" />
                                                </button>
                                            </Badge>
                                        </TooltipTrigger>
                                        <TooltipContent side="bottom">
                                            <span className="text-xs">
                                                添加时间:{" "}
                                                {tag.add_time
                                                    ? formatTime(
                                                        tag.add_time,
                                                        TIME_STYLE.ABSOLUTE_TEXT
                                                    )
                                                    : "-"}
                                            </span>
                                        </TooltipContent>
                                    </Tooltip>
                                </TooltipProvider>
                            ))}
                        </div>
                    )}

                    {/* 标签统计 */}
                    {!isLoading && !isError && tags.length > 0 && (
                        <div className="mt-4 pt-3 border-t text-xs text-muted-foreground">
                            共 {tags.length} 个标签
                        </div>
                    )}
                </div>
            </DrawerContent>
        </Drawer>
    );
}
