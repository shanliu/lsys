import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import DeleteIcon from '@mui/icons-material/Delete';
import EditIcon from '@mui/icons-material/Edit';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Autocomplete, Box, Button, Divider, Drawer, FormControl, FormLabel, Grid, IconButton, InputLabel, List, ListItem, MenuItem, Paper, Select, Skeleton, TableBody, TableCell, TableRow, TextField, Typography } from '@mui/material';
import Table from '@mui/material/Table';
import { Stack } from '@mui/system';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { ToastContext } from '../../../../common/context/toast';
import { ConfirmButton } from '../../../../common/ui/dialog';
import { ClearTextField, InputTagSelect, TagSelect } from '../../../../library/input';
import { LoadingButton, Progress } from '../../../../library/loading';
import { BaseTableBodyRow, BaseTableFooter, BaseTableHead, BaseTableNoRows } from '../../../../library/table_page';
import { resAdd, resAll, resDelete, resEdit, resListData, resTags } from '../../../../common/rest/access';
import { useSearchChange } from '../../../../common/utils/hook';
import { showTime } from '../../../../common/utils/utils';
import { ResOpItem, UserSearchInput, UserTags } from '../../common/user';
import { ResEditBox } from './res_edit_box';
//页面提交

export function AddBox(props) {
    const { tags, res_type, res, onSave } = props;
    let tags_options = (tags ?? []).map((e) => { return e.name })

    let init_res_data = {
        res_type: res_type[0].key,
        loading: false,
        user_id: 0,
        res_name: '',
        res_key: '',
        res_key_open: false,
        tags: [],
        op_items: [],
        op_focused: false,
        res_focused: false
    };
    const [resData, setResData] = useState(init_res_data);
    useEffect(() => {
        if (!res || !res.res || res.res.id <= 0) return;
        setResData({
            ...resData,
            res_type: res.res.user_id > 0 ? 2 : 1,
            user_id: res.res.user_id,
            res_name: res.res.name,
            res_key: res.res.res_key,
            tags: res.tags.map((e) => { return e.name }),
            op_items: res.ops.map((e) => {
                return {
                    op_name: e.name,
                    op_key: e.op_key,
                }
            }),
        })
    }, [props.item])
    const [res_keys, set_res_keys] = useState({
        keys: [],
    });
    useEffect(() => {
        resAll({
            global_res: resData.res_type == 1
        }).then((data) => {
            if (data.status) {
                set_res_keys({
                    keys: (data.data ?? []),
                })
            }
        })
    }, [resData.res_type])
    const { toast } = useContext(ToastContext);


    let sysDataFill = (res_key) => {
        let set_data = {}
        let sres = res_keys.keys.find((k) => {
            return k.key == res_key
        })
        if (sres) {
            if (resData.res_key != res_key) {

                let add_ops = sres.ops.map((e) => {
                    return {
                        op_key: e.key,
                        op_name: e.name,
                    }
                })
                set_data = {
                    tags: [...sres.tags],
                    op_items: add_ops,
                }
            } else {
                let add_tag = sres.tags.filter((e) => {
                    let check = resData.tags.find((t) => { return t == e })
                    return !resData.tags || !check
                })
                let add_ops = sres.ops.filter((e) => {
                    return !resData.op_items || !resData.op_items.find((t) => { return t.op_key == e.key })
                }).map((e) => {
                    return {
                        op_key: e.key,
                        op_name: e.name,
                    }
                })
                set_data = {
                    tags: [...resData.tags, ...add_tag],
                    op_items: [...resData.op_items, ...add_ops],
                }
            }
        }
        set_data.res_name = sres.name;
        return set_data
    }

    return (
        <Fragment>
            <Typography
                align="center"
                variant="subtitle1"
                noWrap
                sx={{
                    mt: 4,
                    mb: 2,
                    fontWeight: 100,
                    alignItems: "center",
                    letterSpacing: '.3rem',
                    color: 'inherit',
                    textDecoration: 'none',
                }}
            >
                {res && res.res && res.res.id ? "编辑资源" : "创建资源"}
            </Typography>
            <Divider variant="middle" />
            <Form method="post" >
                <Grid
                    sx={{
                        mt: 3,
                    }}
                    container
                    justifyContent="center"
                    alignItems="center"
                >
                    <Grid item xs={10}>
                        <FormControl fullWidth sx={{ mb: 2 }} >
                            <InputLabel size="small" id="res-add-select-label">选择资源</InputLabel>
                            <Select
                                onChange={(e) => {
                                    if (res && res.res && res.res.id > 0) {
                                        return
                                    }

                                    setResData({
                                        ...resData,
                                        user_id: 0,
                                        res_type: e.target.value,
                                        tags: [],
                                        op_items: [],
                                        res_key: ""
                                    })


                                }}
                                value={resData.res_type}
                                disabled={resData.loading || !!res}
                                size="small"
                                labelId="res-add-select-label"
                                id="res-add-select"
                                label="选择资源"
                            >
                                {res_type.map((item) => { return <MenuItem key={`res-type-${item.key}`} value={item.key}>{item.title}</MenuItem> })}
                            </Select>
                        </FormControl>
                        {
                            resData.res_type == 2 ?
                                <UserSearchInput
                                    onSelect={(nval) => {

                                        let value = (nval + '').replace(/[^0-9]+/, '');
                                        setResData({
                                            ...resData,
                                            user_id: value
                                        })
                                    }}
                                    sx={{ width: 1, mb: 2 }}
                                    variant="outlined"
                                    label={`选择用户`}
                                    value={resData.user_id > 0 ? resData.user_id : ''}
                                    type="text"
                                    name="code"

                                    size="small"
                                    disabled={resData.loading || !!res}
                                    enableUser={true}
                                /> : null
                        }
                        <TextField
                            label="名称"
                            variant="outlined"
                            name="name"
                            size="small"
                            disabled={resData.loading}
                            value={resData.res_name}
                            onChange={(e) => {
                                setResData({
                                    ...resData,
                                    res_name: e.target.value,
                                })
                            }}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                        />
                        <Autocomplete
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            onChange={(e) => {
                                setResData({
                                    ...resData,
                                    res_key_open: false,
                                })
                            }}
                            disabled={resData.loading}
                            value={resData.res_key}
                            options={res_keys.keys.map((e) => e.key)}
                            open={resData.res_key_open}
                            getOptionLabel={(option) => option}
                            noOptionsText={"回车确认使用该值"}
                            renderInput={(params) => (
                                <TextField
                                    {...params}
                                    variant="outlined"
                                    label="标识符"
                                    size="small"
                                    placeholder="输入标识符"
                                    focused={resData.res_focused}
                                    onClick={(e) => {
                                        setResData({
                                            ...resData,
                                            res_key_open: true,
                                        })
                                    }}
                                    onFocus={(e) => {
                                        setResData({
                                            ...resData,
                                            res_key_open: true,
                                        })
                                    }}
                                    onBlur={(e) => {
                                        setResData({
                                            ...resData,
                                            res_focused: false,
                                            res_key: res ? resData.res_key : e.target.value,
                                            res_key_open: false,
                                            ...sysDataFill(e.target.value)
                                        })
                                    }}
                                    onKeyUp={(e) => {
                                        if (!res_keys.keys.map((item) => {
                                            return item.key == e.target.value
                                        })) {
                                            e.stopPropagation();
                                            e.preventDefault();
                                        }
                                    }}
                                    onKeyDown={(e) => {
                                        if (e.key != 'Enter')
                                            return
                                        setResData({
                                            ...resData,
                                            res_key: res ? resData.res_key : e.target.value,
                                            res_key_open: false,
                                            ...sysDataFill(e.target.value)
                                        })
                                        e.stopPropagation();
                                        e.preventDefault();
                                    }}
                                />
                            )}
                        />


                        <InputTagSelect
                            name="tag"
                            size="small"
                            disabled={resData.loading}
                            options={tags_options}
                            value={resData.tags}
                            onChange={(value) => {
                                setResData({
                                    ...resData,
                                    tags: value
                                })
                            }}
                        />
                        <ResEditBox
                            focused={resData.op_focused}
                            opItems={resData.op_items}
                            sx={{
                                mt: 1,
                                p: 1,
                                mb: 2,
                            }} onChange={(data) => {
                                let sdata = data ?? [];
                                setResData({
                                    ...resData,
                                    op_items: sdata,
                                    op_focused: sdata.length == 0
                                })
                            }} />
                    </Grid>
                    <Grid item xs={10}>
                        <LoadingButton sx={{
                            width: 1,
                            mb: 3
                        }} variant="contained"
                            loading={resData.loading}
                            disabled={resData.loading}
                            onClick={() => {
                                if (!resData.op_items || resData.op_items.length == 0) {
                                    setResData({
                                        ...resData,
                                        op_focused: true
                                    })
                                    return
                                }
                                let res_r = /\{.*\}/.exec(resData.res_key);
                                if (res_r && res_r.length > 0) {
                                    toast("请先替换资源变量:" + res_r[0])
                                    setResData({
                                        ...resData,
                                        res_focused: true
                                    })
                                    return
                                }
                                setResData({
                                    ...resData,
                                    loading: true
                                })
                                if (res && res.res && res.res.id > 0) {
                                    resEdit({
                                        res_id: res.res.id,
                                        name: resData.res_name,
                                        ops: resData.op_items.map((e) => {
                                            return { name: e.op_name, key: e.op_key }
                                        }),
                                        tags: resData.tags
                                    }).then((data) => {
                                        setResData({
                                            ...resData,
                                            loading: false
                                        })
                                        if (!data.status) {
                                            toast(data.message)
                                        } else {
                                            onSave(res.res.user_id, res.res.id)
                                            toast("保存完成");
                                        }
                                    })
                                } else {
                                    resAdd({
                                        user_id: resData.user_id,
                                        key: resData.res_key,
                                        name: resData.res_name,
                                        ops: resData.op_items.map((e) => {
                                            return { name: e.op_name, key: e.op_key }
                                        }),
                                        tags: resData.tags
                                    }).then((data) => {
                                        if (!data.status) {
                                            toast(data.message)
                                            setResData({
                                                ...resData,
                                                loading: false
                                            })
                                        } else {
                                            toast("添加完成")
                                            onSave(resData.user_id, data.id)
                                            setResData(init_res_data)
                                        }
                                    })
                                }

                            }}
                        >{res && res.res && res.res.id ? "保存" : "添加"}</LoadingButton>
                    </Grid>
                </Grid>
            </Form ></Fragment >)
}


function UserAccessResItem(props) {
    const { item, columns, onTagClick } = props;

    let isTags = (item.tags ?? []).length > 0;

    let isOps = (item.ops ?? []).length > 0;

    return <ListItem key={`res-${item.res.id}`}
        sx={{
            borderBottom: "1px solid #ddd",
            "&:hover": {
                background: "#f9f9f9"
            }
        }} disablePadding>
        <Grid
            container
            direction="column"
            justifyContent="center"
            alignItems="center">
            <Table stickyHeader sx={{ width: 1 }}>
                <TableBody>
                    <BaseTableBodyRow
                        row={item.res ? item.res : {}}
                        columns={columns}
                        cellProps={{ style: { borderBottom: "1px solid #f0f0f0" } }}
                    />
                    {isOps ? <TableRow>
                        <TableCell colSpan={6} style={{ padding: 4, borderBottom: "1px solid #f0f0f0" }}>
                            <Grid container>
                                <Grid sx={{ width: 80, lineHeight: "52px", fontWeight: 500, fontSize: "0.875rem", textAlign: "right", mr: 1 }}>可用操作</Grid>
                                <Grid sx={{ flex: 1, paddingBottom: 1 }}>
                                    {
                                        item.ops.map((op) => {
                                            return <ResOpItem
                                                style={{ margin: "8px 8px 0px 0px" }}
                                                key={`op-${item.res.id}-${op.id}`}
                                                name={op.name}
                                                opKey={op.op_key}
                                                tips={`最后修改时间:${showTime(op.change_time, "未知")}`} />
                                        })
                                    }
                                </Grid>
                            </Grid>
                        </TableCell>
                    </TableRow> : null}
                    {isTags ? <TableRow>
                        <TableCell colSpan={6} style={{ padding: 4, borderBottom: "1px solid #f0f0f0" }}>
                            <Grid container >
                                <Grid sx={{ width: 80, lineHeight: "52px", fontWeight: 500, fontSize: "0.875rem", textAlign: "right", mr: 1 }}>标记</Grid>
                                <Grid sx={{ flex: 1, color: "#333", paddingTop: "2px" }}>
                                    {
                                        item.tags.map((tag) => {
                                            return <UserTags
                                                onClick={() => {
                                                    onTagClick(tag.name)
                                                }}
                                                name={tag.name}
                                                key={`res-${item.res.id}-${tag.id}`}
                                                sx={{ m: 1, ml: 0 }}
                                                tips={`添加于:${showTime(tag.change_time, "未知")}`}
                                            />
                                        })
                                    }
                                </Grid>
                            </Grid>
                        </TableCell>
                    </TableRow > : null}
                </TableBody>
            </Table>
        </Grid>
    </ListItem>;
}

export default function SystemAccessResPage(props) {
    //URL参数
    const [searchParam, setSearchParam] = useSearchChange({
        user_id: "0",
        tag: "",
        res_id: "",
        res_name: "",
        page: 0,
        page_size: 25,
    });




    //过滤组件数据
    const [filterData, setfilterData] = useState({
        tag: searchParam.get("tag"),
        res_id: searchParam.get("res_id"),
        res_name: searchParam.get("res_name")
    })

    useEffect(() => {
        setfilterData({
            ...filterData,
            tag: searchParam.get("tag"),
            res_id: searchParam.get("res_id"),
            res_name: searchParam.get("res_name")
        })
        setPageTagData({
            ...pageTagData,
            init: parseInt(searchParam.get("user_id") ?? '0') == parseInt(pageTagData.user_id ?? '0'),
            user_id: searchParam.get("user_id")
        })
    }, [searchParam])
    //左边弹层数据
    const [boxPage, setBoxPage] = useState({
        show: false,
        box: null,
        item: null
    });
    //初始化数据
    const [pageRowData, setPageRowData] = useState({
        rows: [],
        rows_total: 0,
        rows_error: null,
        rows_loading: true,
    })
    const [pageTagData, setPageTagData] = useState({
        init: false,
        user_id: 0,
        // user_focused: false,
        load_user_id: false,
        res_type: 1,
        tag_rows: [],
        tag_rows_error: null,
        tag_rows_loading: true,
    })
    //页面数据初始化
    const loadResData = () => {
        let set_data = { ...pageTagData }
        if (!pageTagData.init) {
            let user_id = searchParam.get("user_id");
            let res_type = user_id > 0 ? 2 : 1;
            set_data = {
                ...set_data,
                init: true,
                tag_rows_loading: true,
                user_id: user_id,
                res_type: res_type,
            }
            setPageTagData(set_data)
        }

        let loadRow = () => {
            setPageRowData({
                ...pageRowData,
                rows_loading: true
            })
            window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
            resListData({
                tags: true,
                ops: true,
                count_num: true,
                user_id: searchParam.get("user_id") ?? 0,
                tag: searchParam.get("tag"),
                res_id: searchParam.get("res_id"),
                res_name: searchParam.get("res_name"),
                page: searchParam.get("page"),
                page_size: searchParam.get("page_size")
            }).then((data) => {

                if (!data.status) {
                    setPageRowData({
                        ...pageRowData,
                        rows_error: data.message,
                        rows_loading: false
                    })
                    return;
                }
                setPageRowData({
                    ...pageRowData,
                    rows: data.data ?? [],
                    rows_total: data.total ?? 0,
                    rows_loading: false
                })
            })
        };
        let user_id = searchParam.get("user_id");
        if (user_id == null) user_id = 0
        else user_id = parseInt(user_id);
        if (isNaN(user_id)) return;
        if (user_id === pageTagData.load_user_id) {
            loadRow();
        } else {
            resTags({
                user_id: parseInt(user_id)
            }).then((data) => {
                if (!data.status) {
                    setPageTagData({
                        ...set_data,
                        tag_rows_error: data.message,
                        tag_rows_loading: false
                    })
                    return;
                }
                if (pageTagData.load_user_id !== false) {
                    setfilterData({
                        tag: '',
                        res_id: '',
                        res_name: ''
                    })
                }
                setPageTagData({
                    ...set_data,
                    load_user_id: user_id,
                    tag_rows: data.data ?? [],
                    tag_rows_loading: false
                })
                loadRow();
            })
        }

    };
    useEffect(loadResData, [searchParam])



    const res_type = [
        { key: 1, title: "系统资源" },
        { key: 2, title: "指定用户" },
    ]

    //渲染页面
    let boxData;
    switch (boxPage.box) {
        case "add":
            boxData = <AddBox
                tags={pageTagData.tag_rows}
                res_type={res_type}
                onSave={(user_id, id) => {
                    setSearchParam({
                        tag: "",
                        user_id: user_id,
                        res_id: id,
                        res_name: "",
                        page: 0
                    }, loadResData)
                }} />
            break;
        case "edit":
            boxData = <AddBox
                tags={pageTagData.tag_rows}
                res_type={res_type}
                res={boxPage.item}
                onSave={(user_id, id) => {
                    setSearchParam({
                        tag: "",
                        user_id: user_id,
                        res_id: id,
                        res_name: "",
                        page: 0
                    }, loadResData)
                }} />
            break;
    }
    let page_error;
    let tag_show = !pageTagData.tag_rows_loading;
    if (pageRowData.rows_error && pageRowData.rows_error != '') {
        page_error = pageRowData.rows_error;
    }
    if (pageTagData.tag_rows_error && pageTagData.tag_rows_error != '') {
        page_error = pageTagData.tag_rows_error;
        tag_show = false;
    }

    const delRes = (res_id) => {
        res_id = parseInt(res_id)
        if (isNaN(res_id) || res_id <= 0) return;
        let param = {
            res_id: res_id
        };
        return resDelete(param);
    }
    const { toast } = useContext(ToastContext);
    const columns = [
        {
            field: 'id',
            label: 'ID',
            align: "right",
            style: { width: 120, borderBottom: "1px solid #f3f3f3" }
        },
        {
            field: 'name',
            label: '名称',
            style: { width: 150, borderBottom: "1px solid #f3f3f3" }
        },
        {
            field: 'res_key',
            label: '标识符',
            style: { width: 150, borderBottom: "1px solid #f3f3f3" }
        },
        {
            field: 'change_time',
            label: '更新时间',
            style: { width: 140, borderBottom: "1px solid #f3f3f3" },
            render: (row) => {
                return showTime(row.change_time, "未知")
            }
        },
        {

            label: '资源用户ID',
            align: "center",
            style: { width: 120, borderBottom: "1px solid #f3f3f3" },
            render: (row) => {
                if (row.user_id == 0) {
                    return "系统用户"
                } else {
                    return row.user_id
                }
            }
        },
        {
            field: 'add_user_id',
            label: '添加用户ID',
            align: "center",
            style: { width: 120, borderBottom: "1px solid #f3f3f3" }
        },
        {
            style: { width: 120, borderBottom: "1px solid #f3f3f3", borderLeft: "1px solid #f3f3f3" },
            label: '操作',
            rowSpan: 3,
            align: "center",
            render: (row) => {
                return <Fragment>
                    <IconButton onClick={() => {
                        let item = pageRowData.rows.find((item) => {
                            return item.res.id == row.id
                        })
                        if (!item) return;
                        setBoxPage({
                            show: true,
                            box: "edit",
                            item: item
                        })
                    }}><EditIcon fontSize="small" /></IconButton>
                    <ConfirmButton
                        message={`确定要删除资源 [${row.name}] ?`}
                        onAction={() => {
                            return delRes(row.id).then((data) => {
                                if (!data.status) return data;
                                let rows = pageRowData.rows.filter((item) => {
                                    if (item.res.id != row.id) return item;
                                })
                                setPageRowData({
                                    ...pageRowData,
                                    rows: rows,
                                    rows_total: pageRowData.rows_total - 1,
                                })
                                if (rows.length == 0) {
                                    setSearchParam({
                                        tag: "",
                                        res_id: "",
                                        res_name: "",
                                        page: 0
                                    }, loadResData)
                                }
                                return data;
                            })
                        }}
                        renderButton={(props) => {
                            return <IconButton {...props} size='small'>
                                <DeleteIcon fontSize='small' />
                            </IconButton>
                        }} />
                </Fragment>
            }
        },
    ];


    return <Fragment>
        <Drawer
            sx={{ zIndex: (theme) => theme.zIndex.drawer + 3 }}
            anchor={"right"}
            open={boxPage.show}
            onClose={() => {
                setBoxPage({
                    item: null,
                    show: false
                })
            }}
        >
            <Box
                sx={{ width: 450 }}
            >
                {boxData}
            </Box>
        </Drawer>
        <Paper
            component="form"
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1, minWidth: 900 }}
        >
            <Stack direction="row" spacing={1} sx={{ mr: 2 }}>
                <FormControl >
                    <InputLabel size="small" id="res-select-label">资源类型</InputLabel>
                    <Select
                        disabled={pageTagData.tag_rows_loading}
                        size="small"
                        labelId="res-select-label"
                        id="res-select"
                        label="资源类型"
                        onChange={(e) => {
                            setPageTagData({
                                ...pageTagData,
                                res_type: e.target.value
                            })
                        }}
                        value={pageTagData.res_type}
                    >
                        {res_type.map((item) => { return <MenuItem key={`res-type-${item.key}`} value={item.key}>{item.title}</MenuItem> })}
                    </Select>
                </FormControl>
                {
                    pageTagData.res_type == 2 ?
                        <UserSearchInput
                            onSelect={(nval) => {
                                let value = (nval + '').replace(/[^0-9]+/, '');
                                setPageTagData({
                                    ...pageTagData,
                                    user_id: value
                                })
                            }}
                            sx={{ mr: 1, width: 150 }}
                            variant="outlined"
                            label={`选择用户`}
                            value={pageTagData.user_id > 0 ? pageTagData.user_id : ''}
                            type="text"
                            name="code"

                            size="small"
                            disabled={pageTagData.tag_rows_loading}
                            enableUser={true}
                        />
                        : null
                }
                <LoadingButton
                    variant="outlined"
                    size="small"
                    loading={pageTagData.tag_rows_loading}
                    disabled={pageTagData.tag_rows_loading}
                    startIcon={<SearchIcon />}
                    onClick={() => {
                        if (pageTagData.res_type == 2
                            && (pageTagData.user_id == 0 || pageTagData.user_id == '')) {
                            // setPageTagData({
                            //     ...pageTagData,
                            //     user_focused: true
                            // })
                            return;
                        }
                        setSearchParam({
                            user_id: pageTagData.res_type == 1 ? 0 : pageTagData.user_id,
                            tag: "",
                            res_id: "",
                            res_name: "",
                            page: 0,
                        }, loadResData)
                    }}
                    sx={{ pt: "4px", minWidth: 100 }}
                >查询</LoadingButton>
            </Stack >
            {
                (tag_show) ?
                    <Box>
                        {pageTagData.tag_rows.length > 0 ? <TagSelect
                            loading={pageTagData.tag_rows_loading}
                            rows={pageTagData.tag_rows}
                            onChange={(event) => {
                                setfilterData({
                                    ...filterData,
                                    tag: event.target.value
                                })
                            }}
                            value={filterData.tag}
                        /> : null}
                        <FormControl size="small"  >
                            <ClearTextField
                                sx={{ mr: 1, width: 100 }}
                                variant="outlined"
                                label={`资源ID`}
                                type="text"
                                name="code"
                                size="small"

                                value={filterData.res_id}
                                onChange={(e, nval) => {
                                    let value = (nval + '').replace(/[^0-9]+/, '');
                                    setfilterData({
                                        ...filterData,
                                        res_id: value
                                    });
                                }}
                            />
                        </FormControl>
                        <FormControl sx={{ minWidth: 120 }} size="small"  >
                            <ClearTextField
                                sx={{ mr: 1, width: 150 }}
                                variant="outlined"
                                label={`资源名称`}
                                type="text"
                                name="code"
                                size="small"
                                value={filterData.res_name}
                                onChange={(e, nval) => {
                                    setfilterData({
                                        ...filterData,
                                        res_name: nval
                                    });
                                }}
                            />
                        </FormControl>
                        <LoadingButton
                            variant="outlined"
                            size="medium"
                            startIcon={<SearchIcon />}
                            sx={{ mr: 1, p: "7px 15px", minWidth: 110 }}
                            onClick={() => {
                                setSearchParam({
                                    ...filterData
                                }, loadResData)
                            }}
                            disabled={pageTagData.tag_rows_loading || pageRowData.rows_loading}
                            loading={pageTagData.tag_rows_loading || pageRowData.rows_loading}
                        >
                            过滤
                        </LoadingButton>
                        <Button
                            variant="outlined"
                            size="medium"
                            startIcon={<AddCircleOutlineIcon />}
                            sx={{ mr: 1, p: "7px 15px" }}
                            onClick={() => {
                                setBoxPage({
                                    ...boxPage,
                                    show: true,
                                    box: "add"
                                })
                            }}
                        >
                            新增资源
                        </Button>
                    </Box> : null
            }
        </Paper>
        <Box sx={{ border: "1px solid #ddd", borderRadius: 1 }}>
            {!page_error ? (!(pageRowData.rows_loading || pageTagData.tag_rows_loading) ? (
                pageRowData.rows.length > 0 ?
                    <Fragment>
                        <Table stickyHeader

                            style={{ borderBottom: "1px solid #ccc" }}>
                            <BaseTableHead
                                columns={columns}
                            />
                        </Table>
                        <List sx={{ pb: 0, mt: 0, pt: 0 }}>
                            {
                                pageRowData.rows.map((item) => {
                                    return <UserAccessResItem
                                        columns={columns}
                                        item={item}
                                        key={`item-${item.res.id}`}
                                        onTagClick={(tag) => {
                                            setSearchParam({
                                                tag: tag
                                            }, loadResData)
                                        }} />
                                })
                            }
                        </List>
                        <Table>
                            <BaseTableFooter
                                count={pageRowData.rows_total}
                                page={parseInt(searchParam.get("page")) || 0}
                                onPageChange={(e, newPage) => {
                                    setSearchParam({

                                        page: newPage
                                    }, loadResData)
                                }}
                                rowsPerPage={parseInt(searchParam.get("page_size")) || 25}
                                onRowsPerPageChange={(e) => {
                                    setSearchParam({

                                        page_size: e.target.value,
                                        page: 0
                                    }, loadResData)
                                }}
                            />
                        </Table>
                    </Fragment> :
                    <BaseTableNoRows />
            ) :
                <Fragment>
                    <Progress />
                    <Box sx={{ m: 2 }}>
                        <Typography variant="h3"> <Skeleton /></Typography>
                        <Typography variant="h1"> <Skeleton /></Typography>
                    </Box>
                </Fragment>) :
                <Alert severity="error">{page_error}</Alert>
            }
        </Box >
    </Fragment >
}