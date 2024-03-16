import { Box, Chip, ClickAwayListener, Divider, FormControl, Input, InputBase, LinearProgress, ListItemText, MenuItem, MenuList, Paper, Popper, Stack, TextField, Typography } from '@mui/material';
import React, { Fragment, forwardRef, useCallback, useContext, useDeferredValue, useEffect, useRef, useState } from 'react';
import NavigateNextIcon from '@mui/icons-material/NavigateNext';
import { areaRelated, areaList, areaSearch } from '../../../common/rest/area';
import { ToastContext } from '../../../common/context/toast';

/**
 * @param {string} code 
 */
function parseCode(code) {
    let len = code.length;
    let start = 0;
    let out = [];
    if (len == 0) return out;
    while (true) {
        let offset = 2;
        if (start >= 6) {
            offset = 3;
        }
        out.push(code.substring(0, offset + start))
        start += offset;
        if (start >= len) break;
    }
    return out;
}


//地址选择
export function AddressSelect(props) {
    const { toast } = useContext(ToastContext);
    const { addrCode, addrInfo, onChange, sx, ...other } = props;
    const [listData, setListData] = useState([])
    const [searchData, setSearchData] = useState([])
    const [loading, setLoading] = useState(false)


    let ajaxReq = useRef(null)


    let cityCache = useRef({})
    let listDataSet = useCallback((code_data) => {
        let data = [];
        code_data.unshift("");
        for (let code_i in code_data) {
            let code_key = code_data[code_i]
            let code_val = code_data[1 + parseInt(code_i)];
            let tmp = cityCache.current[code_key];
            if (tmp) data.push(tmp.map((e) => {
                let selected = code_val && e.code == code_val;
                return {
                    ...e,
                    selected: selected
                }
            }))
        }
        setListData(data)
    }, [cityCache])
    let listCode = useCallback((code) => {
        if (ajaxReq.current) {
            ajaxReq.current.abort()
        }
        ajaxReq.current = new AbortController();
        setLoading(true)
        areaList({
            code: code
        }, {
            signal: ajaxReq.current.signal
        }).then((data) => {
            setLoading(false)
            ajaxReq.current = null;
            if (!data.status) {
                if (data.message.indexOf("canceled") === false) {
                    toast("加载编码失败[" + code + "]:" + data.message)
                }

                return;
            }
            cityCache.current[code] = data.area
            listDataSet(parseCode(code))
        })
    })
    let detailCode = useCallback((code) => {
        setLoading(true)
        if (ajaxReq.current) {
            ajaxReq.current.abort()
        }
        ajaxReq.current = new AbortController();

        areaRelated({
            code: code
        }, {
            signal: ajaxReq.current.signal
        }).then((data) => {
            setLoading(false)
            ajaxReq.current = null;
            if (!data.status) {
                if (data.message.indexOf("canceled") === false) {
                    toast("加载编码失败[" + code + "]:" + data.message)
                }
                return;
            }
            let code_data = parseCode(code)
            code_data.unshift("");
            for (let tmp in code_data) {
                if (!data.area || !data.area[tmp]) break;
                cityCache.current[code_data[tmp]] = data.area[tmp]
            }
            listDataSet(parseCode(code))
        })
    })

    const [nowValue, setNowValue] = useState("");
    const [anchorEl, setAnchorEl] = useState(null);
    const [open, setOpen] = useState(false);
    const [inSearch, setInSearch] = useState(false);

    useEffect(() => {
        if (inSearch) return;
        let tdat = null;
        if (cityCache.current[addrCode]) {
            listDataSet(parseCode(addrCode))
            return
        }
        if (addrCode.length > 0) {
            let code_data = parseCode(addrCode)
            code_data.unshift("");
            tdat = cityCache.current[code_data[code_data.length - 2]];
        }
        if (addrCode == '' || tdat) {
            if (tdat && tdat.find((e) => { return e.code == addrCode && e.leaf })) {
                listDataSet(parseCode(addrCode))
                return;
            }
            listCode(addrCode)
        } else {
            detailCode(addrCode)
        }
    }, [addrCode])


    const inputDefVal = useDeferredValue(nowValue);
    let searchCode = useCallback((key_word) => {
        setLoading(true)
        if (ajaxReq.current) {
            ajaxReq.current.abort()
        }
        ajaxReq.current = new AbortController();
        areaSearch({
            key_word: key_word
        }, {
            signal: ajaxReq.current.signal
        }).then((data) => {
            setLoading(false)
            ajaxReq.current = null;
            if (!data.status) {
                if (data.message.indexOf("canceled") === false) {
                    toast("搜索编码失败[" + key_word + "]:" + data.message)
                }
                return;
            }
            setSearchData(data.area)
        })
    })

    useEffect(() => {
        if (inputDefVal == '' || nowValue == '') {
            return
        }
        if (inputDefVal != nowValue) {
            setLoading(true)
        }
        searchCode(inputDefVal)
    }, [inputDefVal])
    const inProp = useRef(false);
    const hideRef = useRef(null);

    const handleClose = (event) => {
        if (
            anchorEl.current &&
            anchorEl.current.contains(event.target)
        ) {
            return;
        }
        setOpen(inProp.current);
    };

    return <FormControl sx={sx}>

        <Fragment>
            <TextField
                {...other}
                sx={{ width: 1 }}
                autoComplete="off"
                variant="outlined"
                size={"small"}
                onMouseEnter={(event) => {
                    hideRef.current && clearTimeout(hideRef.current)
                    setAnchorEl(event.currentTarget);
                    setOpen(true)
                    inProp.current = true;
                }}
                onMouseLeave={() => {
                    inProp.current = false;
                    setOpen(false)
                    hideRef.current && clearTimeout(hideRef.current)
                    hideRef.current = setTimeout(() => {
                        setOpen(inProp.current)
                    }, 100)
                }}
                value={addrInfo + ''}
                disabled={true}
            />

            {
                anchorEl ? <Popper
                    onMouseEnter={(event) => {
                        hideRef.current && clearTimeout(hideRef.current)
                        setOpen(true)
                        inProp.current = true;
                    }}
                    onMouseLeave={() => {
                        inProp.current = false;
                        hideRef.current && clearTimeout(hideRef.current)
                        hideRef.current = setTimeout(() => {
                            setOpen(inProp.current)
                        }, 800)
                    }}
                    open={open}
                    anchorEl={anchorEl}
                    marginthreshold={1}
                    sx={{ zIndex: (theme) => theme.zIndex.drawer + 4 }}
                >
                    <Paper sx={{ width: anchorEl.offsetWidth }} >
                        <ClickAwayListener onClickAway={handleClose}>
                            <Box sx={{ p: 1 }}>
                                <TextField
                                    label="搜索地址"
                                    placeholder='输入城市名称，拼音等进行搜索'
                                    autoComplete="off"
                                    sx={{ width: 1 }}
                                    variant="outlined"
                                    size={"small"}
                                    onChange={(e) => {
                                        let val = e.target.value ?? '';
                                        val = val.replace(/^\s+/, '');
                                        setNowValue(val)
                                    }}
                                    value={nowValue + ''}
                                />

                                {loading ? <LinearProgress /> : null}
                                {(nowValue.length > 0 || inSearch) && searchData.length > 0 ?
                                    <MenuList
                                        onMouseEnter={() => {
                                            setInSearch(true);
                                        }}
                                        onMouseLeave={() => {
                                            setInSearch(false);
                                        }}
                                        dense>
                                        {searchData.map((ldat, si) => {
                                            let showtxt = ldat.map((tt) => {
                                                return tt.name
                                            }).join(",");
                                            let lastcode = ldat[ldat.length - 1].code;
                                            return <MenuItem key={`search-${si}`} onClick={() => {
                                                onChange(lastcode, showtxt)
                                                setInSearch(false);
                                                setNowValue('')
                                                if (lastcode.leaf) setOpen(false)
                                            }} selected={lastcode == addrCode} sx={{ paddingLeft: 1, paddingRight: 1, lineHeight: 2.5 }}>
                                                <ListItemText>{showtxt}</ListItemText>
                                            </MenuItem>
                                        })}
                                    </MenuList> :
                                    <Stack direction="row"
                                        justifyContent="space-between"
                                        alignItems="flex-start"
                                        spacing={0}
                                        sx={{ flexWrap: "nowrap", width: 1, height: 200 }}
                                        divider={<Divider orientation="vertical" flexItem />}
                                    >
                                        {listData.map((ldat, di) => {
                                            return <MenuList
                                                key={`group-${di}`} dense sx={{
                                                    height: 1,
                                                    overflowY: 'auto',
                                                    overflowX: 'hidden',
                                                    flex: di + 1 == listData.length && listData.length > 2 ? listData.length / 2 : 1,
                                                    textOverflow: "ellipsis",
                                                    wordBreak: " break-all",
                                                    whiteSpace: "nowrap"
                                                }}>
                                                {
                                                    ldat.map((litm, li) => {
                                                        return <MenuItem
                                                            title={litm.name}
                                                            key={`list-${li}`}
                                                            onClick={(e) => {
                                                                let name_val = [];
                                                                let tdata = cityCache.current[""];
                                                                for (let tcode of parseCode(litm.code)) {
                                                                    let c = tdata.find((t) => { return t.code == tcode });
                                                                    if (c) name_val.push(c.name)
                                                                    tdata = cityCache.current[tcode];
                                                                }
                                                                onChange(litm.code, name_val.join(","))
                                                                if (litm.leaf) setOpen(false)
                                                            }}
                                                            selected={litm.selected}
                                                            sx={{ paddingLeft: 1, paddingRight: 1, lineHeight: 2.5 }}>
                                                            <Stack direction="row"
                                                                sx={{ width: "100%" }}
                                                                alignItems="center"
                                                                justifyContent="space-between"
                                                            >
                                                                <Typography variant="inherit" noWrap>
                                                                    {litm.name}
                                                                </Typography>
                                                                {litm.leaf ? null : <NavigateNextIcon />}
                                                            </Stack>
                                                        </MenuItem>
                                                    })
                                                }
                                            </MenuList>
                                        })
                                        }
                                    </Stack>
                                }
                            </Box>
                        </ClickAwayListener>
                    </Paper>
                </Popper> : null
            }</Fragment>
    </FormControl >
};
