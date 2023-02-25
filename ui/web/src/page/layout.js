import { Tab, Tabs } from '@mui/material';
import { Box } from '@mui/system';
import React from 'react';
import { Outlet, useParams } from 'react-router-dom';




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