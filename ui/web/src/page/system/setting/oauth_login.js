import React from 'react';
import PropTypes from 'prop-types';
import Tabs from '@mui/material/Tabs';
import Tab from '@mui/material/Tab';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import { Form } from 'react-router-dom';
import { Paper, Stack, TextField } from '@mui/material';
import { LoadingButton } from '../../../library/loading';
export default function SystemSettingOauthLoginPage() {
    return <VerticalTabs />
}


function TabPanel(props) {
    const { children, value, index, ...other } = props;

    return (
        <div
            role="tabpanel"
            hidden={value !== index}
            id={`vertical-tabpanel-${index}`}
            aria-labelledby={`vertical-tab-${index}`}
            {...other}
        >
            {value === index && (
                <Box sx={{ p: 3 }}>
                    <Typography>{children}</Typography>
                </Box>
            )}
        </div>
    );
}

TabPanel.propTypes = {
    children: PropTypes.node,
    index: PropTypes.number.isRequired,
    value: PropTypes.number.isRequired,
};

function a11yProps(index) {
    return {
        id: `vertical-tab-${index}`,
        'aria-controls': `vertical-tabpanel-${index}`,
    };
}

function VerticalTabs() {
    const [value, setValue] = React.useState(0);

    const handleChange = (event, newValue) => {
        setValue(newValue);
    };

    return <Paper
        component="form"
        sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1, minWidth: 900 }}
    >
        <Box
            sx={{ flexGrow: 1, bgcolor: 'background.paper', display: 'flex' }}
        >
            <Tabs
                orientation="vertical"
                variant="scrollable"
                value={value}
                onChange={handleChange}
                aria-label="Vertical tabs example"
                sx={{ borderRight: 1, mt: 3, borderColor: 'divider', minWidth: 120 }}
            >
                <Tab label="微信登陆" {...a11yProps(0)} />
                <Tab label="QQ登陆" {...a11yProps(1)} />
            </Tabs>
            <TabPanel value={value} index={0}>
                <Box sx={{
                    width: 300,
                }}>
                    <Form method="post" >
                        <Stack>
                            <TextField
                                sx={{
                                    paddingBottom: 2
                                }}
                                label="微信 app id"
                                variant="outlined"
                                name="name"
                                size="small"
                                fullWidth
                                onChange={(e) => {

                                }}

                                required
                            />

                            <TextField
                                label="微信 app secret"
                                variant="outlined"
                                name="name"
                                size="small"
                                fullWidth
                                onChange={(e) => {

                                }}
                                sx={{
                                    paddingBottom: 2
                                }}
                                required
                            />

                            <LoadingButton variant="contained">保存</LoadingButton>
                        </Stack>
                    </Form>
                </Box>
            </TabPanel>
            <TabPanel value={value} index={1}>
                <Box sx={{
                    width: 300,
                }}>
                    <Form method="post" >
                        <Stack>
                            <TextField
                                sx={{
                                    paddingBottom: 2
                                }}
                                label="QQ app id"
                                variant="outlined"
                                name="name"
                                size="small"
                                fullWidth
                                onChange={(e) => {

                                }}

                                required
                            />

                            <TextField
                                label="QQ app secret"
                                variant="outlined"
                                name="name"
                                size="small"
                                fullWidth
                                onChange={(e) => {

                                }}
                                sx={{
                                    paddingBottom: 2
                                }}
                                required
                            />

                            <LoadingButton variant="contained">保存</LoadingButton>
                        </Stack>
                    </Form>
                </Box>
            </TabPanel>
        </Box >
    </Paper >
        ;
}