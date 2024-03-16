import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import CheckCircleOutlineIcon from '@mui/icons-material/CheckCircleOutline';
import { Box, Button, Chip, Divider, FormControl, FormLabel, Grid, TextField, Typography } from '@mui/material';
import React, { Fragment, useEffect, useState } from 'react';
import { ResOpItem } from '../../common/user';

export function ResKeyEditBox(props) {
    const { loading, value, onChange, ...other } = props;
    const [resData, setResData] = useState({
        vars: [],
        value: value
    });
    return <FormControl {...other}>
        <TextField
            label="资源KEY"
            variant="outlined"
            name="name"
            size="small"
            disabled={loading}
            value={value}
            onChange={(e) => {
                let val = e.target.value + '';
                let reg = RegExp(/\${([a-z0-9_]+)}/ig);
                let mt = val.match(reg);
                if (mt && mt.length) {
                    mt = mt.map((e) => {
                        return e.replace('${', '').replace('}', '')
                    })
                    mt = mt.filter((item, index) => {
                        return mt.indexOf(item) === index;
                    })
                } else { mt = [] }
                setResData({
                    value: val,
                    vars: mt
                })
                onChange && onChange(e)
            }}
            sx={{
                width: 1,

            }}
            required
        />
        <Box sx={{
            background: '#f9f9f9',
            padding: '8px'
        }}>
            {resData.vars.length > 0 ? <Fragment>
                <Typography variant="caption" display="block" gutterBottom>
                    已声明变量
                </Typography>
                <Box>
                    {resData.vars.map((e) => {
                        return <Chip key={`var-${e}`} fontSize="small" label={e} variant="outlined" sx={{ mr: 1 }} />
                    })}
                </Box>
            </Fragment> : <Typography variant="caption" display="block" gutterBottom>
                使用{"${}"}声明变量,例如:{"${var1}"}
            </Typography>}
        </Box>
    </FormControl>
}
export function ResEditBox(props) {
    const { opItems, loading, onChange, focused, ...other } = props;
    let init_res_data = {
        input_op_name: '',
        input_op_key: '',
    };
    let op_items = opItems ?? [];
    const [resData, setResData] = useState(init_res_data);
    return <FormControl {...other}>
        <FormLabel style={{
            position: "absolute",
            transform: "translate(0, -12px) scale(0.75)"
        }}>权限操作</FormLabel>
        <Box className='MuiInputBase-root MuiOutlinedInput-root MuiInputBase-colorPrimary MuiInputBase-formControl MuiInputBase-sizeSmall'
            style={{
                borderRadius: "4px"
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
            }} ><span>权限操作</span></legend></fieldset>
        </Box>
        <Box sx={{ mt: "5px" }}>
            {op_items.length == 0 ? <div style={{
                textAlign: "center",
                fontSize: "0.9rem",
                color: "#999",
                lineHeight: 3
            }}>请添加操作</div> : op_items.map((item, i) => {
                return <ResOpItem
                    tips="点击修改"
                    key={`add-res-key-${i}`}
                    style={item.op_key == resData.input_op_key ? { margin: "8px 8px 0px 0px", background: "#e8ecff" } : { margin: "8px 8px 0px 0px" }}
                    name={item.op_name}
                    opKey={item.op_key}
                    onDelete={() => {
                        let items = op_items.map((dd) => {
                            if (item.op_key != dd.op_key) {
                                return dd;
                            }
                        }).filter((e) => { return e });

                        onChange(items)
                    }}
                    onClick={(e, data) => {
                        setResData({
                            ...resData,
                            input_op_name: data.name,
                            input_op_key: data.opKey
                        })
                    }}
                />
            })}

        </Box>
        <Divider sx={{ mb: 1, mt: 1 }}></Divider>
        <Grid container spacing={1}>
            <Grid item xs={5}>
                <TextField
                    disabled={loading}
                    label="操作名称"
                    variant="outlined"
                    name="name"
                    focused={focused}
                    size="small"
                    onChange={(e) => {
                        setResData({
                            ...resData,
                            input_op_name: e.target.value
                        })
                    }}
                    value={resData.input_op_name}

                />
            </Grid>
            <Grid item xs={5}>
                <TextField
                    disabled={loading}
                    label="操作标识"
                    variant="outlined"
                    name="name"
                    size="small"
                    onChange={(e) => {
                        setResData({
                            ...resData,
                            input_op_key: e.target.value
                        })
                    }}
                    value={resData.input_op_key}

                />
            </Grid>
            <Grid item xs={2}>
                <Button variant="outlined"
                    onClick={() => {
                        if (resData.input_op_key == '' || resData.input_op_name == '') return
                        let find = false;
                        let items = op_items.map((item) => {
                            if (item.op_key == resData.input_op_key) {
                                find = true;
                                return {
                                    op_key: item.op_key,
                                    op_name: resData.input_op_name
                                }
                            } else {
                                return item;
                            }
                        })
                        if (!find) {
                            items.push({
                                op_key: resData.input_op_key,
                                op_name: resData.input_op_name
                            })
                        }
                        setResData({
                            ...resData,
                            input_op_name: '',
                            input_op_key: ''
                        })
                        onChange(items)
                    }}
                    sx={{
                        borderColor: "#aaa",
                        minWidth: "30px",
                        padding: "7px 14px",

                        '&:hover svg': {
                            color: '#1976d2'
                        }
                    }} >

                    {
                        op_items.find((item) => {
                            return item.op_key == resData.input_op_key
                        }) ? <CheckCircleOutlineIcon color={loading ? "disabled" : "primary"} sx={{
                            color: "#666",
                        }} /> : <AddCircleOutlineIcon color={loading ? "disabled" : "primary"} sx={{
                            color: "#666",
                        }} />
                    }

                </Button>
            </Grid>
        </Grid>
    </FormControl>

}