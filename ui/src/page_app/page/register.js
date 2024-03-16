
import { Box, Grid, Link, Paper, Tab, Tabs } from '@mui/material';
import React from 'react';

import { Link as RouterLink, useNavigate, useParams } from 'react-router-dom';

import { SignupFromCode } from './register/code';



export default function RegisterPage() {
    const navigate = useNavigate();
    let { type } = useParams();
    const navData = [
        {
            key: "email",
            name: "邮箱注册",
            props: {
                action: "email",
                label: "邮件",
                sendCaptchaType: "reg-email",
                type: "email",
            },
        },
        {
            key: "mobile",
            name: "手机注册",
            props: {
                action: "mobile",
                label: "手机号",
                sendCaptchaType: "reg-sms",
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
        navigate("/register/" + newValue);
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
                    <SignupFromCode {...item.props} />
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

