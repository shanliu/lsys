import RemoveCircleRoundedIcon from '@mui/icons-material/RemoveCircleRounded';
import { Chip, Typography } from '@mui/material';
import React from 'react';
import { ItemTooltip } from '../../library/tips';

//角色中显示的单个资源块
export function RoleResOpItem(props) {
    const { allow, opName, opKey, onDelete, tips } = props;
    let bg = allow ? "#d7ebff" : '#ffeeee';
    let item = (<div style={{ margin: 8, marginBottom: 0, borderRadius: 16, color: "#333", background: bg, padding: 6, display: "inline-flex" }}>
        <span style={{
            paddingRight: 8,
            paddingLeft: 8,

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
    const { name, opKey, onDelete, style,onClick } = props;
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
        <span onClick={(event)=>{onClick&&onClick(event,{
            name:name,
            opKey:opKey
        })}} style={{ paddingRight: 8, paddingLeft: 8, }}>{name}</span>
        <span onClick={(event)=>{onClick&&onClick(event,{
            name:name,
            opKey:opKey
        })}} style={{ paddingLeft: 12, paddingRight: 12, paddingBottom: 1, color: "#999", background: "#fff", borderRadius: 12 }}>{opKey}</span>
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
