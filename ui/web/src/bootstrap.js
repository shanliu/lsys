import { Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle } from '@mui/material';
import AppBar from '@mui/material/AppBar';
import Avatar from '@mui/material/Avatar';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Container from '@mui/material/Container';
import IconButton from '@mui/material/IconButton';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import Toolbar from '@mui/material/Toolbar';
import Tooltip from '@mui/material/Tooltip';
import Typography from '@mui/material/Typography';
import React, { Fragment, useEffect, useState } from 'react';
import { Outlet, useNavigate } from 'react-router';
import { Link, useRouteError } from 'react-router-dom';
import { SessionClear, UserSessionContext } from './context/session';
import { logout } from './rest/login';




function NavUser() {
    const { userData, dispatch } = React.useContext(UserSessionContext)
    const [anchorElUser, setAnchorElUser] = React.useState(null);
    const handleOpenUserMenu = (event) => {
        setAnchorElUser(event.currentTarget);
    };
    const handleCloseUserMenu = () => {
        setAnchorElUser(null);
    };
    const [outMsg, setOutMsg] = React.useState({
        open: false,
        msg: null
    });
    const navigate = useNavigate();
    const handleLogout = () => {
        logout().then((data) => {
            if (data.status) {
                dispatch({ type: SessionClear })
                navigate("/")
            } else {
                setOutMsg({ msg: data.message ?? "系统异常", open: true })
            }
        });
    };
    const cancelLogout = () => {
        setOutMsg({ ...outMsg, open: false })
    }
    if (!userData) return;
    return <Box sx={{ flexGrow: 0 }}>
        <Dialog
            open={outMsg.open}
            onClose={cancelLogout}

        >
            <DialogTitle>
                系统退出异常
            </DialogTitle>
            <DialogContent sx={{
                minWidth: 350
            }}>
                <DialogContentText>
                    {outMsg.msg}
                </DialogContentText>
            </DialogContent>
            <DialogActions>
                <Button onClick={cancelLogout}>
                    取消退出
                </Button>
                <Button onClick={() => {
                    dispatch({ type: SessionClear })
                    navigate("/")
                }} autoFocus>
                    继续退出
                </Button>
            </DialogActions>
        </Dialog>
        <Tooltip>
            <IconButton onClick={handleOpenUserMenu} sx={{ p: 0 }}>
                <Avatar alt={userData.user_data?.name?.username} />
            </IconButton>
        </Tooltip>
        <Menu
            sx={{ mt: '45px' }}
            id="menu-appbar"
            anchorEl={anchorElUser}
            anchorOrigin={{
                vertical: 'top',
                horizontal: 'right',
            }}
            keepMounted
            transformOrigin={{
                vertical: 'top',
                horizontal: 'right',
            }}
            open={Boolean(anchorElUser)}
            onClose={handleCloseUserMenu}
        >
            <MenuItem onClick={handleCloseUserMenu}>
                <Typography textAlign="center">用户中心</Typography>
            </MenuItem>
            <MenuItem onClick={handleLogout}>
                <Typography textAlign="center">退出登陆</Typography>
            </MenuItem>
        </Menu>
    </Box >
}

export function LayoutAppBar(props) {
    return <AppBar position="static" sx={{ zIndex: (theme) => theme.zIndex.drawer + 1, position: "relative" }}>
        <Container maxWidth="xl">
            <Toolbar disableGutters>
                <Typography
                    variant="h6"
                    noWrap
                    component={Link}
                    to="/"
                    sx={{
                        mr: 2,
                        display: { xs: 'none', md: 'flex' },
                        fontFamily: 'monospace',
                        fontWeight: 700,
                        letterSpacing: '.3rem',
                        color: 'inherit',
                        textDecoration: 'none',
                    }}
                >
                    LSYS
                </Typography>
                <Box sx={{ flexGrow: 1, display: { xs: 'none', md: 'flex' } }}>
                    {/* <Button
                    sx={{ my: 2, color: 'white', display: 'block' }}
                >
                    解析二维码
                </Button> */}
                </Box>
                {props.children}
            </Toolbar>
        </Container>
    </AppBar>
}

export function Layout() {
    return <Fragment>
        <LayoutAppBar>
            <NavUser />
        </LayoutAppBar >
        <Outlet />
    </Fragment>
}

export function ErrorPage() {
    const error = useRouteError();//错误信息
    const [go, setGo] = useState(10)
    useEffect(() => {
        let a = setInterval(() => {
            if (go - 1 <= 0) {
                window.location.reload();
                return;
            }
            setGo(go - 1)
        }, 1000);
        return () => {
            clearInterval(a)
        }
    })
    return (
        <Box id="error-page">
            <h1>发生错误!</h1>
            <p>你请求的页码当前不可用,请稍后再尝试.错误信息如下:</p>
            <p>
                <i>{error.statusText || error.message}</i>
            </p>
            {<Box>将在 {go} 秒后刷新本页面</Box>}
        </Box>
    );
}
