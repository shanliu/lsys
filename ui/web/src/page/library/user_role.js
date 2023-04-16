import { default as AddCircleOutline, default as AddCircleOutlineIcon } from '@mui/icons-material/AddCircleOutline';
import AllOutIcon from '@mui/icons-material/AllOut';
import DeleteIcon from '@mui/icons-material/Delete';
import EditIcon from '@mui/icons-material/Edit';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Autocomplete, Box, Button, Divider, Drawer, FormControl, FormControlLabel, FormGroup, FormHelperText, FormLabel, Grid, IconButton, InputLabel, List, ListItem, MenuItem, Paper, Select, Switch, TextField, ToggleButton, ToggleButtonGroup, Typography } from '@mui/material';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableRow from '@mui/material/TableRow';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import { DesktopDateTimePicker } from '@mui/x-date-pickers/DesktopDateTimePicker';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import dayjs from 'dayjs';
import PropTypes from 'prop-types';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { ToastContext } from '../../context/toast';
import { ConfirmButton } from '../../library/dialog';
import { ClearTextField, InputTagSelect, LoadSelect, SliderInput, TagSelect } from '../../library/input';
import { LoadingButton, Progress } from '../../library/loading';
import { BaseTableBodyRow, BaseTableFooter, BaseTableHead, BaseTableNoRows, BaseTablePage } from '../../library/table_page';
import { ItemTooltip } from '../../library/tips';
import { resListData, roleAdd, roleAddUser, roleDelete, roleDeleteUser, roleEdit, roleListData, roleListUser, roleOptions, roleRelationCheck, roleTags } from '../../rest/access';
import { useSearchChange } from '../../utils/hook';
import { showTime } from '../../utils/utils';
import { RoleResOpGroupItem, RoleResOpItem, UserTags } from './user';
import { roleRelationData } from '../../rest/access';

//添加角色资源选择
function UserResSelect(props) {
    const {
        userId,
        size,
        onChange,
        value,
        disabled,
        ...other
    } = props;
    const [resData, setResData] = useState({
        loading: false,
        next: true,
        page: 0,
        show: 10,
        items: [],
        item_ops: [],
        item_ops_cache: {},
        value: '',
        op_value: '',
        error: null
    });
    const [allowData, setAllowData] = useState({
        value: 'allow',
    });
    //[{res_key:'',res_name:'',ops:[{op_name: '',op_key: '',allow:false,op_id:1}]}]
    const [listResData, setListResData] = useState([])
    const changeListResData = (resData) => {
        setListResData(resData)
        onChange(resData)
    }
    useEffect(() => {
        setListResData(value)
    }, [props.value])
    return <FormControl {...other}>
        <FormLabel style={{
            position: "absolute",
            transform: "translate(0, -12px) scale(0.75)"
        }}>绑定权限</FormLabel>
        <Box className='MuiInputBase-root MuiOutlinedInput-root MuiInputBase-colorPrimary MuiInputBase-formControl MuiInputBase-sizeSmall'
            style={{
                borderRadius: "4px",
                marginBottom: "4px"
            }}>
            <fieldset style={{
                textAlign: "left",
                position: "absolute",
                bottom: 0,
                right: 0,
                top: "-5px",
                left: 0,
                margin: 0,
                padding: "0 8px",
                pointerEvents: "none",
                borderRadius: "inherit",
                borderStyle: "solid",
                borderWidth: "1px ",
                overflow: "hidden",
                borderColor: " rgba(0, 0, 0, 0.23)",
            }} className="MuiOutlinedInput-notchedOutline "><legend style={{
                visibility: "hidden"
            }} ><span>绑定权限</span></legend></fieldset>
        </Box>
        <Box sx={{ mt: 1, mb: 1 }}>
            {
                (listResData.length == 0) ?
                    <div style={{
                        textAlign: "center",
                        fontSize: "0.9rem",
                        color: "#999",
                        lineHeight: 3
                    }}>请添加权限</div>
                    : listResData.map((item, i) => {
                        return <RoleResOpGroupItem
                            key={`res-op-item-${i}`}
                            resName={item.res_name}
                            resKey={item.res_key}
                        >
                            {
                                item.ops.map((op, i) => {
                                    return <RoleResOpItem
                                        key={`op-item-${i}`}
                                        allow={op.allow}
                                        opName={op.op_name}
                                        opKey={op.op_key}
                                        onDelete={(op_key) => {
                                            let res_key = item.res_key;
                                            let items = [];
                                            listResData.map((res) => {
                                                let tmp;
                                                if (res.res_key == res_key) {
                                                    let tmpop = [];
                                                    item.ops.map((op) => {
                                                        if (op.op_key != op_key) {
                                                            tmpop.push({ ...op })
                                                        }
                                                    });
                                                    if (tmpop.length > 0) {
                                                        tmp = {
                                                            ...res,
                                                            ops: tmpop
                                                        };
                                                    }
                                                } else {
                                                    tmp = { ...res };
                                                }
                                                if (tmp) items.push(tmp)
                                            })
                                            changeListResData(items)
                                        }}
                                    />
                                })}
                        </RoleResOpGroupItem>
                    })
            }
        </Box>
        <Divider sx={{ mb: 1 }}></Divider>
        <Grid container item sx={{ mt: 1 }}>
            <Grid item xs={4}>
                <FormControl fullWidth sx={{
                    width: 1,
                    paddingBottom: 1
                }}>
                    <InputLabel size={size} id="user-res-select-label">选择资源</InputLabel>
                    <LoadSelect
                        label="选择资源"
                        size={size}
                        labelId="user-res-select-label"
                        id="user-res-select"
                        loading={resData.loading}
                        next={resData.next}
                        value={resData.value}
                        error={resData.error}
                        onChange={(e) => {
                            let val = e.target.value;
                            let res = resData.items.find((e) => {
                                return e.res_key == val
                            })
                            if (!res) {
                                setResData({ ...resData, loading: false, value: val })
                                return
                            }
                            let cache_item = resData.item_ops_cache[res.id] ?? null;
                            if (cache_item) {
                                setResData({
                                    ...resData,
                                    item_ops: cache_item.ops,
                                    op_value: cache_item.ops[0].op_key ?? '',
                                    value: val
                                })
                                return;
                            }
                            resListData({
                                user_id: userId,
                                tags: false,
                                res_id: res.id,
                                ops: res.id > 0,
                                count_num: res.id == 0,
                                page: 0,
                                page_size: 1
                            }).then((data) => {
                                if (!data.status) {
                                    setResData({
                                        ...resData,
                                        error: resData.items.length > 0 ? null : data.message
                                    })
                                    return;
                                }
                                let items = (data.data ?? [])[0];
                                if (!items) return;
                                let cache = { ...resData.item_ops_cache };
                                cache[items.res.id] = items;
                                setResData({
                                    ...resData,
                                    item_ops: items.ops ?? [],
                                    item_ops_cache: cache,
                                    op_value: items.ops[0].op_key ?? '',
                                    value: val
                                })
                            })
                        }}
                        onLoad={() => {
                            setResData({ ...resData, loading: true })
                            resListData({
                                user_id: userId,
                                tags: false,
                                ops: false,
                                count_num: true,
                                page: resData.page,
                                page_size: resData.show
                            }).then((data) => {
                                if (!data.status) {
                                    setResData({
                                        ...resData,
                                        loading: false,
                                        next: false,
                                        error: resData.items.length > 0 ? null : data.message
                                    })
                                    return;
                                }
                                let items = (data.data ?? []).map((e) => {
                                    return e.res
                                });
                                setResData({
                                    ...resData,
                                    items: [...resData.items, ...items],
                                    loading: false,
                                    page: resData.page + 1,
                                    next: resData.page * resData.show < data.total
                                })
                            })
                        }}
                    >
                        {resData.items.map((item) => {
                            return <MenuItem key={`res-${item.res_key}`} value={item.res_key}>{item.name}</MenuItem>
                        })}
                    </LoadSelect>
                </FormControl>
            </Grid>
            <Grid item xs={4}>
                <FormControl fullWidth sx={{
                    width: 1,
                    paddingBottom: 1,
                    pl: 1,
                }}>
                    <InputLabel size={size} id="user-res-op-select-label">选择操作</InputLabel>
                    <Select
                        disabled={disabled}
                        label="选择操作"
                        size={size}
                        labelId="user-res-op-select-label"
                        id="user-res-op-select"
                        value={resData.op_value}
                        onChange={(e) => {
                            setResData({
                                ...resData,
                                op_value: e.target.value
                            })
                        }}
                    >
                        {resData.item_ops.map((item) => {
                            return <MenuItem key={`res-op-${item.op_key}`} value={item.op_key}>{item.name}</MenuItem>
                        })}
                    </Select>
                </FormControl>

            </Grid>
            <Grid item xs={3} sx={{ textAlign: "center" }}>
                <ToggleButtonGroup
                    disabled={disabled}
                    exclusive
                    sx={{ w: 1 }}
                    size={size}
                    value={allowData.value}
                >
                    <ToggleButton disableRipple value="allow" onClick={() => {
                        setAllowData({
                            ...allowData,
                            value: "allow"
                        })
                    }} sx={{
                        "&.Mui-selected": {
                            background: "#d7ebff"
                        },
                        "&.Mui-selected:hover": {
                            background: "#ddeeff"
                        }
                    }}>
                        授权
                    </ToggleButton>
                    <ToggleButton disableRipple value="deny" onClick={() => {
                        setAllowData({
                            ...allowData,
                            value: "deny"
                        })
                    }} sx={{
                        "&.Mui-selected": {
                            background: "#ffeeee"
                        },
                        "&.Mui-selected:hover": {
                            background: "#fff1f1"
                        }
                    }} >
                        禁止
                    </ToggleButton>
                </ToggleButtonGroup>
            </Grid>
            <Grid item xs={1}>
                <Button variant="outlined"
                    disabled={disabled} sx={{
                        borderColor: "#aaa",
                        minWidth: "20px",
                        padding: "8px 6px",
                        '&:hover svg': {
                            color: '#1976d2'
                        }
                    }} >
                    <AddCircleOutlineIcon
                        onClick={() => {
                            let res = resData.items.find((e) => {
                                return e.res_key == resData.value
                            })
                            let res_op = resData.item_ops.find((e) => {
                                return e.op_key == resData.op_value
                            })
                            //[{res_key:'',res_name:'',ops:[{op_name: '',op_key: '',allow:false,op_id:1}]}]
                            let find;
                            let items = listResData.map((item) => {
                                if (item.res_key == res.res_key) {
                                    find = true;
                                    let find_op = false;
                                    let ops = item.ops.map((op) => {
                                        if (op.op_key == res_op.op_key) {
                                            find_op = true;
                                            return {
                                                ...op,
                                                allow: allowData.value == 'allow'
                                            }
                                        }
                                        return { ...op };
                                    })
                                    if (!find_op) {
                                        ops.push({
                                            op_id: res_op.id,
                                            op_key: res_op.op_key,
                                            op_name: res_op.name,
                                            allow: allowData.value == 'allow'
                                        })
                                    }
                                    return {
                                        ...item,
                                        ops: ops
                                    }
                                }
                                return { ...item };
                            })
                            if (!find) {
                                items.push({
                                    res_name: res.name,
                                    res_key: res.res_key,
                                    ops: [{
                                        op_id: res_op.id,
                                        op_key: res_op.op_key,
                                        op_name: res_op.name,
                                        allow: allowData.value == 'allow'
                                    }]
                                })
                            }
                            changeListResData(items)
                        }}
                        fontSize={size}
                        sx={{
                            color: "#666",
                        }} />
                </Button>
            </Grid>
        </Grid>
    </FormControl >
}


//关系KEY输入框
function UserRoleRelationInput(props) {
    const { value, options, onFinish, onChange, disabled, ...params } = props;
    //过滤组件数据
    const [relationData, setRelationData] = useState({
        relation_open: false,
        value: value
    })
    useEffect(() => {
        setRelationData({
            ...relationData,
            value: value
        })
    }, [props.value])
    return <Autocomplete
        {...params}
        disabled={disabled}
        value={relationData.value ?? ''}
        options={options ?? []}
        getOptionLabel={(option) => {
            return option
        }}
        noOptionsText={"回车确认"}
        open={relationData.relation_open && !disabled}
        renderInput={(params) => (
            <TextField
                {...params}
                variant="outlined"
                onChange={onChange}
                onClick={(e) => {
                    setRelationData({
                        ...relationData,
                        relation_open: true,
                    })
                }}
                onFocus={(e) => {
                    setRelationData({
                        ...relationData,
                        relation_open: true,
                    })
                    onFinish(e.target.value);
                }}
                onBlur={(e) => {

                    setRelationData({
                        ...relationData,
                        relation_open: false,
                    })
                    onFinish(e.target.value);
                }}
                onKeyUp={(e) => {
                    if (!options.map((item) => {
                        return item == e.target.value
                    })) {
                        e.stopPropagation();
                        e.preventDefault();
                    }
                }}
                onKeyDown={(e) => {
                    if (e.key != 'Enter')
                        return
                    setRelationData({
                        ...relationData,
                        relation_open: false,
                    })
                    e.stopPropagation();
                    e.preventDefault();
                    onFinish(e.target.value);
                }}
            />
        )}
    />;
}


function UserRoleRelationSet(props) {
    const { userId, options, value, ...params } = props;
    const { toast } = useContext(ToastContext);
    const [relationData, setRelationData] = useState({
        abort: null,
        timeout: null,
        now_value: "",
        err: null,
        data: options,
    })
    useEffect(() => {
        if (relationData.timeout) {
            clearTimeout(relationData.timeout)
        }
        if (relationData.abort) {
            relationData.abort.abort()
        }
        if (!relationData.now_value || relationData.now_value.length == 0) return
        setRelationData({
            ...relationData,
            timeout: setTimeout(() => {
                let abort = new AbortController();
                setRelationData({
                    ...relationData,
                    abort: abort
                })
                roleRelationCheck({
                    user_id: parseInt(userId),
                    relation_find: [relationData.now_value],
                }, {
                    signal: abort.signal
                }).then((data) => {
                    if (!data.status) {
                        if (!(data.message + '').indexOf("cancel")) {
                            toast("关系数据获取异常:" + data.message)
                        }
                    } else {
                        if (data.relation_find && data.relation_find.length > 0) {
                            setRelationData({
                                ...relationData,
                                abort: null,
                                timeout: null,
                                err: `已存在${data.relation_find[0].relation_key}关系角色 名称 :${data.relation_find[0].name} ID:${data.relation_find[0].id}`
                            })
                        } else {
                            setRelationData({
                                ...relationData,
                                abort: null,
                                timeout: null,
                            })
                        }

                    }
                })
            }, 800)
        })
    }, [relationData.now_value])


    return <Fragment>
        <UserRoleRelationInput
            {...params}
            options={relationData.data}
            value={value ?? ""}
            onChange={(e) => {
                let val = e.target.value;
                setRelationData({
                    ...relationData,
                    now_value: val.replace(/^\s+/, '').replace(/\s+$/, '')
                })
            }} />
        {relationData.err ? <Alert sx={{
            marginTop: 1
        }} severity="error">{relationData.err}</Alert> : null}
    </Fragment>
}

//角色添加或编辑弹出页
function UserRoleAdd(props) {
    const { title, tags, rowData, initData, onSave, userId } = props;
    let tags_options = (tags ?? []).map((e) => { return e[0] })
    const initAddData = {

        loading: false,
        res_name: '',
        tags: [],
        priority: 50,
        res_range: initData.res_range[0].key,
        user_range: initData.user_range[0].key,
        user_access_key: '',
        user_select: []
    }
    const [addData, setAddData] = useState(initAddData)
    useEffect(() => {
        if (!rowData?.role) return;
        let tags = (rowData.tags ?? []).map((e) => { return e.name });
        let user_select = [];
        //[{res_key:'',res_name:'',ops:[{op_name: '',op_key: '',allow:false,op_id:1}]}]
        (rowData.ops ?? []).map((op) => {
            if (!user_select[op.res.id]) user_select[op.res.id] = {
                res_key: op.res.res_key,
                res_name: op.res.name,
                ops: [],
            }
            user_select[op.res.id].ops.push({
                op_name: op.res_op.name,
                op_key: op.res_op.op_key,
                op_id: op.res_op.id,
                allow: op.role_op.positivity == 1,
            });
        })

        setAddData({
            ...addData,
            res_name: rowData.role.name || '',
            priority: rowData.role.priority || 50,
            res_range: rowData.role.res_op_range || initData.res_range[0].key,
            user_range: rowData.role.user_range || initData.user_range[0].key,
            user_access_key: rowData.role.relation_key || '',
            tags: tags,
            user_select: user_select
        })
    }, [rowData])

    const { toast } = useContext(ToastContext);
    const submitAction = () => {
        setAddData({
            ...addData,
            loading: true
        });
        // 
        let ops = [];
        addData.user_select.map((e) => {
            e.ops.map((e) => {
                ops.push({
                    op_id: e.op_id,
                    op_positivity: e.allow ? 1 : 0
                })
            })
        })

        if (addData.user_access_key && addData.user_access_key.length > 0) {
            let r_r = /\{.*\}/.exec(addData.user_access_key);
            if (r_r && r_r.length > 0) {
                toast("请先替换关系变量:" + r_r[0])
                setAddData({
                    ...addData,
                    loading: false
                })
                return
            }
        }



        let doAction;
        if (rowData && rowData.role && rowData.role.id > 0) {
            return roleEdit({
                role_id: rowData.role.id,
                name: addData.res_name,
                user_range: addData.user_range,
                role_op_range: addData.res_range,
                priority: addData.priority,
                relation_key: addData.user_access_key,
                tags: addData.tags,
                role_ops: ops
            }).then((data) => {
                if (data.status) {
                    onSave(rowData.role.id, addData.res_name, addData.user_range)
                    toast("已保存")
                } else {
                    toast(data.message)
                }
                setAddData({
                    ...addData,
                    loading: false
                });
                return data
            })
        } else {
            doAction = roleAdd({
                user_id: initData.user_id,
                name: addData.res_name,
                user_range: addData.user_range,
                role_op_range: addData.res_range,
                priority: addData.priority,
                relation_key: addData.user_access_key,
                tags: addData.tags,
                role_ops: ops,
            }).then((data) => {
                if (data.status) {
                    onSave(data.id, addData.res_name, addData.user_range)
                }
                if (!data.status) {
                    toast(data.message)
                    setAddData({
                        ...addData,
                        loading: false
                    });
                } else {
                    toast("已添加")
                    setAddData({
                        ...initAddData,
                        loading: false
                    });
                }
                return data
            })
        }
        return doAction;
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
                {title}
            </Typography>
            <Divider variant="middle" />
            {
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
                            <TextField
                                disabled={addData.loading}
                                label="名称"
                                variant="outlined"
                                name="name"
                                size="small"
                                value={addData.res_name}
                                onChange={(e) => {
                                    setAddData({
                                        ...addData,
                                        res_name: e.target.value,
                                    })
                                }}
                                sx={{
                                    width: 1,
                                    paddingBottom: 2
                                }}
                                required
                            />
                            <InputTagSelect
                                disabled={addData.loading}
                                name="ddd"
                                size="small"
                                value={addData.tags}
                                options={tags_options}
                                onChange={(value) => {
                                    setAddData({
                                        ...addData,
                                        tags: value
                                    })
                                }}
                            />
                            <SliderInput fullWidth
                                sx={{
                                    width: 1,
                                    mb: 2,
                                    mt: 2,
                                    padding: "0 16px",
                                    textAlign: "center"
                                }}
                                label="优先级"
                                loading={addData.loading}
                                value={addData.priority}
                                onChange={(e) => {
                                    setAddData({
                                        ...addData,
                                        priority: e.target.value || 100
                                    })
                                }}
                            />
                            <FormControl fullWidth sx={{
                                width: 1,
                                paddingBottom: 2
                            }}>
                                <InputLabel size="small" id="user-select-label">用户范围</InputLabel>
                                <Select
                                    disabled={addData.loading || !!rowData?.role}
                                    size="small"
                                    labelId="user-select-label"
                                    id="user-select"
                                    label="用户范围"
                                    value={addData.user_range}
                                    onChange={(e) => {
                                        setAddData({
                                            ...addData,
                                            user_range: e.target.value
                                        });
                                    }}
                                >
                                    {
                                        initData.user_range.map((item) => {
                                            if (rowData?.role) {
                                                if (rowData.role.user_range == 4 && item.key != 4) return;
                                                if (rowData.role.user_range != 4 && item.key == 4) return;

                                            }
                                            return <MenuItem key={`res_range_${item.key}`} value={item.key}>{item.name}</MenuItem>
                                        }).filter((e) => { return e })
                                    }
                                </Select>
                                {
                                    addData.user_range == 3 && !rowData?.role ? <FormHelperText >可在下一步添加指定用户</FormHelperText> : null
                                }
                            </FormControl>
                            {addData.user_range == 4 ?
                                <FormControl fullWidth sx={{
                                    width: 1,
                                    paddingBottom: 2
                                }}>
                                    <UserRoleRelationSet
                                        label="选择关系"
                                        size="small"
                                        sx={{
                                            width: 1
                                        }}
                                        options={initData.options_relation ?? []}
                                        disabled={addData.loading || !!rowData?.role}
                                        userId={userId}
                                        value={addData.user_access_key}
                                        onFinish={(value) => {
                                            setAddData({
                                                ...addData,
                                                user_access_key: value
                                            });
                                        }} />
                                </FormControl> : null}
                            <FormControl fullWidth sx={{
                                width: 1,
                                paddingBottom: 1
                            }}>
                                <InputLabel size="small" id="res-select-label">资源范围</InputLabel>
                                <Select
                                    disabled={addData.loading}
                                    size="small"
                                    labelId="res-select-label"
                                    id="res-select"
                                    label="资源范围"
                                    value={addData.res_range}
                                    onChange={(e) => {
                                        setAddData({
                                            ...addData,
                                            res_range: e.target.value
                                        });
                                    }}
                                >
                                    {
                                        initData.res_range.map((item) => {
                                            return <MenuItem key={`res_range_${item.key}`} value={item.key}>{item.name}</MenuItem>
                                        })
                                    }
                                </Select>
                            </FormControl>
                            {addData.res_range == 1 ? <UserResSelect
                                userId={initData.user_id}
                                disabled={addData.loading}
                                value={addData.user_select}
                                onChange={(value) => {
                                    setAddData({
                                        ...addData,
                                        user_select: value
                                    });
                                }}
                                size="small"
                                sx={{
                                    p: 1,
                                    width: "100%",
                                }} /> : null}
                        </Grid>
                        <Grid item xs={10}>
                            <LoadingButton sx={{
                                width: 1,
                                mb: 3,
                                mt: 2,
                            }}
                                variant="contained"
                                loading={addData.loading}
                                disabled={addData.loading}
                                onClick={() => {
                                    submitAction()
                                }}
                            >{rowData?.role?.id > 0 ? `保存` : `添加`}</LoadingButton>
                        </Grid>
                    </Grid>
                </Form >}
        </Fragment>)
}



//角色关联用户弹出页面
function UserRoleListUser(props) {
    const { roleId, roleName } = props;
    const defTimeout = dayjs((new Date()).getTime() + 1000 * 3600 * 24 * 30);
    const [userDataInput, setUserDataInput] = useState({
        show: false,
        op_user_id: 0,
        input_user_id: 0,
        timeout: defTimeout,
        need_timeout: false,
        add_loading: false,
        add_error: '',
    });
    const [userDataParam, setUserDataParam] = useState({
        op_user_id: 0,
        page: 0,
        page_size: 10
    });
    const [userData, setUserData] = useState({
        loading: false,
        rows: [],
        rows_total: 0,
        error: null,
    })


    const loadUserData = () => {
        setUserData({
            ...userData,
            loading: true
        })
        roleListUser({
            role_id: roleId,
            op_user_id: userDataParam.op_user_id,
            page: userDataParam.page,
            page_size: userDataParam.page_size
        }).then((data) => {
            if (!data.status) {
                setUserData({
                    ...userData,
                    error: data.message,
                    loading: false
                })
                return;
            }
            setUserData({
                ...userData,
                rows: data.data[roleId] ?? [],
                rows_total: data.total || 0,
                loading: false,

            })
            if (parseInt(data.total) == 0) {
                setUserDataInput({
                    ...userDataInput,
                    input_user_id: userDataParam.op_user_id,
                    show: true,
                    need_timeout: false
                })
            }
        })
    }

    useEffect(() => {
        loadUserData();
    }, [userDataParam])

    const columns = [
        {
            label: "用户ID",
            style: { width: 100 },
            align: "right",
            field: "user_id",
        },
        {
            label: "有效期",
            style: { width: 170 },
            align: "left",
            render: (row) => {
                let showime = showTime(row.timeout, "长期有效")
                let time = (new Date()).getTime() / 1000;
                if (row.timeout > 0 && (row.timeout) < time) {
                    return <ItemTooltip
                        placement="top"
                        title={`此用户已过期`}>
                        <div style={{ color: "#f00" }}>{showime}</div>
                    </ItemTooltip>
                } else {
                    return showime
                }
            }
        },
        {
            label: "绑定时间",
            style: { width: 170 },
            align: "left",
            render: (row) => {
                return showTime(row.change_time, "未知")
            }
        },
        {
            label: "操作",
            align: "center",
            render: (row) => {
                return <Fragment>
                    <IconButton onClick={() => {
                        setUserData({
                            ...userData,
                            rows: [],
                            rows_total: 0
                        });
                        setUserDataInput({
                            ...userDataInput,
                            input_user_id: row.user_id,
                            op_user_id: 0,

                            need_timeout: row.timeout > 0,
                            timeout: row.timeout > 0 ? dayjs(row.timeout * 1000) : defTimeout,
                            show: true
                        })
                    }} size='small'>
                        <EditIcon fontSize='small' />
                    </IconButton>
                    <ConfirmButton
                        message={`确定要移除该角色下的用户ID [${row.user_id}] ?`}
                        onAction={() => {
                            return roleDeleteUser({
                                roleId: roleId,
                                op_user_id: row.user_id
                            }).then((data) => {
                                if (!data.status) return data;
                                setUserDataParam({
                                    ...userDataParam,
                                    op_user_id: 0
                                });
                                return data;
                            })
                        }}
                        renderButton={(props) => {
                            return <IconButton {...props} size='small'>
                                <DeleteIcon fontSize='small' />
                            </IconButton>
                        }} />
                </Fragment >
                    ;
            }
        },
    ];
    let LoadError = function (props) {
        const { error, ...other } = props;
        return <Box {...other}>
            <Alert severity="error">{error}</Alert>
        </Box>
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
                角色<span style={{
                    background: "#eee",
                    margin: "0 0px 0 1px",
                    padding: "0 0 0 5px",
                    color: "#000",
                    display: "inline-block",
                    textAlign: "center"
                }}>{roleName}</span>
                <span style={{
                    background: "#eee",
                    margin: "0px 3px 0 1px",
                    padding: "0px 3px 0px 5px",
                    color: "#999",
                    display: "inline-block",
                    textAlign: "center"
                }}>{roleId}</span>绑定用户
            </Typography>
            <Divider variant="middle" />
            <Grid
                sx={{
                    mt: 3,
                }}
                container
                justifyContent="center"
                alignItems="center"
            >
                <Grid item xs={11}>
                    <Form method="post" >
                        <Grid
                            container item
                            justifyContent="center"
                            alignItems="center"
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}

                        >
                            <Grid item xs={3.6}>
                                <TextField
                                    sx={{
                                        width: 1,
                                    }}
                                    label="用户ID"
                                    variant="outlined"
                                    name="name"
                                    size="small"
                                    value={userDataInput.op_user_id > 0 ? userDataInput.op_user_id : ''}
                                    onChange={(e) => {
                                        let value = (e.target.value + '').replace(/[^0-9]+/, '');
                                        setUserDataInput({
                                            ...userDataInput,
                                            op_user_id: value
                                        })
                                    }}
                                    disabled={userDataInput.add_loading || userData.loading}
                                    required
                                />
                            </Grid>
                            <Grid item xs={2.8} sx={{ pl: 1 }}>
                                <LoadingButton
                                    loading={userData.loading}
                                    disabled={userDataInput.add_loading || userData.loading}
                                    variant="outlined"
                                    size="medium"
                                    startIcon={<SearchIcon />}
                                    sx={{ pd: 1, pt: 1, width: 1 }}
                                    onClick={() => {
                                        setUserDataParam({
                                            ...userDataParam,
                                            op_user_id: userDataInput.op_user_id
                                        })
                                    }}
                                >
                                    查找
                                </LoadingButton>
                            </Grid>
                            <Grid item xs={2.8} sx={{ pl: 1 }}>
                                <LoadingButton
                                    loading={userDataInput.add_loading}
                                    disabled={userDataInput.add_loading}
                                    variant="outlined"
                                    size="medium"
                                    startIcon={<AddCircleOutline />}
                                    sx={{ pd: 1, pt: 1, width: 1 }}
                                    onClick={() => {
                                        setUserData({
                                            ...userData,
                                            rows: [],
                                            rows_total: 0
                                        });
                                        setUserDataInput({
                                            ...userDataInput,
                                            input_user_id: 0,
                                            op_user_id: 0,
                                            need_timeout: false,

                                            show: true
                                        })
                                    }}
                                >
                                    新增
                                </LoadingButton>
                            </Grid>
                            <Grid item xs={2.8} sx={{ pl: 1 }}>
                                <LoadingButton
                                    loading={userData.loading}
                                    disabled={userDataInput.add_loading || userData.loading}
                                    variant="outlined"
                                    size="medium"
                                    startIcon={<AllOutIcon />}
                                    sx={{ pd: 1, pt: 1, width: 1 }}
                                    onClick={() => {
                                        setUserDataInput({
                                            ...userDataInput,
                                            op_user_id: ''
                                        })
                                        setUserDataParam({
                                            ...userDataParam,
                                            op_user_id: 0
                                        })
                                    }}
                                >
                                    全部
                                </LoadingButton>
                            </Grid>

                        </Grid>

                    </Form>
                </Grid>
                <Grid item xs={11} sx={{ mb: 3 }}>

                    {userData.error ?
                        <LoadError error={userData.error} /> :
                        (userDataInput.show && userData.rows.length == 0) ?
                            <Fragment>
                                <Paper sx={{ p: 2 }}>
                                    {userDataParam.op_user_id ? <BaseTableNoRows msg={`当前角色未添加用户ID ${userDataParam.op_user_id} ,可以尝试添加`} /> : null}

                                    {userDataInput.add_error ? <LoadError sx={{ mb: 1 }} error={userDataInput.add_error} /> : null}
                                    <LocalizationProvider dateAdapter={AdapterDayjs}>

                                        <Form>

                                            <TextField
                                                sx={{
                                                    width: 1,
                                                }}
                                                label="用户ID"
                                                variant="outlined"
                                                name="name"
                                                size="small"
                                                value={userDataInput.input_user_id > 0 ? userDataInput.input_user_id : ''}
                                                onChange={(e) => {
                                                    let value = (e.target.value + '').replace(/[^0-9]+/, '');
                                                    setUserDataInput({
                                                        ...userDataInput,
                                                        input_user_id: value
                                                    })
                                                }}
                                                disabled={userDataInput.add_loading || userData.loading}
                                                required
                                            />
                                            <FormControl fullWidth sx={{ mt: 1 }}>
                                                <FormLabel style={{
                                                    position: "absolute",
                                                    transform: "translate(10px, -5px) scale(0.75)"
                                                }}>有效期</FormLabel>
                                                <Box className='MuiInputBase-root MuiOutlinedInput-root MuiInputBase-colorPrimary MuiInputBase-formControl MuiInputBase-sizeSmall'
                                                    style={{
                                                        borderRadius: "4px",
                                                        marginBottom: "4px"
                                                    }}>
                                                    <fieldset style={{
                                                        textAlign: "left",
                                                        position: "absolute",
                                                        bottom: 0,
                                                        right: 0,
                                                        top: "-5px",
                                                        left: 0,
                                                        margin: 0,
                                                        padding: "0 8px",
                                                        pointerEvents: "none",
                                                        borderRadius: "inherit",
                                                        borderStyle: "solid",
                                                        borderWidth: "1px ",
                                                        overflow: "hidden",
                                                        borderColor: " rgba(0, 0, 0, 0.23)",
                                                    }} className="MuiOutlinedInput-notchedOutline "><legend style={{
                                                        visibility: "hidden"
                                                    }} ><span>有效期</span></legend></fieldset>
                                                </Box>


                                                <FormGroup>
                                                    <FormControlLabel
                                                        label="设置有效期"

                                                        sx={{ m: 1, mt: 2, color: "#666" }}
                                                        control={
                                                            <Switch
                                                                value={userDataInput.need_timeout}
                                                                onChange={(e) => {
                                                                    setUserDataInput({
                                                                        ...userDataInput,
                                                                        need_timeout: e.target.checked,

                                                                    })
                                                                }} />
                                                        }
                                                    />
                                                    {
                                                        !userDataInput.need_timeout ? <FormHelperText sx={{ mb: 1 }} >不设置有效期为长期有效</FormHelperText> : null
                                                    }
                                                </FormGroup>
                                                <FormGroup>

                                                    {userDataInput.need_timeout ? <DesktopDateTimePicker
                                                        inputFormat="YYYY-MM-DD hh:mm:ss"
                                                        label="过期时间"
                                                        minDateTime={dayjs(new Date())}
                                                        variant="outlined"
                                                        size="small"
                                                        onChange={(value) => {
                                                            setUserDataInput({
                                                                ...userDataInput,
                                                                timeout: value
                                                            })
                                                        }}
                                                        value={userDataInput.timeout}
                                                        disabled={userDataInput.add_loading}
                                                        renderInput={(params) => <TextField
                                                            size="small" name="timeout" {...params} sx={{
                                                                m: 2,
                                                                mt: 1
                                                            }} />}
                                                    /> : null}
                                                </FormGroup>
                                            </FormControl>




                                            <LoadingButton
                                                loading={userDataInput.add_loading}
                                                disabled={userDataInput.add_loading}
                                                variant="outlined"
                                                size="medium"
                                                startIcon={<AddCircleOutlineIcon />}
                                                sx={{ width: 1, mt: 2 }}
                                                onClick={() => {
                                                    setUserDataInput({
                                                        ...userDataInput,
                                                        add_loading: true
                                                    })
                                                    roleAddUser({
                                                        roleId: roleId,
                                                        op_user_id: userDataInput.input_user_id,
                                                        timeout: userDataInput.need_timeout ? userDataInput.timeout.unix() : 0
                                                    }).then((data) => {
                                                        if (!data.status) {
                                                            setUserDataInput({
                                                                ...userDataInput,
                                                                add_loading: false,
                                                                add_error: data.message
                                                            })
                                                        } else {
                                                            setUserDataInput({
                                                                ...userDataInput,
                                                                add_loading: false,
                                                                op_user_id: userDataInput.input_user_id
                                                            })
                                                            setUserDataParam({
                                                                ...userDataParam,
                                                                op_user_id: userDataInput.input_user_id
                                                            });
                                                        }
                                                    })
                                                }}
                                            >
                                                添加用户ID到角色
                                            </LoadingButton>
                                        </Form>

                                    </LocalizationProvider>
                                </Paper>
                            </Fragment> :
                            <BaseTablePage
                                rows={userData.rows ?? []}
                                columns={columns}
                                count={userData.rows_total ?? 0}
                                page={userDataParam.page}
                                rowsPerPage={userDataParam.page_size}
                                onPageChange={(e, newPage) => {
                                    setUserDataParam({
                                        ...userDataParam,
                                        page: newPage
                                    })
                                }}
                                onRowsPerPageChange={(e) => {
                                    setUserDataParam({
                                        ...userDataParam,
                                        page_size: e.target.value,
                                        page: 0
                                    })
                                }}
                                loading={userData.loading}
                            />
                    }
                </Grid>
            </Grid >
        </Fragment >)
}


//角色记录块
function UserRoleRow(props) {
    const { roleData, tagData, opData, columns, onTagClick } = props;

    let isTags = (tagData ?? []).length > 0;
    let isOpData;
    if ((opData ?? []).length > 0) {
        isOpData = [];
        opData.map((op) => {
            if (!isOpData[op.res.id]) isOpData[op.res.id] = {
                res: op.res,
                res_op: [],
            }
            isOpData[op.res.id].res_op.push({
                res_op: op.res_op,
                role_op: op.role_op,

            });
        })
    }

    return <ListItem
        disablePadding
        sx={{
            borderBottom: "1px solid #ddd",
            "&:hover": {
                background: "#f9f9f9"
            }
        }}
    >
        <Table sx={{ width: 1, borderBottom: "0px", }}>

            <TableBody>
                <BaseTableBodyRow
                    row={roleData}
                    columns={columns}
                    cellProps={{ style: { borderBottom: "1px solid #f0f0f0" } }}
                />

                {isOpData ? <TableRow >
                    <TableCell colSpan={7} style={{ padding: 8, paddingBottom: 0, paddingTop: 0, borderBottom: "1px solid #f0f0f0" }}>
                        <Grid container >
                            <Grid sx={{ width: 80, lineHeight: "52px", fontWeight: 500, fontSize: "0.875rem", textAlign: "right", mr: 1 }}>绑定资源</Grid>
                            <Grid sx={{ flexGrow: 1, width: "70%", marginLeft: "8px" }}>
                                {
                                    isOpData.map((op, i) => {
                                        let ops = op.res_op.map((e) => {
                                            return {
                                                allow: e.role_op.positivity == 1,
                                                op_name: e.res_op.name,
                                                op_key: e.res_op.op_key,
                                                time: e.res_op.change_time
                                            }
                                        })
                                        return <RoleResOpGroupItem
                                            key={`key-access-item-${i}`}
                                            resName={op.res.name}
                                            resKey={op.res.res_key}
                                        >
                                            {
                                                ops.map((op, i) => {
                                                    return <RoleResOpItem
                                                        key={`op-item-${i}`}
                                                        allow={op.allow}
                                                        opName={op.op_name}
                                                        opKey={op.op_key}
                                                        tips={`添加于:${showTime(op.time)}`}
                                                    />
                                                })}
                                        </RoleResOpGroupItem>;
                                    })
                                }
                            </Grid>
                        </Grid>
                    </TableCell>
                </TableRow> : null}
                {isTags ? <TableRow>
                    <TableCell colSpan={7} style={{ padding: 4, borderBottom: "1px solid #f0f0f0" }}>
                        <Grid container>
                            <Grid sx={{ width: 80, lineHeight: "52px", fontWeight: 500, fontSize: "0.875rem", textAlign: "right", mr: 1 }}>标签</Grid>
                            <Grid sx={{ flexGrow: 1, color: "#333", paddingTop: "2px", paddingLeft: "8px", width: "70%" }}>
                                {tagData.map((tag) => {
                                    return <UserTags
                                        onClick={() => {
                                            onTagClick(tag.name)
                                        }}
                                        name={tag.name}
                                        key={`res-tag-${tag.id}`}
                                        sx={{ m: 1, ml: 0 }}
                                        tips={`添加于:${showTime(tag.change_time, "未知")}`}
                                    />
                                })}
                            </Grid>
                        </Grid>
                    </TableCell>
                </TableRow> : null}
            </TableBody>
        </Table>
    </ListItem>;
}


function UserRoleRelationFind(props) {
    const { userId, value, ...params } = props;
    const { toast } = useContext(ToastContext);
    const [relationData, setRelationData] = useState({
        data: [],
        abort: null,
        timeout: null,
        now_value: "",
    })
    useEffect(() => {
        if (relationData.timeout) {
            clearTimeout(relationData.timeout)
        }
        if (relationData.abort) {
            relationData.abort.abort()
        }
        setRelationData({
            ...relationData,
            timeout: setTimeout(() => {
                let abort = new AbortController();
                setRelationData({
                    ...relationData,
                    abort: abort
                })
                roleRelationData({
                    user_id: parseInt(userId),
                    relation_prefix: relationData.now_value,
                    page: 0,
                    page_size: 20,
                    count_num: false
                }, {
                    signal: abort.signal
                }).then((data) => {
                    if (!data.status) {
                        if (!(data.message + '').indexOf("cancel")) {
                            toast("关系数据获取异常:" + data.message)
                        }
                    } else {
                        setRelationData({
                            ...relationData,
                            abort: null,
                            timeout: null,
                            data: data.data
                        })
                    }
                })
            }, 800)
        })
    }, [relationData.now_value])
    return <UserRoleRelationInput
        {...params}
        value={value ?? ""}
        options={relationData.data}
        onChange={(e) => {
            let val = e.target.value;
            setRelationData({
                ...relationData,
                now_value: val.replace(/^\s+/, '').replace(/\s+$/, '')
            })
        }} />
}



//角色管理页面
export function UserRolePage(props) {
    const { userId } = props;
    //URL参数
    const [searchParam, setSearchParam] = useSearchChange({
        tag: "",
        role_id: "",
        role_name: "",
        user_range: "",
        res_range: "",
        relation_prefix: "",
        page: 0,
        page_size: 10,
    });


    //过滤组件数据
    const [filterData, setfilterData] = useState({
        tag: '',
        role_id: '',
        role_name: '',
        user_range: "",
        res_range: "",
        relation_prefix: "",
    })

    useEffect(() => {
        setfilterData({
            ...filterData,
            tag: searchParam.get("tag"),
            role_id: searchParam.get("role_id"),
            role_name: searchParam.get("role_name"),
            user_range: searchParam.get("user_range"),
            res_range: searchParam.get("res_range"),
            relation_prefix: searchParam.get("relation_prefix")
        })
    }, [searchParam])
    //初始化数据
    const [pageInitData, setPageInitData] = useState({
        user_id: userId,
        res_range: [],
        user_range: [],
        options_relation: [],
    }, [props.userId])
    const [pageRowData, setPageRowData] = useState({
        rows: [],
        rows_total: 0,
        rows_error: null,
        rows_loading: true,
    })

    const [pageTagData, setPageTagData] = useState({
        tag_rows: [],
        tag_rows_error: null,
        tag_rows_loading: true,
        load_user_id: false
    })


    const loadRoleData = () => {
        let set_data = { ...pageTagData }
        let loadRow = () => {
            setPageRowData({
                ...pageRowData,
                rows_loading: true
            })
            roleListData({
                user_id: userId,
                user_range: searchParam.get("user_range"),
                res_range: searchParam.get("res_range"),
                relation_prefix: searchParam.get("relation_prefix"),
                tag: searchParam.get("tag"),
                role_id: searchParam.get("role_id"),
                role_name: searchParam.get("role_name"),
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

        roleOptions({
            user_id: userId
        }).then((data) => {
            if (!data.status) {
                setPageTagData({
                    ...set_data,
                    tag_rows_error: data.message,
                    tag_rows_loading: false
                })
                return
            }
            setPageInitData({
                ...pageInitData,
                res_range: data.res_range ?? [],
                user_range: data.user_range ?? [],
                options_relation: data.relation_tpl ?? [],
            });
            roleTags({
                user_id: parseInt(userId)
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
                    tag_rows: data.data ?? [],
                    tag_rows_loading: false
                })
                loadRow();
            })
        });
    }

    useEffect(loadRoleData, [searchParam])

    const [showPage, setShowPage] = useState({
        show: false,
        page: null,
        role: {}
    });

    let rpage;
    switch (showPage.page) {
        case "add":
            rpage = <UserRoleAdd
                userId={userId}
                onSave={(id, name, user_range) => {
                    setSearchParam({
                        role_id: id,
                        page: 0,
                        tag: '',

                        role_name: '',
                        user_range: "",
                        res_range: "",
                        relation_prefix: "",
                    }, loadRoleData)
                    if (user_range == 3) {
                        setShowPage({
                            show: true,
                            role: {
                                name: name,
                                id: id
                            },
                            page: "user"
                        });
                    }
                }}
                title="创建角色"
                tags={pageTagData.tag_rows}
                initData={pageInitData}
            />
            break;
        case "edit":
            rpage = <UserRoleAdd
                userId={userId}
                title="编辑角色"
                onSave={(id) => {
                    setSearchParam({
                        role_id: id,
                        page: 0,
                        tag: '',

                        role_name: '',
                        user_range: "",
                        res_range: "",
                        relation_prefix: "",
                    }, loadRoleData)
                }}
                tags={pageTagData.tag_rows}
                initData={pageInitData}
                rowData={showPage.role}
            />
            break;
        case "user":
            rpage = <UserRoleListUser roleId={showPage.role.id} roleName={showPage.role.name} />
            break;
    }

    const columns = [
        {
            field: 'id',
            label: 'ID',
            align: "right",
            style: { width: 100, textAlign: "right", borderBottom: "1px solid #f3f3f3" }
        },
        {
            field: "name",
            label: '角色名',
            style: { width: 150, textAlign: "left", borderBottom: "1px solid #f3f3f3" }
        },
        {
            label: '用户范围',
            style: { width: 150, textAlign: "left", borderBottom: "1px solid #f3f3f3" },
            render: (row) => {
                let item = pageInitData.user_range.find((item) => {
                    return item.key == row.user_range
                })
                if (row.user_range == 3) {
                    return <Fragment>
                        <span style={{ marginRight: "3px" }}> {item ? item.name : row.user_range}</span>
                        <Button onClick={() => {
                            let pageItem = pageRowData.rows.find((e) => {
                                return e.role.id == row.id
                            })
                            if (!pageItem) {
                                return;
                            }
                            setShowPage({
                                show: true,
                                role: pageItem.role,
                                page: "user"
                            })
                        }}>查看({row.group_user ?? 0})</Button>
                    </Fragment>
                } else if (row.user_range == 4) {
                    return <Fragment>
                        <span> {item ? item.name : row.user_range}</span>
                        <b style={{
                            fontWeight: 700,
                            background: "#bebebe",
                            marginLeft: 5,
                            padding: "5px 9px",
                            borderRadius: 3,
                            color: " #fff"
                        }}>{row.relation_key}</b>
                    </Fragment>
                } else {
                    return item ? item.name : row.user_range;
                }
            }
        },
        {
            label: '资源范围',
            style: { width: 120, textAlign: "left", borderBottom: "1px solid #f3f3f3" },
            render: (row) => {
                let item = pageInitData.res_range.find((item) => {
                    return item.key == row.res_op_range
                })
                return item ? item.name : row.res_op_range;
            }
        },
        {
            field: 'priority',
            label: '优先级',
            style: { width: 80, textAlign: "center", borderBottom: "1px solid #f3f3f3" }
        },
        {
            field: 'change_user_id',
            label: '添加用户ID',
            style: { width: 100, textAlign: "center", borderBottom: "1px solid #f3f3f3" }
        },
        {
            label: '更新时间',
            style: { width: 180, textAlign: "left", borderBottom: "1px solid #f3f3f3" },
            render: (row) => {
                return showTime(row.change_time, "未知")
            }
        },
        {
            style: { width: 125, textAlign: "center", borderBottom: "1px solid #f3f3f3", borderLeft: "1px solid #f3f3f3" },
            label: '操作',
            rowSpan: 3,
            render: (row) => {
                return <Fragment>
                    <IconButton onClick={() => {
                        let pageItem = pageRowData.rows.find((e) => {
                            return e.role.id == row.id
                        })
                        if (!pageItem) {
                            return;
                        }
                        setShowPage({
                            show: true,
                            role: pageItem,
                            page: "edit"
                        })
                    }}><EditIcon fontSize="small" /></IconButton>
                    <ConfirmButton
                        message={`确定要删除角色 [${row.name}] 吗?`}
                        onAction={() => {
                            return roleDelete({
                                role_id: row.id
                            }).then((data) => {
                                if (!data.status) return data;
                                let rows = pageRowData.rows.filter((item) => {
                                    if (item.role.id != row.id) return item;
                                })
                                setPageRowData({
                                    ...pageRowData,
                                    rows: rows,
                                    rows_total: pageRowData.rows_total - 1
                                })
                                if (rows.length == 0) {
                                    setSearchParam({
                                        tag: "",
                                        role_id: "",
                                        role_name: "",
                                        page: 0,
                                    }, loadRoleData)
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
            open={showPage.show}
            onClose={() => {
                setShowPage({
                    page: null,
                    show: false
                })
            }}
        >
            <Box
                sx={{ width: 600 }}
            >
                {rpage}
            </Box>
        </Drawer>
        <Paper
            component="form"
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1, minWidth: 700 }}
        >
            <FormControl fullWidth sx={{ width: 110, mr: 1 }}>
                <InputLabel size="small" id="res-select-label">资源范围</InputLabel>
                <Select
                    disabled={pageTagData.tag_rows_loading}
                    size="small"
                    labelId="res-select-label"
                    id="res-select"
                    label="资源范围"
                    value={filterData.res_range ?? ''}
                    onChange={(event) => {
                        setfilterData({
                            ...filterData,
                            res_range: event.target.value
                        })
                    }}
                >
                    <MenuItem value="">
                        全部
                    </MenuItem>
                    {
                        pageInitData.res_range.map((item) => {
                            return <MenuItem key={`res_range_${item.key}`} value={item.key}>{item.name}</MenuItem>
                        })
                    }
                </Select>
            </FormControl>
            <FormControl sx={{ width: 110, mr: 1 }}>
                <InputLabel size="small" id="user-select-label">用户范围</InputLabel>
                <Select

                    disabled={pageTagData.tag_rows_loading}
                    size="small"
                    labelId="user-select-label"
                    id="user-select"
                    label="用户范围"
                    value={filterData.user_range ?? ''}
                    onChange={(e) => {
                        setfilterData({
                            ...filterData,
                            user_range: e.target.value,
                            relation_prefix: e.target.value == 4 ? filterData.relation_prefix : ""
                        })
                    }}
                >
                    <MenuItem value="">
                        全部
                    </MenuItem>
                    {
                        pageInitData.user_range.map((item) => {
                            return <MenuItem key={`res_range_${item.key}`} value={item.key}>{item.name}</MenuItem>
                        })
                    }
                </Select>

            </FormControl>
            {filterData.user_range == 4 ? <FormControl sx={{ mr: 1 }} size="small"  >
                <UserRoleRelationFind

                    label="关系名"
                    size="small"
                    sx={{
                        width: 130
                    }}
                    userId={userId}
                    value={filterData.relation_prefix}
                    onFinish={(value) => {
                        setfilterData({
                            ...filterData,
                            relation_prefix: value,
                        });
                    }} />
            </FormControl> : null}

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
            <FormControl sx={{ mr: 1 }} size="small"  >
                <ClearTextField
                    sx={{ width: 95 }}
                    variant="outlined"
                    label={`角色ID`}
                    type="text"
                    name="code"
                    size="small"
                    value={filterData.role_id ?? ''}
                    onChange={(e, nval) => {
                        let value = (nval + '').replace(/[^0-9]+/, '');
                        setfilterData({
                            ...filterData,
                            role_id: value
                        });
                    }}
                />
            </FormControl>
            <FormControl sx={{ mr: 1 }} size="small"  >
                <ClearTextField
                    sx={{ width: 105 }}
                    variant="outlined"
                    label={`角色名称`}
                    type="text"
                    name="name"
                    size="small"
                    value={filterData.role_name ?? ''}
                    onChange={(e, nval) => {
                        setfilterData({
                            ...filterData,
                            role_name: nval
                        });
                    }}
                />
            </FormControl>
            <Button

                variant="outlined"
                size="medium"
                startIcon={<SearchIcon />}
                sx={{ mr: 1, p: "7px 15px", minWidth: 85 }}
                onClick={() => {
                    setSearchParam({
                        ...filterData,
                        page: 0
                    }, loadRoleData)
                }}
            >
                过滤
            </Button>
            <Button
                variant="outlined"
                size="medium"
                startIcon={<AddCircleOutlineIcon />}
                sx={{ mr: 1, p: "7px 15px", minWidth: 120 }}
                onClick={() => {
                    setShowPage({
                        show: true,
                        page: "add"
                    })
                }}
            >
                新增角色
            </Button>
        </Paper>
        <Box sx={{ border: "1px solid #ddd", borderRadius: 1 }}>
            {pageTagData.tag_rows_loading ?
                <Fragment>
                    <Progress />
                </Fragment> :
                pageTagData.tag_rows_error ?
                    <Alert severity="error">{pageTagData.tag_rows_error}</Alert> :
                    (pageRowData.rows && pageRowData.rows.length == 0) ? <BaseTableNoRows /> :
                        <Fragment>
                            <Table sx={{ mb: 0 }}
                                style={{ borderBottom: "2px solid #ccc" }}>
                                <BaseTableHead
                                    columns={columns}
                                />
                            </Table>
                            <List sx={{ pb: 0, mt: 0, pt: 0 }}>
                                {
                                    pageRowData.rows.map((item, i) => {
                                        return <UserRoleRow
                                            onTagClick={(tag) => {
                                                setSearchParam({
                                                    tag: tag
                                                }, loadRoleData)
                                            }}
                                            key={`key-access-item-${i}`}
                                            roleData={{
                                                ...item.role,
                                                group_user: item.users_group
                                            }}
                                            opData={item.ops}
                                            tagData={item.tags}
                                            columns={columns}
                                        />
                                    })
                                }
                            </List>
                            <Table>
                                <BaseTableFooter
                                    count={pageRowData.rows_total}
                                    page={parseInt(searchParam.get("page")) || 0}
                                    rowsPerPage={parseInt(searchParam.get("page_size")) || 10}
                                    onPageChange={(e, newPage) => {
                                        setSearchParam({

                                            page: newPage
                                        }, loadRoleData)
                                    }}
                                    onRowsPerPageChange={(e) => {
                                        setSearchParam({

                                            page_size: e.target.value,
                                            page: 0
                                        }, loadRoleData)
                                    }}
                                />
                            </Table></Fragment>
            }

        </Box>
    </Fragment >
}


UserRolePage.propTypes = {
    userId: PropTypes.number
};
