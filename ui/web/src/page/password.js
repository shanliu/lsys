import { Box, Grid, Link, Paper, Tab, Tabs } from '@mui/material';
import React from 'react';

import { Link as RouterLink, useNavigate, useParams } from 'react-router-dom';
import { FindPasswordBox } from './password/find';

export default function FindPasswordPage() {
    const navigate = useNavigate();
    let { type } = useParams();
    const navData = [
        {
            key: "email",
            name: "通过邮箱找回密码",
            props: {
                action: "email",
                label: "邮件",
                sendCaptchaType: "reset-password-send-mail",
                type: "email",
            },
        },
        {
            key: "mobile",
            name: "通过短信找回密码",
            props: {
                action: "mobile",
                label: "手机号",
                sendCaptchaType: "reset-password-send-sms",
                type: "text",
            },
        }
    ];
    let item = navData.find((e) => {
        if (e.key == type) {
            return true
        }
    });
    if (!item) {
        return;
    }

    const handleChange = (event, newValue) => {
        navigate("/find_password/" + newValue);
    };

    return (
        <div className='center_page'>
            <Paper elevation={0}
                sx={{
                    margin: 'auto',
                    maxWidth: 500,
                    paddingTop: 2,
                    paddingBottom: 5,
                }}
            >
                <Box sx={{ width: '90%', margin: 'auto' }}>
                    <Box sx={{ borderBottom: 1, borderColor: 'divider', marginBottom: 3 }}>
                        <Tabs value={type} onChange={handleChange}>
                            {
                                navData.map((e) => {
                                    return <Tab label={e.name} value={e.key} key={`nav-${e.key}`} />
                                })
                            }
                        </Tabs>
                    </Box>
                    <FindPasswordBox {...item.props} />
                    <Grid
                        container
                        justifyContent="center"
                        alignItems="center"
                        sx={{
                            marginTop: 2
                        }}
                    >
                        <Grid item xs={10} direction="row"
                            justifyContent="space-between"
                            alignItems="center" container>
                            <Link component={RouterLink} to="/login/name" underline="none"
                                variant="body2" >返回登陆</Link >
                        </Grid>

                    </Grid>
                </Box>
            </Paper>
        </div >
    );
}
