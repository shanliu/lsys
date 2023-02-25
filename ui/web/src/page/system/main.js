
import { Alert } from '@mui/material';
import Box from '@mui/material/Box';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import React, { useContext, useEffect, useState } from 'react';
import { Navigate, Outlet, useParams } from 'react-router';
import { UserSessionContext } from '../../context/session';
import { Progress } from '../../library/loading';
import { ActiveNavLink } from '../../library/nav';
import { accessMenu } from '../../rest/access';
import { Menus } from './menu';


export default function SystemMainPage() {
    const { userData } = useContext(UserSessionContext)
    let param = useParams()//从请求url中获取数据
    if (!userData) {
        return <Navigate to={"/login/main"} />
    }

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

    return loadMenu.loading ? <Progress /> : loadMenu.menu.length == 0 ? <Alert severity="warning">未开通任何管理功能</Alert> : <Box sx={{ display: 'flex' }}>
        <Box sx={{ minWidth: 200, minHeight: "calc(100vh - 69px)", borderRight: " 1px solid #eee" }}>
            <List>
                {loadMenu.menu.map((item) => {
                    const Icon = item.icon;
                    return <ListItem key={`system-${item.url}`} disablePadding>
                        <ListItemButton prefix="/system/" component={ActiveNavLink} to={item.url}>
                            <ListItemIcon>
                                <Icon />
                            </ListItemIcon>
                            <ListItemText primary={item.text} />
                        </ListItemButton>
                    </ListItem>;
                })}
            </List>
        </Box><Box component="main" sx={{ flexGrow: 1, p: 3 }}>
            <Outlet />
        </Box>
    </Box >
        ;
}