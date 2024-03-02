import RemoveCircleRoundedIcon from '@mui/icons-material/RemoveCircleRounded';
import { Autocomplete, Box, Chip, TextField, Typography } from '@mui/material';
import React, { useContext, useEffect, useRef, useState } from 'react';
import { ItemTooltip } from '../../../library/tips';
import { searchType, userIdSearch, userSearch } from '../../../common/rest/user';
import { ToastContext } from '../../../common/context/toast';
//角色中显示的单个资源块
export function RoleResOpItem(props) {
    const { allow, opName, opKey, onDelete, tips } = props;
    let bg = allow ? "#d7ebff" : '#ffeeee';
    let item = (<div style={{ margin: 8, marginBottom: 0, borderRadius: 16, color: "#333", background: bg, padding: 6, display: "inline-flex" }}>
        <span style={{
            paddingRight: 8,
            paddingLeft: 8,
            paddingTop: 3,
            fontSize: " 0.9rem",
            color: "#333"
        }}>{opName}</span>
        <span style={{ paddingLeft: 12, marginRight: "4px", paddingRight: 12, paddingBottom: 1, color: "#999", background: "#fff", borderRadius: 12 }}>{opKey}</span>
        {onDelete ? <RemoveCircleRoundedIcon
            onClick={() => {
                onDelete(opKey);
            }}
            sx={{
                cursor: "pointer",
                margin: "3px", color: "#aaa", marginLeft: "5px",
                '&:hover': {
                    color: '#999'
                }
            }}
            fontSize='small'
        /> : null}
    </div>);


    const title = <span>{allow ? `允许 ${opName} 操作` : `禁止 ${opName} 操作`} {tips}</span>
    return <ItemTooltip title={title} placement="top">
        {item}
    </ItemTooltip>;
    return;
}

//角色中显示的资源块
export function RoleResOpGroupItem(props) {
    const { resName, resKey, children } = props;
    return <div>
        <div style={{
            border: " 1px solid #f0f0f0",
            borderRadius: "4px",
            marginBottom: "8px",
            marginTop: "8px",
        }}>
            <div style={{
                padding: "8px",
                borderBottom: " 1px dashed #f0f0f0",
                color: "#666"
            }}>
                <Typography
                    noWrap
                    sx={{
                        fontSize: "1rem",
                        fontWeight: 100,
                        letterSpacing: '.1rem',
                        color: 'inherit',
                        textDecoration: 'none',
                    }}
                >
                    <span>名称:</span>
                    <span style={{ fontWeight: 400, paddingRight: "8px", paddingLeft: "4px" }}>
                        {resName}
                    </span>
                    <span>标识:</span>
                    <span style={{ fontWeight: 400, paddingLeft: "4px" }}>
                        {resKey}
                    </span>
                </Typography>
            </div>
            <div style={{
                marginBottom: 8
            }}>
                {children}
            </div>
        </div>
    </div >
}

//资源操作元素显示
export function ResOpItem(props) {
    const { name, opKey, onDelete, style, onClick } = props;
    let delEl;
    if (onDelete) {
        delEl = <RemoveCircleRoundedIcon
            sx={{
                cursor: "pointer",
                margin: "3px", color: "#aaa", marginLeft: "5px",
                '&:hover': {

                    color: '#999'
                }
            }}
            onClick={() => {
                onDelete(opKey)
            }}
            fontSize='small'
        />
    }
    let item = <div style={{ marginLeft: 0, borderRadius: 16, color: "#333", background: "#eee", padding: 6, display: "inline-flex", ...style }}>
        <span onClick={(event) => {
            onClick && onClick(event, {
                name: name,
                opKey: opKey
            })
        }} style={{ paddingRight: 8, paddingLeft: 8, cursor: "default" }}>{name}</span>
        <span onClick={(event) => {
            onClick && onClick(event, {
                name: name,
                opKey: opKey
            })
        }} style={{ cursor: "default", paddingLeft: 12, paddingRight: 12, paddingBottom: 1, color: "#999", background: "#fff", borderRadius: 12 }}>{opKey}</span>
        {delEl}
    </div>;
    if (props.tips) {
        const tips = <span>{props.tips}</span>
        return <ItemTooltip title={tips} placement="top">
            {item}
        </ItemTooltip>;
    } else {
        return item;
    }
}

//用户标签显示
export function UserTags(props) {
    const { name, sx, tips, ...other } = props;
    let tag = <Chip sx={sx} label={name} {...other} />;
    if (tips && tips != '') {
        tag = <ItemTooltip placement="top" arrow title={tips}>{tag}</ItemTooltip>
    }
    return tag;
}


//用户搜索输入框

export function UserSearchInput(props) {
    const { label, enableUser, value, onSelect, disabled, ...params } = props;
    const { toast } = useContext(ToastContext);
    //todo 存在值时加载指定用户
    const [userData, setUserData] = useState({
        data: [],
        show_value: '',
        show_name: '',
        input_value: '',
        select_value: '',
        is_select: false,
        open: false,
        loading: false,
    })
    useEffect(() => {
        if (!value || value == '') return;
        if ((!userData.input_value || userData.input_value == '')
            || userData.select_value != value) {
            userIdSearch({
                user_id: value,
                opt: false,
            }).then((data) => {
                if (!data.status) {
                    toast("加载用户数据失败:" + data.message)
                    return;
                }
                if (!data.data.user) {
                    setUserData({
                        ...userData,
                        show_value: '',
                        input_value: ''
                    })
                    return;
                }
                let name = data.data.user.nickname;
                setUserData({
                    ...userData,
                    show_value: name,
                    input_value: name,
                    show_name: name
                })
            })
        }
    }, [props.value]);
    let ajaxReq = useRef(null)
    let ajaxTimeout = useRef(null)
    useEffect(() => {
        if (userData.is_select) return;
        if (ajaxTimeout.current) {
            clearTimeout(ajaxTimeout.current)
            ajaxTimeout.current = null;
        }
        if (ajaxReq.current) {
            ajaxReq.current.abort()
        }
        setUserData({
            ...userData,
            loading: true,
        })
        let timeout = setTimeout(() => {
            if (ajaxTimeout.current != timeout) return;
            ajaxReq.current = new AbortController();
            let param = {
                opt: false,
                more: false,
                key_word: userData.input_value,
                start_pos: '',
                end_pos: '',
                page_size: 25,
                enable_user: enableUser,
            }
            return userSearch(param, {
                signal: ajaxReq.current.signal
            }).then((data) => {
                ajaxReq.current = null;
                let setData = data.status && data.data && data.data.length > 0 ? data.data : [];
                setData = setData.map((e) => {
                    let cat = e.user.nickname;
                    let name = e.user.nickname
                    if (e.cat && e.cat.length) {
                        cat = e.cat[0].val
                        searchType.map((t) => {
                            if (t.key == e.cat[0].type) {
                                name = t.val + ":" + e.cat[0].val
                            }
                        })

                    }
                    return {
                        "input": cat,
                        "val": e.user.id,
                        "name": name
                    }
                })
                setUserData({
                    ...userData,
                    status: data.status ?? false,
                    message: data.message ?? '',
                    data: setData,
                    loading: false,
                })
            })
        }, 800);
        ajaxTimeout.current = timeout
    }, [userData.input_value])
    return <Autocomplete

        {...params}
        disabled={disabled}
        value={userData.show_value ?? ""}
        options={userData.data ?? []}
        getOptionLabel={(option) => {
            if (option && option.input) return option.input;
            return option
        }}
        isOptionEqualToValue={(a, b) => {
            if (!a) return b
            if (a.val == b) return a
        }}
        onChange={(_, v) => {
            let input = '';
            let val = '';
            let name = '';
            if (v) {
                val = v.val
                input = v.input
                name = v.name
            }
            if (val == '' || !val) {
                input = ''
                name = ''
            }
            setUserData({
                ...userData,
                open: false,
                show_value: input,
                show_name: name,
                select_value: val
            })
            onSelect(val)
        }}
        noOptionsText={userData.loading ? "数据加载中..." : "无匹配用户"}
        open={userData.open}
        renderInput={(params) => (
            <Box sx={{ position: "relative" }}>
                <TextField
                    {...params}
                    disabled={disabled}
                    label={label}
                    onChange={(e) => {
                        setUserData({
                            ...userData,
                            is_select: e.target.value == userData.input_value,
                            input_value: e.target.value
                        })
                    }}
                    variant="outlined"
                    onClick={(e) => {
                        setUserData({
                            ...userData,
                            open: disabled ? false : true,
                        })
                    }}
                    onFocus={(e) => {
                        setUserData({
                            ...userData,
                            open: disabled ? false : true,
                        })

                    }}
                    onBlur={(e) => {
                        let show_value = userData.show_value;
                        let show_name = userData.show_name;
                        if (value == '') {
                            show_value = ''
                            show_name = ''
                        }
                        setUserData({
                            ...userData,
                            open: false,
                            show_value: show_value,
                            show_name: show_name,
                        })
                    }}
                />
                {value ? <Box sx={{ zIndex: 1, position: "absolute", right: 30, top: 6, color: "#666", background: "#eee", borderRadius: '2px', padding: "1px 5px" }}>
                    <ItemTooltip title={userData.show_name} placement="top">
                        <span>ID:{value}</span>
                    </ItemTooltip></Box> : null}
            </Box>
        )}
    />;
}
