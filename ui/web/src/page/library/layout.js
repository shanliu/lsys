import { Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle, Link, Tab, Tabs } from '@mui/material';
import Avatar from '@mui/material/Avatar';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import Tooltip from '@mui/material/Tooltip';
import Typography from '@mui/material/Typography';
import { Box } from '@mui/system';
import { Fragment, default as React } from 'react';
import { Outlet, Link as RLink, useNavigate, useParams } from 'react-router-dom';
import { SessionClear, UserSessionContext } from '../../context/session';
import { logout } from '../../rest/login';
import { LayoutAppBar } from './public';


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
        {anchorElUser ? <Menu
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
                <Typography textAlign="center">
                    <Link component={RLink} to={"/user"}>用户中心</Link>
                </Typography>
            </MenuItem>
            <MenuItem onClick={handleLogout}>
                <Typography textAlign="center">退出登陆</Typography>
            </MenuItem>
        </Menu> : null}

    </Box >
}

export function PageLayout() {
    return <Fragment>
        <LayoutAppBar>
            <Box sx={{ flexGrow: 1, mr: 3, display: { xs: 'none', md: 'flex' }, justifyContent: "flex-end" }}>
                <Button
                    component={RLink}
                    to="/doc"
                    sx={{ my: 2, color: 'white', display: 'block' }}
                >
                    开发文档
                </Button>
            </Box>
            <NavUser />
        </LayoutAppBar >
        <Outlet />
    </Fragment >
}



export function TabLayout(props) {
    let param = useParams();
    let type = param['*'].split('/')[1];
    return (
        <Box >
            <Box sx={{ borderBottom: 1, borderColor: 'divider', marginBottom: 2 }}>
                <Tabs value={type} onChange={props.onChange}>
                    {
                        props.menus.map((e) => {
                            return <Tab label={e.name} value={e.key} key={`nav-${e.key}`} />
                        })
                    }
                </Tabs>
            </Box>
            <Outlet />
        </Box >)

}