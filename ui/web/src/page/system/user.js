
import SearchIcon from '@mui/icons-material/Search';
import { Accordion, AccordionDetails, AccordionSummary, Alert, Avatar, Card, CardContent, CardHeader, Divider, Drawer, FormControl, IconButton, InputLabel, List, ListItem, ListItemIcon, ListItemText, ListSubheader, MenuItem, Paper, Select, Stack, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useEffect, useState } from 'react';
import { ClearTextField } from '../../library/input';
import { LoadingButton } from '../../library/loading';
import { SimplePaginationTablePage } from '../../library/table_page';
import { ItemTooltip } from '../../library/tips';
import { OauthType, genderStatus, searchType, userSearch, userStatus } from '../../rest/user';
import { useSearchChange } from '../../utils/hook';
import { showTime } from '../../utils/utils';
import MoreHorizIcon from '@mui/icons-material/MoreHoriz';
import SmsIcon from '@mui/icons-material/Sms';
import HomeOutlinedIcon from '@mui/icons-material/HomeOutlined';
import MailIcon from '@mui/icons-material/Mail';
import PeopleOutlineOutlinedIcon from '@mui/icons-material/PeopleOutlineOutlined';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import StyleOutlinedIcon from '@mui/icons-material/StyleOutlined';
import NotListedLocationOutlinedIcon from '@mui/icons-material/NotListedLocationOutlined';
import PermIdentityOutlinedIcon from '@mui/icons-material/PermIdentityOutlined';
const filterStatus = [
    { key: 1, val: '全部' },
    { key: 2, val: '已确认' },
];

function SystemUserDetail(props) {
    const { item } = props;
    let gender = null
    if (item.info) {
        gender = genderStatus.find((e) => e.key == item.info.gender);
    }
    let email_ok = item.email ? item.email.filter((e) => e.status == 2) : null;
    let email_init = item.email ? item.email.filter((e) => e.status != 2) : null;
    let mobile_ok = item.mobile ? item.mobile.filter((e) => e.status == 2) : null;
    let mobile_init = item.mobile ? item.mobile.filter((e) => e.status != 2) : null;

    return <Fragment>
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
            用户详细
        </Typography>
        <Divider variant="middle" />
        <Stack direction="column"
            justifyContent="flex-start"
            alignItems="stretch"
            spacing={2}>
            <Card sx={{ margin: 2 }}>
                <CardHeader
                    avatar={
                        <Avatar aria-label="recipe">
                            {item.user.nickname.substr(0, 1)}
                        </Avatar>
                    }
                    title={item.user.nickname}
                    subheader={`注册时间:${showTime(item.add_time, "未知")}`}
                />
                <CardContent sx={{ pt: 1 }}>
                    <Box >
                        <Stack
                            direction="row"
                            justifyContent="flex-start"
                            alignItems="center"
                            spacing={1}
                        >
                            <PermIdentityOutlinedIcon color="disabled" fontSize='small' />
                            <Typography color="text.secondary" gutterBottom>
                                用户信息
                            </Typography>
                        </Stack>
                        <Typography variant="body2" sx={{ mt: 1, mb: 1.5, lineHeight: 1.6 }} color="text.secondary">
                            {item.user.status == 2 ?
                                <Fragment>
                                    <label>状态:</label><span>已激活</span><br />
                                    <label>激活时间:</label><span>{showTime(item.user.confirm_time, "未知")}</span><br />
                                </Fragment> :
                                <Fragment><label>状态:</label><span>未激活</span><br /></Fragment>
                            }
                            {item.user.password_id > 0 ? <Fragment><label>密码状态:</label><span>已设置</span><br /></Fragment> : null}
                            {item.info ?
                                <Fragment>
                                    {gender ? <Fragment><label>性别:</label><span>{gender.val}</span><br /></Fragment> : null}
                                    {item.info.birthday ? <Fragment><label>性别:</label><span>{item.info.birthday}</span><br /></Fragment> : null}
                                    {item.info.reg_ip ? <Fragment><label>注册IP:</label><span>{item.info.reg_ip}</span><br /></Fragment> : null}
                                    {item.info.reg_from ? <Fragment><label>注册来源:</label><span>{item.info.reg_from}</span><br /></Fragment> : null}
                                </Fragment> : null
                            }
                            {/* 最后登陆时间:111.11.1.1 */}
                        </Typography>
                    </Box>
                    {item.name && item.name.status ?
                        <Box >
                            <Stack
                                direction="row"
                                justifyContent="flex-start"
                                alignItems="center"
                                spacing={1}
                            >
                                <StyleOutlinedIcon color="disabled" fontSize='small' />
                                <Typography color="text.secondary" gutterBottom>
                                    登陆用户名
                                </Typography>
                            </Stack>
                            <Typography variant="h6" component="div">
                                {item.name.username}
                            </Typography>
                            <Typography sx={{ fontSize: 14, mb: 1.5 }} color="text.secondary">
                                设置时间:{showTime(item.name.change_time, "未知")}
                            </Typography>
                        </Box> : null
                    }


                    {mobile_ok || mobile_init ? <Accordion>
                        <AccordionSummary
                            expandIcon={<ExpandMoreIcon />}
                        >
                            <Typography sx={{ width: '33%', flexShrink: 0 }}>
                                绑定手机
                            </Typography>
                            <Typography sx={{ color: 'text.secondary', fontSize: "0.8rem", marginTop: '2px' }}>
                                {(mobile_ok && mobile_ok.length) ? `已确认 ${mobile_ok.length} ` : null}
                                {(mobile_init && mobile_init.length) ? `待确认 ${mobile_init.length} 个` : null}
                            </Typography>
                            <Typography></Typography>
                        </AccordionSummary>
                        <AccordionDetails>
                            <List >
                                {mobile_ok ? mobile_ok.map((e) => {
                                    return <ListItem key={`mo-${e.id}`}>
                                        <ListItemIcon>
                                            <SmsIcon fontSize='small' />
                                        </ListItemIcon>
                                        <ListItemText
                                            primary={e.area_code + "-" + e.mobile}
                                            secondary={`确认于:${showTime(e.confirm_time, "未知")}`}
                                        />
                                    </ListItem>
                                }) : null}
                                {mobile_init ? mobile_init.map((e) => {
                                    return <ListItem key={`mi-${e.id}`}>
                                        <ListItemIcon>
                                            <SmsIcon fontSize='small' />
                                        </ListItemIcon>
                                        <ListItemText title="未确认"
                                            primary={<Stack
                                                direction="row"
                                                justifyContent="flex-start"
                                                alignItems="center"
                                                spacing={1}
                                            >
                                                <Box>
                                                    {e.area_code + "-" + e.mobile}
                                                </Box>
                                                <NotListedLocationOutlinedIcon color="disabled" fontSize='small' />
                                            </Stack>}
                                            secondary={`添加于:${showTime(e.change_time, "未知")}`}
                                        />
                                    </ListItem>
                                }) : null}
                            </List>
                        </AccordionDetails>
                    </Accordion> : null}
                    {email_ok || email_init ? <Accordion>
                        <AccordionSummary
                            expandIcon={<ExpandMoreIcon />}
                        >
                            <Typography sx={{ width: '33%', flexShrink: 0 }}>
                                绑定邮箱
                            </Typography>
                            <Typography sx={{ color: 'text.secondary', fontSize: "0.8rem", marginTop: '2px' }}>
                                {(email_ok && email_ok.length) ? `已确认 ${email_ok.length} 个` : null}
                                {(email_init && email_init.length) ? `待确认 ${email_init.length} 个` : null}
                            </Typography>
                            <Typography></Typography>
                        </AccordionSummary>
                        <AccordionDetails>
                            <List >
                                {email_ok ? email_ok.map((e) => {
                                    return <ListItem key={`eo-${e.id}`}>
                                        <ListItemIcon>
                                            <MailIcon fontSize='small' />
                                        </ListItemIcon>
                                        <ListItemText
                                            primary={e.email}
                                            secondary={`确认于:${showTime(e.confirm_time, "未知")}`}
                                        />
                                    </ListItem>
                                }) : null}
                                {email_init ? email_init.map((e) => {
                                    return <ListItem key={`ei-${e.id}`}>
                                        <ListItemIcon>
                                            <MailIcon fontSize='small' />
                                        </ListItemIcon>
                                        <ListItemText title="未确认"
                                            primary={<Stack
                                                direction="row"
                                                justifyContent="flex-start"
                                                alignItems="center"
                                                spacing={1}
                                            ><Box>{e.email}</Box><NotListedLocationOutlinedIcon color='disabled' fontSize='small' /></Stack>}

                                            secondary={`添加于:${showTime(e.change_time, "未知")}`}
                                        />
                                    </ListItem>
                                }) : null}
                            </List>
                        </AccordionDetails>
                    </Accordion> : null}
                    {item.external && item.external.length ? <Accordion>
                        <AccordionSummary
                            expandIcon={<ExpandMoreIcon />}
                        >
                            <Typography sx={{ width: '33%', flexShrink: 0 }}>
                                绑定外部账号
                            </Typography>
                            <Typography sx={{ color: 'text.secondary', fontSize: "0.8rem", marginTop: '2px' }}>
                                {`已绑定 ${item.external.length} 个`}
                            </Typography>
                            <Typography></Typography>
                        </AccordionSummary>
                        <AccordionDetails>
                            <List >
                                {item.external ? item.external.map((row) => {
                                    let item = OauthType.find((e) => { return e.key == row.external_type });
                                    let name = (item ? item.val : row.external_type) + " : " + row.external_nikename;
                                    return <ListItem key={`et-${row.id}`}>
                                        <ListItemIcon>
                                            <PeopleOutlineOutlinedIcon fontSize='small' />
                                        </ListItemIcon>
                                        <ListItemText
                                            primary={name}
                                            secondary={<Fragment>

                                                {`外部账号:${row.external_name}`} <br />

                                                {`TOKEN超时:${showTime(row.token_timeout, "未知")}`}<br />

                                                {`绑定时间:${showTime(row.change_time, "未知")}`}

                                            </Fragment>}
                                        />
                                    </ListItem>
                                }) : null}
                            </List>
                        </AccordionDetails>
                    </Accordion> : null}
                    {item.address && item.address.length ? <Accordion>
                        <AccordionSummary
                            expandIcon={<ExpandMoreIcon />}
                        >
                            <Typography sx={{ width: '33%', flexShrink: 0 }}>
                                用户收货地址
                            </Typography>
                            <Typography sx={{ color: 'text.secondary', fontSize: "0.8rem", marginTop: '2px' }}>
                                {`已添加 ${item.address.length} 个`}
                            </Typography>
                            <Typography></Typography>
                        </AccordionSummary>
                        <AccordionDetails>
                            <List >
                                {item.address ? item.address.map((row) => {
                                    return <ListItem key={`ar-${row.id}`}>
                                        <ListItemIcon>
                                            <HomeOutlinedIcon fontSize='small' />
                                        </ListItemIcon>
                                        <ListItemText
                                            primary={row.address_info + row.address_detail}
                                            secondary={`添加时间:${showTime(row.change_time, "未知")}`}
                                        />
                                    </ListItem>
                                }) : null}
                            </List>
                        </AccordionDetails>
                    </Accordion> : null}
                </CardContent>
            </Card>
        </Stack>

    </Fragment>
}

export default function SystemUserPage(props) {
    const [searchParam, setSearchParam] = useSearchChange({
        key_word: "",
        start_pos: '',
        end_pos: '',
        page_size: 25,
    });
    let [loadData, setLoadData] = useState({
        status: false,
        message: null,
        loading: true,
        data: [],
        startPos: '',
        nextPos: '',
        isFirst: false,
        isEnd: true,
    });
    const columns = [
        {

            label: 'ID',
            align: "right",
            style: { width: 90 },
            render: (row) => {
                return row.user.id
            }
        },
        {
            style: { width: 180 },
            label: '昵称',
            render: (row) => {
                return <span>{row.user.nickname}</span>
            }
        },
        {
            style: { width: 80 },
            label: '状态',
            render: (row) => {
                let status = userStatus.map((status) => {
                    if (status.key == row.user.status) return status.val
                })
                return status ?? "未知"
            }
        },
        {
            label: '相关资料',
            render: (row) => {
                let more = <IconButton onClick={() => {
                    setBoxPage({
                        show: true,
                        box: "detail",
                        item: row
                    })
                }}><MoreHorizIcon fontSize='small' /></IconButton>;
                let detail = [];
                if (row.user.password_id > 0) {
                    detail.push("已设置密码")
                }
                if (row.user.use_name > 0) {
                    detail.push("启用用户名")
                }
                if (row.user.email_count > 0) {
                    detail.push("绑定邮箱:" + row.user.email_count + "个")
                }
                if (row.user.mobile_count > 0) {
                    detail.push("绑定手机号:" + row.user.mobile_count + "个")
                }
                if (row.user.external_count > 0) {
                    detail.push("关联外部账号:" + row.user.external_count + "个")
                }
                if (row.user.address_count > 0) {
                    detail.push("添加地址:" + row.user.address_count + "个")
                }

                if (row.cat && row.cat.length > 0) {
                    let msg = [];
                    row.cat.map((e) => {
                        searchType.map((t) => {
                            if (t.key == e.type) {
                                msg.push(`匹配[${t.val}]:${e.val}`)
                            }
                        })
                    })
                    return <ItemTooltip placement="top" arrow title={detail.join(",")}>< Box > {msg.join(",")}{more}</Box ></ItemTooltip >
                }
                return <Box>{detail.join(",")}{more}</Box>
            }
        },
        {
            style: { width: 180 },
            label: '注册时间',
            render: (row) => {
                return showTime(row.user.add_time, "未知")
            }
        },
        {
            style: { width: 180 },
            label: '激活时间',
            render: (row) => {
                return showTime(row.user.confirm_time, "未知")
            }
        },
    ];

    useEffect(() => {
        let startPos = searchParam.get("start_pos") ?? '';
        setLoadData({
            ...loadData,
            isFirst: (!startPos || startPos == '') ? true : false,
        })
    }, [])
    const [filterData, setfilterData] = useState({
        status: filterStatus[0].key,
        key_word: searchParam.get("key_word"),
    })
    const loadUserData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        let param = {
            more: true,
            opt: true,
            key_word: searchParam.get("key_word"),
            start_pos: searchParam.get("start_pos") ?? '',
            end_pos: searchParam.get("end_pos") ?? '',
            page_size: searchParam.get("page_size") || 25,
            enable_user: false,
        }
        return userSearch(param).then((data) => {
            let setData = data.status && data.data && data.data.length > 0 ? data.data : [];
            if (param.end_pos && param.end_pos != '') {
                setLoadData({
                    ...loadData,
                    status: data.status ?? false,
                    message: data.message ?? '',
                    data: setData,
                    loading: false,
                    startPos: setData.length > 0 ? setData[0].id : '',
                    nextPos: param.end_pos,
                    isFirst: !data.status || !data.next || data.next == '',
                    isEnd: false,
                })
            } else {
                setLoadData({
                    ...loadData,
                    status: data.status ?? false,
                    message: data.message ?? '',
                    data: setData,
                    loading: false,
                    startPos: setData.length > 0 ? setData[0].id : '',
                    nextPos: data.status && data.next ? data.next : '',
                    isFirst: !param.start_pos || param.start_pos == '',
                    isEnd: !data.status || !data.next || data.next == '',
                })
            }
        })
    }
    useEffect(() => {
        setfilterData({
            ...filterData,
            key_word: searchParam.get("key_word"),
        })
        loadUserData()
    }, [searchParam])


    const [boxPage, setBoxPage] = useState({
        show: false,
        box: null,
        item: null
    });
    let boxData;
    switch (boxPage.box) {
        case "detail":
            boxData = <SystemUserDetail item={boxPage.item} />
            break;
    }


    return <Fragment>
        <Drawer
            sx={{ zIndex: (theme) => theme.zIndex.drawer + 3 }}
            anchor={"right"}
            open={boxPage.show}
            onClose={() => {
                setBoxPage({
                    box: null,
                    show: false,
                    item: null
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
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
        >
            <FormControl sx={{ minWidth: 110, mr: 1 }} size="small"  >
                <InputLabel id="select-type">审核状态</InputLabel>
                <Select
                    labelId="select-type"
                    id="select-type"
                    label="审核状态"
                    disabled={loadData.loading}
                    onChange={(event) => {
                        setfilterData({
                            ...filterData,
                            status: event.target.value
                        })
                    }}
                    value={filterData.status ?? ''}
                >
                    {
                        filterStatus.map((status) => {
                            return <MenuItem key={`status_${status.key}`} value={status.key}>{status.val}</MenuItem>
                        })
                    }
                </Select>
            </FormControl>
            <FormControl sx={{ minWidth: 250, mr: 1 }} size="small"  >
                <ClearTextField
                    sx={{ mr: 1 }}
                    variant="outlined"
                    label={`搜索用户`}
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
                        start_pos: '',
                        end_pos: ''
                    }, loadUserData)
                }}
                variant="outlined"
                size="medium"
                startIcon={<SearchIcon />}
                sx={{ mr: 1, p: "7px 15px", minWidth: 110 }}
                loading={loadData.loading}
            >
                过滤
            </LoadingButton>
        </Paper>

        {(loadData.status || loadData.loading)
            ? <Box sx={{ height: 1, width: '100%' }}>
                <SimplePaginationTablePage
                    rows={loadData.data ?? []}
                    columns={columns}
                    isFirst={loadData.isFirst}
                    isEnd={loadData.isEnd}
                    onPageChange={(e, next) => {
                        if (next) {
                            setSearchParam({
                                start_pos: loadData.nextPos,
                                end_pos: '',
                            }, loadUserData)
                        } else {
                            setSearchParam({
                                start_pos: '',
                                end_pos: loadData.startPos,
                            }, loadUserData)
                        }
                    }}
                    rowsPerPage={searchParam.get("page_size") || 25}
                    onRowsPerPageChange={(e) => {
                        setSearchParam({
                            page_size: e.target.value,
                        }, loadUserData)
                    }}
                    loading={loadData.loading}
                />

            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}


