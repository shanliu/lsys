import { Box, Divider, Link, ListItemIcon, ListItemText, MenuItem, MenuList, Paper } from '@mui/material';
import Avatar from '@mui/material/Avatar';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import { red } from '@mui/material/colors';
import React, { useEffect, useState } from 'react';
import { Link as RouteLink, Navigate, NavLink, Outlet, useParams, useLocation, useNavigate } from 'react-router-dom';
import { UserSessionContext } from '../../context/session';
import { Progress } from '../../library/loading';
import { ActiveNavLink } from '../../library/nav';
import { accessMenu } from '../../rest/access';
import { showTime } from '../../utils/utils';
import { Menus } from './menu';
const drawerWidth = 240;


export default function UserMainPage() {

    const { userData } = React.useContext(UserSessionContext)
    if (!userData) {
        return <Navigate to={"/login/main"} />
    }

    const navigate = useNavigate();

    let [loadMenu, setLoadMenu] = useState({
        loading: true,
        menu: [],
    });
    useEffect(() => {
        accessMenu(Menus).then((data) => {
            setLoadMenu({
                ...loadMenu,
                loading: false,
                menu: data
            })
        })
    }, [])


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
                                R
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

