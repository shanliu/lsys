import Visibility from '@mui/icons-material/Visibility';
import VisibilityOff from '@mui/icons-material/VisibilityOff';
import { Alert, Autocomplete, Box, CircularProgress, FormControl, FormHelperText, FormLabel, Grid, IconButton, InputAdornment, InputLabel, MenuItem, OutlinedInput, Select, Slider, TextField } from '@mui/material';
import PropTypes from 'prop-types';
import randomString from 'random-string';
import React, { useEffect, useRef, useState } from 'react';
import { useUpdateEffect } from 'usehooks-ts';
import ClearIcon from '@mui/icons-material/Clear';

// 带清除按钮输入框
export function ClearTextField(props) {
    const [showIcon, setShowIcon] = useState(false)
    const { onChange, value } = props
    useEffect(() => {
        if ((value + '').length > 0) {
            setShowIcon(true)
        } else {
            setShowIcon(false)
        }
    }, [props.value])
    return <TextField

        value={value ?? ''}
        InputProps={{
            sx: { paddingRight: 0 },
            onChange: (e) => {
                onChange(e, e.target.value)
            },
            value: value ?? '',
            endAdornment: (
                showIcon ? <InputAdornment position="start">
                    <IconButton size='small' onClick={(e) => {
                        onChange(e, '')
                    }}><ClearIcon fontSize="small" /></IconButton>
                </InputAdornment> : null
            ),
        }}
        {...props}
    />;
}
ClearTextField.defaultProps = {
    value: '',
};
ClearTextField.propTypes = {
    value: PropTypes.string,
    onChange: PropTypes.func.isRequired,
};


// 密码输入框
export function PasswordInput(props) {
    const { label, helperText, onChange, size, sx, ...other } = props;
    const [data, setData] = useState({
        show: false
    });
    return <FormControl variant="outlined" sx={sx} size={size}>
        <InputLabel htmlFor={`input-password-${label}`}>{label}</InputLabel>
        <OutlinedInput
            fullWidth
            size={size}
            id={`input-password-${label}`}
            onChange={onChange}
            type={data.show ? 'text' : 'password'}
            endAdornment={
                <InputAdornment position="end">
                    <IconButton
                        onClick={() => {
                            setData({
                                ...data,
                                show: !data.show
                            })
                        }}
                        onMouseDown={(e) => {
                            e.preventDefault();
                        }}
                        edge="end"
                    >
                        {data.show ? <VisibilityOff size="small" /> : <Visibility size="small" />}
                    </IconButton>
                </InputAdornment>
            }
            label={label}
            {...other}
        />
        <FormHelperText error >{helperText}</FormHelperText>
    </FormControl>
}

PasswordInput.defaultProps = {
    sx: {},
};
PasswordInput.propTypes = {
    sx: PropTypes.object,
    label: PropTypes.string.isRequired,
    helperText: PropTypes.string,
    size: PropTypes.string,
    onChange: PropTypes.func.isRequired,
};


// TAG选择或输入
export function InputTagSelect(props) {
    const {
        sx,
        value,
        options,
        onChange,
        disabled,
        ...other } = props;
    let existOption = (values, value) => {
        return values.find((item) => { return item == value })
            || options.find((item) => { return item == value })
    };
    let createOption = (item) => {
        return item;
    };
    let getOptionLabel = (e) => { return e };
    const [tagsData, setTagsData] = useState({
        noValue: "",
        inputValue: '',
    });
    return <Autocomplete
        multiple
        disabled={disabled}
        sx={sx}
        options={options}
        value={value}
        onChange={(_, v) => {
            onChange(v);
        }}
        noOptionsText={tagsData.noValue}
        getOptionLabel={getOptionLabel}
        renderInput={(params) => (
            <TextField
                {...params}
                label="标签"
                placeholder="输入标签"
                value={tagsData.inputValue}
                onKeyUp={(e) => {
                    if (e.key == 'Enter') {
                        return;
                    }
                    let noValue;

                    if (!existOption(value, e.target.value)) {
                        noValue = "回车添加标签:" + e.target.value
                    } else {
                        noValue = "该标签已添加"
                    }
                    setTagsData({
                        ...tagsData,
                        noValue: noValue
                    });
                }}
                onKeyDown={(e) => {
                    if (e.key != 'Enter')
                        return
                    if (e.target.value == '' || !e.target.value) return;
                    if (!existOption(value, e.target.value)) {
                        onChange([...value, createOption(e.target.value)]);
                    }
                    e.stopPropagation();
                    e.preventDefault();
                }}
                {...other}
            />
        )}
        {...other}
    />
}

InputTagSelect.defaultProps = {
    sx: {},
    options: [],
};

InputTagSelect.propTypes = {
    sx: PropTypes.object,
    onChange: PropTypes.func,
    options: PropTypes.array,
};

//从远程加载数据的下拉列表
export function LoadSelect(props) {
    const {
        loading,
        next,
        onLoad,
        error,
        ...other
    } = props;
    let itemRef = useRef();
    useUpdateEffect(() => {
        if (!loading && next && itemRef.current && itemRef.current.getElementsByClassName) {
            let el = itemRef.current.getElementsByClassName("MuiPaper-root")[0];
            if (el && el.clientHeight == el.firstChild.clientHeight) {
                onLoad()
            }
        }
    }, [props.children]);
    return <Select
        {...other}
        MenuProps={
            {
                ref: itemRef
            }
        }
        error={!!error}
        onTransitionEnd={
            (event) => {
                if (event.propertyName != "transform") return
                if (!loading && next && event.target.clientHeight == event.target.firstChild.clientHeight) {
                    onLoad()
                }
            }
        }
        onScrollCapture={
            (event) => {
                if ((event.target.clientHeight + event.target.scrollTop - event.target.scrollHeight) < 10
                    && !loading && next) {
                    onLoad()
                }
            }
        }
    >
        {props.children.map((item) => { return item })}
        {(loading || next) ? <Box style={{ textAlign: "center" }}><CircularProgress size="1em" color="inherit" /></Box> : null}
        {(props.children.length == 0 && error) ? <Alert severity='error' >{error}</Alert> : null}

    </Select>
}

LoadSelect.propTypes = {
    children: PropTypes.array.isRequired,
    next: PropTypes.bool.isRequired,
    loading: PropTypes.bool.isRequired,
};




//TAG下拉列表
export function TagSelect(props) {
    const { loading, error, rows, value, onChange } = props;
    let item;
    if (loading) {
        item = <Select
            labelId="tag-select-small"
            id="tag-select-small"
            label="标签" value="">
            <MenuItem value="">标签加载中...</MenuItem>
        </ Select>
    } else if (error) {
        item = <Select
            value=""
            labelId="tag-select-small"
            id="tag-select-small"
            label="标签">
            <MenuItem value="">{error}</MenuItem>
        </Select>
    } else {
        item = <Select
            labelId="tag-select-small"
            id="tag-select-small"
            label="标签"
            value={value}
            onChange={onChange}
        >
            <MenuItem value="">
                全部
            </MenuItem>
            {
                rows.map((item) => {
                    return <MenuItem key={item[0]} value={item[0]}>{`${item[0]} [${item[1]}个关联]`}</MenuItem>;
                })
            }
        </Select>
    }
    return <FormControl sx={{ minWidth: 120, mr: 1 }} size="small"  >
        <InputLabel id="tag-select-small">标签</InputLabel>
        {item}
    </FormControl>
}

TagSelect.propTypes = {
    rows: PropTypes.array,
    onChange: PropTypes.func,
    error: PropTypes.string,
    loading: PropTypes.bool.isRequired,
};


//验证码输入框
export function CaptchaInput(props) {
    const { src, onChange, value, sx, ...other } = props;
    const [imgSrc, setImgSrc] = useState({
        src: src,
        error: false,
        loading: false
    }, [props.src])
    useEffect(() => {
        setImgSrc({
            src: src,
            loading: true
        })
    }, [props.src])
    useEffect(() => {
        if (!imgSrc.src || imgSrc.src == '') return;
        const imgDom = new Image();
        imgDom.src = imgSrc.src;
        imgDom.onload = function () {
            setImgSrc({
                ...imgSrc,
                error: false,
                loading: false
            })
        }
        imgDom.onerror = function () {
            setImgSrc({
                ...imgSrc,
                error: true,
                loading: false
            })
        }
    }, [imgSrc.src])
    return (<Grid container
        direction="row"
        justifyContent="space-between"
        alignItems="stretch"
        spacing={1} sx={sx} >
        <Grid item xs={7}>
            <TextField
                sx={{
                    width: 1,
                    paddingBottom: 2,
                    height: 1
                }}
                onChange={onChange}
                value={value}
                {...other} />
        </Grid>
        <Grid item xs={5} sx={{ display: "flex", height: 1 }}>
            {(imgSrc.loading || !imgSrc.src || imgSrc.error) ?
                <Box sx={{ textAlign: "center", margin: "auto", width: 1 }} >
                    {imgSrc.error ? <Alert sx={{ m: 0, p: "2px 9px" }} severity='error'>加载错误</Alert> : <CircularProgress sx={{
                        alignSelf: "center",
                        mt: "10px"
                    }} size="1.2em" />}
                </Box> :
                <img style={{
                    alignSelf: "center",
                    width: "100%",
                    borderRadius: 4,
                    cursor: "pointer"
                }}
                    onClick={() => {
                        setImgSrc({
                            src: src.replace(/\?.*$/, "") + "?" + randomString(),
                            loading: true
                        })
                    }} src={imgSrc.src} />
            }
        </Grid>
    </Grid >)
}

CaptchaInput.propTypes = {
    loading: PropTypes.bool,
    src: PropTypes.string,
};



//范围输入框
export function SliderInput(props) {
    const { label,loading, onChange, value, sx,...other} = props;

return <FormControl sx={sx} {...other}>
    <FormLabel style={{
        position: "absolute",
        transform: "translate(-4px, -12px) scale(0.75)"
    }}>{label}</FormLabel>
    <Box className='MuiInputBase-root MuiOutlinedInput-root MuiInputBase-colorPrimary MuiInputBase-formControl MuiInputBase-sizeSmall'
        style={{
            borderRadius: "4px"
        }}>
        <fieldset style={{
            textAlign: "left",
            position: "absolute",
            bottom: 0,
            right: 0,
            top: "-13px",
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
        }} ><span>{label}</span></legend></fieldset>
    </Box>
    <Slider
        disabled={loading}
        sx={{ color: "#aaa", mt: 1, mb: 1 }}
        value={value}
        onChange={onChange}
        step={1}
        marks
        min={1}
        max={100}
        size="small"
        valueLabelDisplay="auto"
    />
</FormControl>
}

