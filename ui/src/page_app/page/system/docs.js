import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import DeleteIcon from '@mui/icons-material/Delete';
import EditIcon from '@mui/icons-material/Edit';
import ReadMoreOutlinedIcon from '@mui/icons-material/ReadMoreOutlined';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Autocomplete, Avatar, Button, Divider, Drawer, FormControl, Grid, IconButton, InputLabel, List, ListItem, ListItemAvatar, ListItemText, MenuItem, Paper, Select, Stack, Switch, Table, TableBody, TableCell, TableRow, TextField, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useRef, useState } from 'react';
import { Form } from 'react-router-dom';
import { ToastContext } from '../../../common/context/toast';
import { ConfirmButton } from '../../../common/ui/dialog';
import { ClearTextField } from '../../../library/input';
import { LoadingButton } from '../../../library/loading';
import { DataTablePage, SimpleTablePage } from '../../../library/table_page';
import { ItemTooltip } from '../../../library/tips';
import { docTagStatus, docsDelGit, docsGitAdd, docsGitDetail, docsGitEdit, docsGitList, docsTagAdd, docsTagCloneDel, docsTagCloneDelList, docsTagDel, docsTagDir, docsTagList, docsTagLogs } from '../../../common/rest/docs';
import { useSearchChange } from '../../../common/utils/hook';
import { showTime } from '../../../common/utils/utils';
import { PageNav } from './menu';
import GiteIcon from '@mui/icons-material/Gite';
import CloseIcon from '@mui/icons-material/Close';
import { docsTagStatusSet } from '../../../common/rest/docs';
import { docsMenuList } from '../../../common/rest/docs';
import { docsMenuAdd } from '../../../common/rest/docs';
import { docsMenuDel } from '../../../common/rest/docs';
import FileCopyIcon from '@mui/icons-material/FileCopy';
function GitBox(props) {
    const { onLoadGit, onEdit, rows, loading } = props;
    const { toast } = useContext(ToastContext);
    let [gitData, setGitData] = useState({
        box: false,
        id: 0,
        url: '',
        name: '',
        max_try: 5,
        loading: false
    });
    const [gitError, setGitError] = useState({
        url: '',
        name: '',
        max_try: '',
    });
    const columns = [
        {
            field: "id",
            label: 'ID',
            align: "right",
            style: { width: 80 }
        },
        {
            field: "name",
            label: '名称',
            style: { width: 120 }
        },
        {
            field: "url",
            label: 'URL',
            style: { width: 220 }
        },
        {
            field: "max_try",
            label: '尝试次数',
            align: "center",
            style: { width: 100 }
        },
        {

            field: "change_user_id",
            label: '操作用户',
            align: "center",
            style: { width: 100 }
        },
        {
            label: '操作时间',
            style: { width: 180 },
            render: (row) => {
                return showTime(row.change_time, "未开始")
            }
        },
        {
            label: '操作',
            style: { width: 120 },
            render: (row) => {
                return <Fragment>
                    <IconButton onClick={() => {
                        setGitData({
                            box: true,
                            id: row.id,
                            url: row.url,
                            name: row.name,
                            max_try: row.max_try,
                        })
                        onEdit(row)
                    }} size='small'>
                        <EditIcon fontSize='small' />
                    </IconButton>
                    <ConfirmButton
                        message={<span>
                            <span style={{ display: "block" }}>该源的文档都会删除!</span>
                            <span style={{ display: "block" }}>确定要进行删除: {row.url} 吗?</span>
                        </span>}
                        onAction={() => {
                            return docsDelGit({
                                git_id: row.id,
                            }).then((data) => {
                                if (!data.status) return data;
                                onLoadGit()
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
    const doGit = () => {
        setGitData({
            ...gitData,
            loading: true
        })
        setGitError({
            url: '',
            name: '',
            max_try: ''
        })
        if (gitData.id > 0) {
            docsGitEdit({
                doc_id: gitData.id,
                name: gitData.name + '',
                url: gitData.url + '',
                max_try: gitData.max_try
            }).then((data) => {
                setGitData({ ...gitData, loading: false });
                if (!data.status) {
                    toast(data.message)
                    setGitError({
                        ...gitError,
                        ...data.field,
                    })
                } else {
                    toast("已保存")
                    onLoadGit()
                }
            })
        } else {
            docsGitAdd({
                name: gitData.name + '',
                url: gitData.url + '',
                max_try: gitData.max_try
            }).then((data) => {
                if (!data.status) {
                    toast(data.message)
                    setGitData({ ...gitData, loading: false });
                    setGitError({
                        ...gitError,
                        ...data.field,
                    })
                } else {
                    toast("添加完成")
                    setGitData({
                        ...gitData,
                        name: "",
                        url: '',
                        tag: '',
                        max_try: 5,
                        loading: false,
                    })
                    onLoadGit()
                }
            })
        }
    }
    return (
        <Fragment>
            <Typography
                align="center"
                variant="subtitle1"
                noWrap
                sx={{
                    mt: 5,
                    mb: 2,
                    fontWeight: 100,
                    alignItems: "center",
                    letterSpacing: '.3rem',
                    color: 'inherit',
                    textDecoration: 'none',
                }}
            >
                Git源管理
            </Typography>
            <Divider variant="middle" />
            <Paper
                component="form"
                sx={{ p: 2, m: 3, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
            >

                {gitData.box ?
                    <Stack sx={{ width: 1, m: 3, position: "relative" }}>
                        <IconButton onClick={() => {
                            setGitData({
                                ...gitData,
                                box: false,
                                url: '',
                                tag: '',
                                max_try: 5,
                                loading: false,
                            })
                        }} sx={{
                            position: 'absolute',
                            right: -32,
                            top: -36
                        }}><CloseIcon fontSize='small' /></IconButton>
                        <TextField
                            disabled={gitData.loading}
                            variant="outlined"
                            label={`名称`}
                            type="text"
                            size="small"
                            onChange={(event) => {
                                setGitData({
                                    ...gitData,
                                    name: event.target.value,
                                })
                            }}
                            value={gitData.name}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            error={!!gitError.name}
                            helperText={gitError.name}
                        />
                        <TextField
                            disabled={gitData.loading}
                            variant="outlined"
                            label={`Git文档地址`}

                            type="text"
                            size="small"
                            onChange={(event) => {
                                setGitData({
                                    ...gitData,
                                    url: event.target.value,
                                })
                            }}
                            value={gitData.url}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            error={!!gitError.url}
                            helperText={gitError.url}
                        />
                        <TextField
                            disabled={gitData.loading}
                            variant="outlined"
                            label={`clone失败重试次数`}

                            type="text"
                            size="small"
                            onChange={(event, nval) => {
                                setGitData({
                                    ...gitData,
                                    max_try: event.target.value,
                                })
                            }}
                            value={gitData.max_try}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            error={!!gitError.max_try}
                            helperText={gitError.max_try}
                        />
                        <LoadingButton disabled={gitData.loading} loading={gitData.loading} onClick={() => {
                            doGit()
                        }} sx={{
                            width: 1,
                        }} variant="contained" type="submit">保存</LoadingButton>
                    </Stack>
                    : <Stack sx={{
                        width: 1,
                        display: "flex",
                        flexFlow: "row nowrap",
                        placeContent: "flex-end space-between",
                        alignItems: "center"
                    }}
                    >
                        <Box>
                            <Button
                                variant="outlined"
                                size="medium"
                                startIcon={<AddCircleOutlineIcon />}
                                sx={{ mr: 1, p: "7px 15px" }}
                                onClick={() => {
                                    setGitData({
                                        ...gitData,
                                        box: true,
                                        id: 0,
                                        url: '',
                                        name: '',
                                        max_try: 5,
                                    })
                                }}>
                                新增
                            </Button>
                        </Box>
                        <Box sx={{ mt: 1 }}>
                            <Typography variant="overline" display="block" gutterBottom>
                                总计:{rows.length ?? 0}个
                            </Typography>
                        </Box>
                    </Stack>}

            </Paper>


            <Stack sx={{ m: 3, mt: 2, mb: 4 }}>
                <DataTablePage
                    rows={(rows ?? [])}
                    columns={columns}
                    loading={loading}
                />
            </Stack>
        </Fragment>)
}

export function MenuInput(props) {
    const { label, tagId, value, onSelect, disabled, ...params } = props;
    const [pathData, setPathData] = useState({
        data: [],
        loading: false,
        status: true,
        message: '',
    })
    const [inputData, setInputData] = useState({
        open: false,
    })
    let ajaxReq = useRef(null)
    let ajaxTimeout = useRef(null)
    useEffect(() => {
        if (pathData.data.find((t) => {
            return t.url_path == value && !t.is_dir
        })) return
        if (!(!value || value == '' || pathData.data.find((t) => {
            return t.url_path == value && t.is_dir
        }) || pathData.data.length == 0)) return

        if (ajaxTimeout.current) {
            clearTimeout(ajaxTimeout.current)
            ajaxTimeout.current = null
        }
        if (ajaxReq.current) {
            ajaxReq.current.abort()
        }
        setPathData({
            ...pathData,
            data: [],
            loading: true,
        })
        ajaxTimeout.current = setTimeout(() => {
            ajaxReq.current = new AbortController();
            return docsTagDir({
                prefix: value,
                tag_id: tagId
            }, {
                signal: ajaxReq.current.signal
            }).then((data) => {
                ajaxReq.current = null;
                ajaxTimeout.current = null
                let setData = data.status && data.data && data.data.length > 0 ? data.data : [];
                setPathData({
                    ...pathData,
                    status: data.status ?? false,
                    message: data.message ?? '',
                    data: setData,
                    loading: false
                })
            })
        }, 800);
    }, [value])

    return <Autocomplete
        {...params}
        disabled={disabled}
        value={value ?? ""}
        options={pathData.data.map((e) => e.url_path) ?? []}
        isOptionEqualToValue={(e, v) => {
            return e == v
        }}
        getOptionLabel={(option) => {
            return option
        }}
        onChange={(_, val) => {
            let is_dir = !!pathData.data.find((t) => {
                return t.url_path == val && t.is_dir
            })
            setInputData({
                ...inputData,
                open: val == '' || !val || is_dir
            })
            if (is_dir) {
                setPathData({
                    ...pathData,
                    data: [],
                    loading: true,
                })
            }
            onSelect(val)
        }}
        noOptionsText={pathData.loading ? "数据加载中..." : "无相关文件"}
        open={inputData.open}
        renderInput={(params) => (
            <Box sx={{ position: "relative" }}>
                <TextField

                    {...params}
                    disabled={disabled}
                    label={label}

                    variant="outlined"
                    onClick={(e) => {
                        setInputData({
                            ...inputData,
                            open: disabled ? false : true,

                        })
                    }}
                    onFocus={(e) => {
                        setInputData({
                            ...inputData,
                            open: disabled ? false : true,
                        })
                    }}
                    onBlur={(e) => {
                        onSelect(e.target.value)
                        setInputData({
                            ...inputData,
                            open: false,
                        })

                    }}
                />
            </Box>
        )}
    />;
}


function MenuBox(props) {
    const { row, onAdd } = props;
    const { toast } = useContext(ToastContext);

    let [menuData, setMenuData] = useState({
        loading: true,
        status: true,
        data: [],
        message: null,
    });
    const loadMenuData = () => {
        setMenuData({
            ...menuData,
            data: [],
            loading: true
        })
        docsMenuList({
            tag_id: row.tag_data.id
        }).then((data) => {
            if (!data.status) {
                setMenuData({
                    ...menuData,
                    loading: false,
                    status: false,
                    message: data.message ?? "加载菜单异常"
                })
                return;
            }
            setMenuData({
                ...menuData,
                data: data.data ?? [],
                loading: false,
                status: true,
            })

        })
    }
    useEffect(() => {
        loadMenuData()
    }, [])

    const columns = [
        {
            field: "id",
            label: 'ID',
            align: "right",
            style: { width: 80 }
        },
        {
            field: "menu_path",
            label: '目录文件',
            style: { width: 180 }
        },
        {

            field: "add_user_id",
            label: '添加用户',
            align: "center",
            style: { width: 100 }
        },
        {
            label: '添加时间',
            style: { width: 190 },
            render: (row) => {
                return showTime(row.add_time, "未开始")
            }
        },
        {
            label: '操作',
            style: { width: 120 },
            render: (row) => {
                return <Fragment>

                    <ConfirmButton
                        message={`确定要删除该目录文件 ${row.id} 吗?`}
                        onAction={() => {
                            return docsMenuDel({
                                menu_id: row.id,
                            }).then((data) => {
                                if (!data.status) return data;
                                loadMenuData()
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
        }
    ];

    let [addData, setAddData] = useState({
        box: false,
        path: '',
        loading: false
    });

    const doMenuAdd = () => {
        setAddData({
            ...addData,
            loading: true
        })
        docsMenuAdd({
            tag_id: row.tag_data.id,
            menu_path: addData.path,
        }).then((data) => {

            if (!data.status) {
                setAddData({ ...addData, loading: false });
                toast(data.message)
            } else {
                setAddData({ ...addData, loading: false, path: "" });
                toast("已添加")
                loadMenuData()
                onAdd()
            }
        })
    }

    return (
        <Fragment>
            <Typography
                align="center"
                variant="subtitle1"
                noWrap
                sx={{
                    mt: 5,
                    mb: 2,
                    fontWeight: 100,
                    alignItems: "center",
                    letterSpacing: '.3rem',
                    color: 'inherit',
                    textDecoration: 'none',
                }}
            >
                Tag目录文件管理
            </Typography>
            <Divider variant="middle" />
            <Paper
                component="form"
                sx={{ p: 2, m: 3, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
            >

                {addData.box ?
                    <Stack sx={{ width: 1, m: 3, position: "relative" }}>
                        <IconButton onClick={() => {
                            setAddData({
                                ...addData,
                                box: false,
                                path: '',
                                loading: false,
                            })
                        }} sx={{
                            position: 'absolute',
                            right: -32,
                            top: -36
                        }}><CloseIcon fontSize='small' /></IconButton>

                        <MenuInput
                            disabled={addData.loading}
                            variant="outlined"
                            label={`目录路径`}
                            type="text"
                            size="small"
                            tagId={row.tag_data.id}
                            onSelect={(val) => {
                                setAddData({
                                    ...addData,
                                    path: val
                                })
                            }}
                            value={addData.path}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                        />

                        <LoadingButton disabled={addData.loading} loading={addData.loading} onClick={() => {
                            doMenuAdd()
                        }} sx={{
                            width: 1,
                        }} variant="contained" type="submit">保存</LoadingButton>
                    </Stack>
                    : <Stack sx={{
                        width: 1,
                        display: "flex",
                        flexFlow: "row nowrap",
                        placeContent: "flex-end space-between",
                        alignItems: "center"
                    }}
                    >
                        <Box>
                            <Button
                                variant="outlined"
                                size="medium"
                                startIcon={<AddCircleOutlineIcon />}
                                sx={{ mr: 1, p: "7px 15px" }}
                                onClick={() => {
                                    setAddData({
                                        ...addData,
                                        box: true,
                                        path: ""
                                    })
                                }}>
                                新增
                            </Button>
                        </Box>
                        <Box sx={{ mt: 1 }}>
                            <Typography variant="overline" display="block" gutterBottom>
                                总计:{menuData.data.length ?? 0}个
                            </Typography>
                        </Box>
                    </Stack>}

            </Paper>


            <Stack sx={{ m: 3, mt: 2, mb: 4 }}>
                {menuData.status ?
                    <DataTablePage
                        rows={(menuData.data ?? [])}
                        columns={columns}
                        loading={menuData.loading}
                    /> : <Alert severity="error">{menuData.message}</Alert>}
            </Stack>

        </Fragment>)
}

function NodeBox(props) {
    const { cloneData, onDelete } = props;
    const { toast } = useContext(ToastContext);
    return (
        <Fragment>
            <Typography
                align="center"
                variant="subtitle1"
                noWrap
                sx={{
                    mt: 5,
                    mb: 2,
                    fontWeight: 100,
                    alignItems: "center",
                    letterSpacing: '.3rem',
                    color: 'inherit',
                    textDecoration: 'none',
                }}
            >
                节点clone状态
            </Typography>
            <Divider variant="middle" />
            <Box sx={{ m: 3 }}>
                {cloneData.length == 0 ? <Alert severity="info">该Tag不存在Clone数据,请稍后查看</Alert> : null}
                <List  >
                    {cloneData.map((e, ci) => {
                        let msg = `克隆中,开始于:${showTime(e.start_time)}`
                        if (e.status == 2) {
                            let st = e.finish_time - e.start_time;
                            msg = `克隆完成于:${showTime(e.finish_time)},用时:`
                            if (st <= 0) {
                                msg += '不到1秒'
                            } else msg += `${st}秒`
                        }

                        return <Fragment key={`clone-${e.id}`}>
                            <ListItem
                                secondaryAction={

                                    <ConfirmButton
                                        key={`clone-${e.id}-del`}
                                        message={<span>
                                            <span style={{ display: "block" }}>{`确定要删除${e.host}上的数据吗?`}</span>
                                            <span>注意:节点在线时可能重新发起clone</span>
                                        </span>}
                                        onAction={() => {
                                            return docsTagCloneDel({
                                                clone_id: e.id
                                            }).then((res) => {
                                                if (!res.status) return res;
                                                onDelete(e.id)
                                                toast("删除完成");
                                                return res;
                                            });
                                        }}
                                        renderButton={(props) => {
                                            return <IconButton title="删除" key={`${e.id}-del-but`} {...props} size='small'>
                                                <DeleteIcon fontSize='small' />
                                            </IconButton>
                                        }} />
                                }
                            >
                                <ListItemAvatar>
                                    <Avatar>
                                        <FileCopyIcon fontSize='small' />
                                    </Avatar>
                                </ListItemAvatar>
                                <ListItemText
                                    primary={`主机:${e.host}`}
                                    secondary={msg}
                                />
                            </ListItem>
                            {cloneData.length - 1 != ci ? <Divider variant="inset" component="li" /> : null}
                        </Fragment>
                    })}

                </List>
            </Box>
        </Fragment>)
}


function TagBox(props) {
    const { onFinish, gitRows } = props;
    const { toast } = useContext(ToastContext);
    const [addData, setAddData] = useState({
        loading: false,
        git_id: '',
        tag: '',
        version: '',
        clear_rule: [],
    });
    const [addError, setAddError] = useState({
        tag: '',
        version: '',
        clear_rule: ''
    });
    let ajaxGitReq = useRef(null)
    const [gitData, setGitData] = useState({
        data: null,
        loading: false,
    });

    useEffect(() => {
        if (!addData.git_id || addData.git_id <= 0) {
            return
        }
        let gitSelect = gitRows.find((e) => {
            return e.id == addData.git_id
        });
        if (!gitSelect) return;
        if (ajaxGitReq.current) {
            ajaxGitReq.current.abort()
        }
        ajaxGitReq.current = new AbortController();
        setGitData({
            ...gitData,
            loading: true
        })
        docsGitDetail({
            url: gitSelect.url + '',
        }, {
            signal: ajaxGitReq.current.signal
        }).then((data) => {
            ajaxGitReq.current = null;
            if (data.status && (
                !data.data ||
                data.data.length == 0
            )) {
                data.status = false;
                data.message = "请先添加tag";
            }
            if (!data.status) {
                toast(data.message)
                setGitData({
                    ...gitData,
                    loading: false
                })
            } else {
                setGitData({
                    ...gitData,
                    data: data.data,
                    loading: false
                })
            }
        })
    }, [addData.git_id])
    useEffect(() => {
        if (!gitData.data || gitData.data.length == 0) return

        setAddData({
            ...addData,
            tag: gitData.data[0].tag,
            version: gitData.data[0].version,
        })

    }, [
        gitData.data
    ])
    const doAdd = function () {
        setAddData({
            ...addData,
            loading: true
        })
        docsTagAdd({
            git_id: addData.git_id,
            tag: addData.tag + '',
            version: addData.version + '',
            clear_rule: addData.clear_rule ?? [],
        }).then((data) => {
            if (!data.status) {
                toast(data.message)
                setAddError({
                    ...addError,
                    ...data.field
                })
                setAddData({
                    ...addData,
                    loading: false
                })
            } else {
                setGitData({
                    ...gitData,
                    data: null,
                })
                setAddData({
                    ...addData,
                    git_id: '',
                    tag: '',

                    loading: false,
                    version: '',

                    clear_rule: [],
                })
                onFinish()
            }
        })
    };

    return (
        <Fragment>
            <Typography
                align="center"
                variant="subtitle1"
                noWrap
                sx={{
                    mt: 5,
                    mb: 2,
                    fontWeight: 100,
                    alignItems: "center",
                    letterSpacing: '.3rem',
                    color: 'inherit',
                    textDecoration: 'none',
                }}
            >
                添加Tag
            </Typography>
            <Divider variant="middle" />
            <Form method="post" onSubmit={(e) => {
                e.preventDefault();

            }}>
                <Grid
                    sx={{
                        mt: 5,
                    }}
                    container
                    justifyContent="center"
                    alignItems="center"
                >
                    <Grid item xs={10}>

                        <FormControl fullWidth >
                            <InputLabel size="small" id="user-select-label">选择Git源</InputLabel>
                            <Select
                                disabled={gitData.loading}
                                size='small'
                                fullWidth
                                sx={{
                                    width: 1,
                                    mb: 2
                                }}
                                label="选择Git源"
                                value={addData.git_id ?? ''}
                                onChange={(e) => {
                                    setAddData({
                                        ...addData,
                                        git_id: e.target.value,
                                    })
                                    setGitData({
                                        data: null,
                                        loading: false,
                                    })
                                }}
                            >
                                {
                                    gitRows.map((item) => {
                                        return <MenuItem key={`git-tag-add-${item.id}`} value={item.id} title={item.url}>{item.name}</MenuItem>;
                                    })
                                }
                            </Select>
                        </FormControl>
                    </Grid>
                    {gitData.data ? <Fragment>
                        <Grid item xs={10}>
                            <FormControl fullWidth >
                                <InputLabel size="small" id="user-select-label">选择Tag</InputLabel>
                                <Select
                                    size='small'
                                    fullWidth
                                    sx={{
                                        width: 1,
                                        mb: 2
                                    }}
                                    label="选择Tag"
                                    value={addData.tag}
                                    onChange={(e) => {
                                        gitData.data.map((item) => {
                                            if (item.tag == e.target.value) {
                                                setAddData({
                                                    ...addData,
                                                    tag: item.tag,
                                                    version: item.version
                                                })
                                            }
                                        })
                                    }}
                                >
                                    {
                                        gitData.data.map((item, ii) => {
                                            return <MenuItem key={`git-tag-${ii}`} value={item.tag}>{item.tag}</MenuItem>;
                                        })
                                    }
                                </Select>
                            </FormControl>
                        </Grid>
                        <Grid item xs={10}>
                            <TextField
                                variant="outlined"
                                label={`Tag版本`}
                                disabled={true}
                                type="text"
                                size="small"

                                onChange={(e) => {
                                    setAddData({
                                        ...addData,
                                        version: e.target.value
                                    })
                                }}
                                value={addData.version}
                                sx={{
                                    width: 1,
                                    mb: 2,
                                }}
                                error={!!addError.version}
                                helperText={addError.version}
                            />
                        </Grid>

                        <Grid item xs={10}>
                            <TextField
                                fullWidth
                                multiline
                                row={2}
                                maxRows={3}
                                sx={{ mb: 1 }}
                                variant="outlined"
                                label={`克隆后删除规则`}
                                type="text"
                                size="small"
                                onChange={(e) => {
                                    let tt = (e.target.value + '').split("\n")
                                    tt = tt.map((t) => { return t.replace(/^\s+/, '').replace(/\s+$/, '') })
                                    let bb = tt.pop()
                                    tt = tt.filter((e) => e && e.length > 0)
                                    tt.push(bb)
                                    setAddData({
                                        ...addData,
                                        clear_rule: tt
                                    })
                                }}
                                value={addData.clear_rule.join("\n")}
                            />
                        </Grid>


                        <Grid item xs={10}  >
                            <LoadingButton disabled={addData.loading} loading={addData.loading} onClick={() => {
                                doAdd()
                            }} sx={{
                                width: 1,
                                mb: 2
                            }} variant="contained" type="submit"> {"添加"}</LoadingButton>
                        </Grid>
                    </Fragment> : null}
                </Grid>
            </Form >
        </Fragment>)
}


function LogsBox(props) {
    const { row } = props;
    let [logsData, setLogsData] = useState({
        loading: true,
        status: true,
        data: [],
        message: null,
    });
    const loadLogsData = () => {
        setLogsData({
            ...logsData,
            loading: true
        })
        docsTagLogs({
            tag_id: row.tag_data.id
        }).then((data) => {
            if (!data.status) {
                setLogsData({
                    ...logsData,
                    loading: false,
                    status: false,
                    message: data.message ?? "加载菜单异常"
                })
                return;
            }
            setLogsData({
                ...logsData,
                data: data.data ?? [],
                loading: false,
                status: true,
            })

        })
    }
    useEffect(() => {
        loadLogsData()
    }, [])

    const columns = [
        {
            field: "id",
            label: 'ID',
            align: "right",
            style: { width: 80 }
        },
        {
            field: "host",
            label: '触发节点',
            style: { width: 180 }
        },

        {
            field: "message",
            label: '日志消息',
        },
        {
            label: '添加时间',
            style: { width: 180 },
            render: (row) => {
                return showTime(row.add_time, "未开始")
            }
        }
    ];



    return (
        <Fragment>
            <Typography
                align="center"
                variant="subtitle1"
                noWrap
                sx={{
                    mt: 5,
                    mb: 2,
                    fontWeight: 100,
                    alignItems: "center",
                    letterSpacing: '.3rem',
                    color: 'inherit',
                    textDecoration: 'none',
                }}
            >
                Tag{row.tag_data.tag}[{row.tag_data.id}]相关日志
            </Typography>
            <Divider variant="middle" />
            {logsData.status ? <Stack sx={{ m: 3, mt: 2, mb: 4 }}>
                <DataTablePage
                    rows={(logsData.data ?? [])}
                    columns={columns}
                    loading={logsData.loading}
                />
            </Stack> : <Alert severity="error">{logsData.message}</Alert>}


        </Fragment>)
}



export default function SystemDocsPage(props) {
    const { toast } = useContext(ToastContext);
    const [searchParam, setSearchParam] = useSearchChange({
        key_word: "",
        status: "",
        git_id: "",
        page: 0,
        page_size: 25,
    });

    //过滤组件数据
    const [filterData, setfilterData] = useState({
        key_word: "",
        status: "",
        git_id: "",
    })
    let [gitData, setGitData] = useState({
        loading: true,
        status: true,
        data: [],
        message: null,
    });
    const loadGitData = () => {
        setGitData({
            ...gitData,
            data: [],
            loading: true
        })
        docsGitList().then((data) => {
            if (!data.status) {
                setGitData({
                    ...gitData,
                    loading: false,
                    status: false,
                    message: data.message ?? "加载GIT异常"
                })
                return;
            }
            setGitData({
                ...gitData,
                data: data.data ?? [],
                loading: false,
                status: true,
            })
        })
    }
    useEffect(() => {
        loadGitData()
    }, [])
    //列表数据
    let [loadData, setLoadData] = useState({
        loading: true,
        status: false,
        data: [],
        total: 0,
        message: null,
    });
    const loadDocsData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        docsTagList({
            key_word: searchParam.get("key_word"),
            page: searchParam.get("page"),
            page_size: searchParam.get("page_size"),
            git_id: searchParam.get("git_id"),
            status: searchParam.get("status"),
        }).then((data) => {
            let doc_data = data.status ? data.data : [];
            doc_data = doc_data.map((item) => {
                if (item.tag_data && item.tag_data.clear_rule) {
                    let clear_rule = JSON.parse(item.tag_data.clear_rule)
                    if (!clear_rule) clear_rule = [];
                    item.tag_data.clear_rule = clear_rule;
                }
                return item
            })
            setLoadData({
                ...loadData,
                ...data,
                data: doc_data,
                loading: false
            })
        })
    };
    useEffect(loadDocsData, [])


    useEffect(() => {
        setfilterData({
            ...filterData,
            git_id: searchParam.get("git_id"),
            status: searchParam.get("status"),
            key_word: searchParam.get("key_word"),
        })
        loadDocsData()
    }, [searchParam])

    const subPage = useRef(null);
    //添加跟更新
    const [changeBoxState, setChangeBox] = useState({
        box: 0,
        row: null
    });
    let pageBox
    switch (changeBoxState.box) {
        case 1:
            pageBox = <TagBox
                gitRows={gitData.data ?? []}
                onFinish={() => {
                    loadDocsData()
                }}
            />;
            break
        case 3:
            pageBox = <LogsBox
                row={changeBoxState.row}
            />
            break
        case 4:
            pageBox = <GitBox
                rows={gitData.data}
                loading={gitData.loading}
                onLoadGit={() => {
                    loadGitData()
                }}
                onEdit={() => {
                    if (subPage.current) {
                        subPage.current.scrollIntoView({ behavior: 'smooth' });
                    }
                }}
            />

            break
        case 5:
            pageBox = <NodeBox
                cloneData={changeBoxState.row.clone_data}
                onDelete={(clone_id) => {
                    let row = { ...changeBoxState.row }
                    row.clone_data = row.clone_data.filter((item) => {
                        if (item.id != clone_id) return item;
                    })
                    setChangeBox({
                        ...changeBoxState,
                        row: row
                    })
                    setLoadData({
                        ...loadData,
                        data: loadData.data.map((item) => {
                            if (changeBoxState.row.tag_data.id == item.tag_data.id) {
                                return row
                            } else return item
                        })
                    })
                }}
            />

            break
        case 6:
            pageBox = <MenuBox
                row={changeBoxState.row}
                onAdd={() => {
                    loadDocsData()
                }}
            />

            break

    };





    const columns = [
        {
            field: 'id',
            label: 'ID',
            style: { width: 90 },
            align: "right",
            render: (row) => {
                return row.tag_data.id;
            }
        },
        {
            field: 'GIT源',
            style: { width: 120 },
            align: "center",
            label: 'URL',
            render: (row) => {
                return <ItemTooltip title={row.git_data.url} placement="top">
                    <Box>{row.git_data.name}</Box>
                </ItemTooltip>

            }
        },
        {
            field: 'tag',
            style: { width: 140 },
            align: "center",
            label: '使用Tag',
            render: (row) => {
                return <ItemTooltip title={row.tag_data.build_version} placement="top">
                    <Box>{row.tag_data.tag}</Box>
                </ItemTooltip>
            }
        },

        {
            style: { width: 170 },
            label: '添加时间',
            align: "center",
            render: (row) => {
                return showTime(row.tag_data.add_time, "未知")
            }
        },
        {
            style: { width: 140 },
            label: '节点详细',
            align: "center",
            render: (row) => {
                if (row.clone_data && row.clone_data.length > 0) {
                    return <Button onClick={() => {
                        setChangeBox({ box: 5, row: row })
                    }} fontSize="small">{row.clone_data.length}个节点</Button>
                } else return "待clone"
            }
        },
        {
            style: { width: 140 },
            label: '目录文件',
            align: "center",
            render: (row) => {
                if (row.menu_num > 0) {
                    return <Button onClick={() => {
                        setChangeBox({ box: 6, row: row })
                    }} fontSize="small">{row.menu_num}个目录</Button>
                } else return <Button onClick={() => {
                    setChangeBox({ box: 6, row: row })
                }} fontSize="small">请添加</Button>
            }
        },

        {
            style: { width: 180 },
            label: '状态',
            align: "center",
            render: (row) => {
                let status = docTagStatus.find((t) => t.key == row.tag_data.status)
                return status ? status.val : "未知";
            }
        },
        {
            label: '操作',
            style: { width: 150 },
            align: "center",
            render: (row) => {

                return <Fragment>
                    <Switch size="small" checked={loadData.data.find((t) => {
                        return t.tag_data.id == row.tag_data.id
                    })?.tag_data.status == 2}
                        onChange={
                            (e) => {
                                let tmpData = [...loadData.data].map((t) => {
                                    if (t.tag_data.id == row.tag_data.id) {
                                        t.tag_data.status = e.target.checked ? 2 : 1
                                    }
                                    return t
                                });
                                setLoadData({
                                    ...loadData,
                                    data: tmpData,
                                })
                                return docsTagStatusSet({
                                    tag_id: row.tag_data.id,
                                    status: e.target.checked ? 2 : 1
                                }).then((res) => {
                                    if (!res.status) {
                                        toast(res.message)
                                        let tmpData = [...loadData.data].map((t) => {
                                            if (t.tag_data.id == row.tag_data.id) {
                                                t.tag_data.status = e.target.checked ? 1 : 2
                                            }
                                            return t
                                        });
                                        setLoadData({
                                            ...loadData,
                                            data: tmpData,
                                        })
                                    }
                                });
                            }
                        } />
                    <IconButton title="日志" key={`item-${row.tag_data.id}-show`} onClick={() => {
                        setChangeBox({ box: 3, row: row })
                    }} size='small'>
                        <ReadMoreOutlinedIcon fontSize='small' />
                    </IconButton>
                    <ConfirmButton
                        key={`item-${row.tag_data.id}-del`}
                        message={`确定要删除此Tag [${row.tag_data.id}] ?`}
                        onAction={() => {
                            return docsTagDel({
                                tag_id: row.tag_data.id
                            }).then((res) => {
                                if (!res.status) return res;
                                let rows = loadData.data.filter((item) => {
                                    if (item.tag_data.id != row.tag_data.id) return item;
                                })
                                setLoadData({
                                    ...loadData,
                                    data: rows
                                })
                                toast("删除完成");
                                if (rows.length == 0) {
                                    loadDocsData()
                                }
                                return res;
                            });
                        }}
                        renderButton={(props) => {
                            return <IconButton title="删除" key={`${row.tag_data.id}-sms`} {...props} size='small'>
                                <DeleteIcon fontSize='small' />
                            </IconButton>
                        }} />

                </Fragment>
            }
        },
    ];

    return <Fragment>
        <PageNav />
        <Drawer
            sx={{ zIndex: (theme) => theme.zIndex.drawer + 3 }}
            anchor={"right"}
            open={changeBoxState.box != 0}
            onClose={() => {
                setChangeBox({ box: 0, row: null })
            }}
        >
            <Box
                ref={subPage}
                sx={{ width: changeBoxState.box == 3 ? 700 : (changeBoxState.box == 6 ? 680 : (changeBoxState.box == 4 ? 960 : 480)) }}
                role="presentation"
            >
                {pageBox}
            </Box>
        </Drawer>
        <Paper
            component="form"
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
        >

            {gitData.status && gitData.data.length ? <FormControl sx={{
                minWidth: 130,
                mr: 1
            }}>
                <InputLabel size="small" >Git源</InputLabel>
                <Select
                    fullWidth
                    size='small'
                    disabled={loadData.loading}
                    value={filterData.git_id ?? ''}
                    onChange={
                        (e) => {
                            setfilterData({
                                ...filterData,
                                git_id: e.target.value
                            })
                        }
                    }
                    label="Git源"><MenuItem value=''>全部</MenuItem>
                    {gitData.data.map((item) => {
                        return <MenuItem key={`git-select-${item.id}`} value={item.id} title={item.url}>{item.name}</MenuItem>
                    })}
                </Select>
            </FormControl> : null}
            <FormControl sx={{
                minWidth: 130,
                mr: 1
            }}>
                <InputLabel size="small" >状态</InputLabel>
                <Select
                    fullWidth
                    disabled={loadData.loading}
                    size='small'
                    value={filterData.status ?? ''}
                    onChange={
                        (e) => {
                            setfilterData({
                                ...filterData,
                                status: e.target.value
                            })
                        }
                    }
                    label="状态"><MenuItem value=''>全部</MenuItem>
                    {docTagStatus.map((item) => {
                        return <MenuItem key={`tag-status-${item.key}`} value={item.key} >{item.val}</MenuItem>
                    })}
                </Select>
            </FormControl>
            <FormControl sx={{ minWidth: 80, mr: 1 }} size="small"  >
                <ClearTextField

                    variant="outlined"
                    label={`搜索关键字`}
                    type="text"
                    name="code"
                    value={filterData.key_word}
                    size="small"
                    disabled={loadData.loading}
                    onChange={(event, nval) => {
                        setfilterData({
                            ...filterData,
                            key_word: nval
                        })
                    }}
                />
            </FormControl>
            <LoadingButton
                onClick={() => {
                    setSearchParam({
                        ...filterData,
                        page: 0
                    }, loadDocsData)
                }}
                variant="outlined"
                size="medium"
                startIcon={<SearchIcon />}
                sx={{ mr: 2, p: "7px 15px", minWidth: 110 }}
                loading={loadData.loading}
                disabled={loadData.loading}
            >
                过滤
            </LoadingButton>

            {gitData.status && gitData.data.length ? <Button
                variant="outlined"
                size="medium"
                startIcon={<AddCircleOutlineIcon />}
                sx={{ mr: 1, p: "7px 15px" }}
                onClick={() => {
                    setChangeBox({ box: 1, row: null })

                }}>
                添加文档
            </Button> : null}
            <Button
                variant="outlined"
                size="medium"
                startIcon={<GiteIcon />}
                sx={{ mr: 1, p: "7px 15px" }}
                onClick={() => {
                    setChangeBox({ box: 4, row: null })

                }}>
                Git源管理
            </Button>
        </Paper>

        {(loadData.status || loadData.loading)
            ? <Box sx={{ height: 1, width: '100%' }}>
                <SimpleTablePage
                    rows={loadData.data}
                    columns={columns}
                    count={loadData.total}
                    page={searchParam.get("page") || 0}
                    onPageChange={(e, newPage) => {
                        setSearchParam({
                            page: newPage
                        }, loadDocsData)
                    }}
                    rowsPerPage={searchParam.get("page_size") || 25}
                    onRowsPerPageChange={(e) => {
                        setSearchParam({
                            page_size: e.target.value,
                            page: 0
                        }, loadDocsData)
                    }}
                    loading={loadData.loading}
                />

            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}