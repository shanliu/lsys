import AppBar from '@mui/material/AppBar';
import Container from '@mui/material/Container';
import Toolbar from '@mui/material/Toolbar';
import logo from "../../../../../server/examples/lsys-actix-web/static/logo.png"
import React from 'react';
import { Box, Link } from '@mui/material';

export function LayoutAppBar(props) {
    return <AppBar position="sticky" sx={{ top: 0, zIndex: (theme) => theme.zIndex.drawer + 1, position: "sticky" }}>
        <Container maxWidth="xl">
            <Toolbar disableGutters>
                <Box sx={{
                    mr: 5
                }} title={"企业内部应用开放平台"}>
                    <Link href={"/app.html"}>
                        <img style={{
                            width: 42,
                            marginTop: 8
                        }} src={logo} />
                    </Link>
                </Box>
                {props.children}
            </Toolbar>
        </Container>
    </AppBar>
}
