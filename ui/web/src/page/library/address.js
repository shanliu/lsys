import { Box, Divider, FormControl, LinearProgress, ListItemText, MenuItem, MenuList, Paper, Popover, Popper, Stack, TextField, Typography } from '@mui/material';
import React, { useState } from 'react';
import NavigateNextIcon from '@mui/icons-material/NavigateNext';
import ClickAwayListener from '@mui/base/ClickAwayListener';
//地址选择
export function AddressSelect(props) {
    const { size, ...other } = props;
    const [value, setValue] = useState("");
    const [loadData, setLoadData] = useState({
        loading: false
    })
    const [anchorEl, setAnchorEl] = useState(null);
    const [onfr, setFr] = useState(true);
    const [open, setOpen] = useState(false);

    return <FormControl  {...other}>
        <TextField
            autoComplete="off"
            variant="outlined"
            label="用户昵称"
            name="nikename"
            size={size}
            focused={onfr}
            onFocus={(event) => {
                setAnchorEl(event.currentTarget);
                setFr(true)
                setOpen(true)
            }}
            onBlur={(event) => {
                setFr(false)
            }}
            onChange={(e) => {
                let val = e.target.value ?? '';
                val = val.replace(/^\s+/, '');
                val = val.replace(/\s$/, '');
                setValue(val)
            }}
            value={value}
            onMouseLeave={() => {
                setFr(0)
            }}
        />
        {
            anchorEl ? <Popper
                onMouseEnter={() => {
                    setOpen(true)
                }}
                onMouseLeave={() => {
                    setOpen(false)
                }}
                anchorOrigin={{
                    vertical: 'bottom',
                    horizontal: 'left',
                }}
                open={open}
                onClose={() => {
                    setOpen(false)
                }}
                anchorEl={anchorEl}
                marginThreshold={1}
                sx={{ zIndex: (theme) => theme.zIndex.drawer + 4 }}
            >
                <Paper sx={{ width: anchorEl.offsetWidth }} >
                    {loadData.loading ? <LinearProgress /> : null}
                    {value.length > 0 ?
                        <MenuList dense>
                            <MenuItem onClick={() => {
                                setFr(1)
                            }} selected={true} sx={{ paddingLeft: 1, paddingRight: 1, lineHeight: 2.5 }}>
                                <ListItemText>广东，深圳，龙岗</ListItemText>
                            </MenuItem>
                            <MenuItem onClick={() => {
                                setFr(1)
                            }} sx={{ paddingLeft: 1, paddingRight: 1, lineHeight: 2.5 }}>
                                <ListItemText>广东，深圳，龙岗</ListItemText>
                            </MenuItem>
                        </MenuList> :
                        <Stack direction="row"
                            justifyContent="space-between"
                            alignItems="flex-start"
                            spacing={0}

                            sx={{ flexWrap: "nowrap", width: anchorEl.offsetWidth }}
                            divider={<Divider orientation="vertical" flexItem />}
                        >
                            <MenuList dense sx={{
                                flex: 1,
                                textOverflow: "ellipsis",
                                overflow: "hidden",
                                wordBreak: " break-all",
                                whiteSpace: "nowrap"
                            }}>
                                <MenuItem selected={true} sx={{ paddingLeft: 1, paddingRight: 1, lineHeight: 2.5 }}>

                                    <Stack direction="row"
                                        sx={{ width: "100%" }}
                                        alignItems="center"
                                        justifyContent="space-between"
                                    >
                                        <Typography variant="inherit" noWrap>
                                            A very long text that overflows
                                        </Typography>
                                        <NavigateNextIcon />
                                    </Stack>

                                </MenuItem>
                                <MenuItem sx={{ paddingLeft: 1, paddingRight: 1, lineHeight: 2.5 }}>
                                    <Stack direction="row"
                                        sx={{ width: "100%" }}
                                        alignItems="center"
                                        justifyContent="space-between"
                                    >
                                        <Typography variant="inherit" noWrap>
                                            A very long text that overflows
                                        </Typography>
                                        <NavigateNextIcon color="disabled" />
                                    </Stack>
                                </MenuItem>
                            </MenuList>
                            <MenuList dense sx={{
                                flex: 1,
                                textOverflow: "ellipsis",
                                overflow: "hidden",
                                wordBreak: " break-all",
                                whiteSpace: "nowrap"
                            }}>
                                <MenuItem sx={{ paddingLeft: 1, paddingRight: 1, lineHeight: 2.5 }}>
                                    <Stack direction="row"
                                        sx={{ width: "100%" }}
                                        alignItems="center"
                                    >
                                        <Typography variant="inherit" noWrap>
                                            A very long text that overflows
                                        </Typography>
                                        <NavigateNextIcon color="disabled" />
                                    </Stack>

                                </MenuItem>
                                <MenuItem selected={true} sx={{ paddingLeft: 1, paddingRight: 1, lineHeight: 2.5 }}>
                                    <Stack direction="row"
                                        sx={{ width: "100%" }}
                                        alignItems="center"
                                    >
                                        <Typography variant="inherit" noWrap>
                                            A very long text that overflows
                                        </Typography>
                                        <NavigateNextIcon />
                                    </Stack>

                                </MenuItem>
                            </MenuList>
                            <MenuList dense sx={{
                                flex: 1,

                                overflow: "hidden",
                                wordBreak: " break-all",
                                whiteSpace: "nowrap"
                            }}>
                                <MenuItem sx={{ paddingLeft: 1, paddingRight: 1, lineHeight: 2.5 }}>
                                    <Stack direction="row"
                                        sx={{ width: "100%" }}
                                        alignItems="center"
                                    >
                                        <Typography variant="inherit" noWrap>
                                            A very long text that overflows
                                        </Typography>
                                        <NavigateNextIcon color="disabled" />
                                    </Stack>

                                </MenuItem>
                                <MenuItem selected={true} sx={{ paddingLeft: 1, paddingRight: 1, lineHeight: 2.5 }}>
                                    <Stack direction="row"
                                        sx={{ width: "100%" }}
                                        alignItems="center"
                                    >
                                        <Typography variant="inherit" noWrap>
                                            A very long text that overflows
                                        </Typography>
                                        <NavigateNextIcon />
                                    </Stack>

                                </MenuItem>
                            </MenuList>
                            <MenuList dense sx={{
                                flex: 1,
                                textOverflow: "ellipsis",
                                overflow: "hidden",
                                wordBreak: " break-all",
                                whiteSpace: "nowrap"
                            }}>
                                <MenuItem selected={true} sx={{ paddingLeft: 1, paddingRight: 1, lineHeight: 2.5 }}>
                                    <Stack direction="row"
                                        sx={{ width: "100%" }}
                                        alignItems="center"
                                    >
                                        <Typography variant="inherit" noWrap>
                                            A very long text that overflows
                                        </Typography>
                                        <NavigateNextIcon color="disabled" />
                                    </Stack>

                                </MenuItem>
                                <MenuItem sx={{ paddingLeft: 1, paddingRight: 1, lineHeight: 2.5 }}>
                                    <Stack direction="row"
                                        sx={{ width: "100%" }}
                                        alignItems="center"
                                    >
                                        <Typography variant="inherit" noWrap>
                                            A very long text that overflows
                                        </Typography>
                                        <NavigateNextIcon color="disabled" />
                                    </Stack>

                                </MenuItem>
                            </MenuList>
                            <MenuList dense sx={{
                                flex: 3,
                                textOverflow: "ellipsis",
                                overflow: "hidden",
                                wordBreak: " break-all",
                                whiteSpace: "nowrap"
                            }}>
                                <MenuItem>
                                    <Stack direction="row"
                                        sx={{ width: "100%" }}
                                        alignItems="center"
                                    >
                                        <Typography variant="inherit" noWrap>
                                            A very long text that overflows
                                        </Typography>

                                    </Stack>

                                </MenuItem>

                                <MenuItem>
                                    <Stack direction="row"
                                        sx={{ width: "100%" }}
                                        alignItems="center"
                                    >
                                        <Typography variant="inherit" noWrap>
                                            A very long text that overflows
                                        </Typography>

                                    </Stack>
                                </MenuItem>
                            </MenuList>
                        </Stack>
                    }
                </Paper>
            </Popper> : null
        }

    </FormControl >
};
