import { Box, Divider, Link, ListItemIcon, ListItemText, MenuItem, MenuList, Paper } from '@mui/material';
import Avatar from '@mui/material/Avatar';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import { red } from '@mui/material/colors';
import React, { useContext, useEffect, useState } from 'react';
import { Link as RouteLink, Navigate, Outlet, useNavigate } from 'react-router-dom';
import { UserSessionContext } from '../../../common/context/session';
import { Progress } from '../../../library/loading';
import { ActiveNavLink } from '../../../library/nav';
import { accessMenu } from '../../../common/rest/access';
import { showTime } from '../../../common/utils/utils';
import { Menus } from './menu';
import { ToastContext } from '../../../common/context/toast';
const drawerWidth = 240;


export default function UserMainPage() {
    const { userData } = useContext(UserSessionContext)
    const navigate = useNavigate();
    const { toast } = useContext(ToastContext);
    let [loadMenu, setLoadMenu] = useState({
        loading: true,
        menu: [],
    });
    useEffect(() => {
        if (!userData) { return }
        accessMenu(Menus).then((data) => {
            if (!data.status) {
                data.message && toast(data.message)
            } else {
                setLoadMenu({
                    ...loadMenu,
                    loading: false,
                    menu: data.data
                })
            }
        })
    }, [])
    if (!userData) {
        return <Navigate to={"/login/main"} />
    }
    return loadMenu.loading ? <Progress /> : <Box sx={{ display: 'flex' }}>
        <Paper variant="permanent"
            anchor="left"
            sx={{
                mt: 2,
                ml: 2,
                width: drawerWidth,
                flexShrink: 0,
                '& .MuiDrawer-paper': {
                    width: drawerWidth,
                    boxSizing: 'border-box',
                },
            }} >
            <Card

                elevation={3}
            >
                <CardHeader
                    avatar={
                        <Link component={RouteLink} to="" underline="none" >
                            <Avatar sx={{ bgcolor: red[500] }}>
                                {userData.user_data.user_nickname.substr(0, 1)}
                            </Avatar>
                        </Link>
                    }
                    title={<span onClick={() => { navigate("/user") }}>{userData.user_data.user_nickname}</span>}
                    subheader={
                        <div style={
                            {
                                marginTop: 3
                            }
                        }>登陆于:{showTime(userData.user_data.login_time).substr(-15)}</div>
                    }
                />
                <Divider variant="middle" ></Divider>
                <CardContent sx={{
                    pt: 1,
                    pb: 1
                }}>
                    <MenuList>
                        {loadMenu.menu.map((item) => {
                            const Icon = item.icon;
                            return <MenuItem key={item.url} prefix="/user/" component={ActiveNavLink} to={item.url} sx={{
                                pt: 2,
                                pb: 2
                            }}  >
                                <ListItemIcon>
                                    <Icon fontSize="small" />
                                </ListItemIcon>
                                <ListItemText>{item.text}</ListItemText>
                            </MenuItem>
                        })}
                    </MenuList>
                </CardContent>
            </Card>
        </Paper>
        <Paper component="main"
            elevation={3}
            sx={{
                margin: 2,
                flexGrow: 1,
                p: 3,
                paddingBottom: 5,
            }} >

            <Outlet />
        </Paper>
    </Box>;
}

