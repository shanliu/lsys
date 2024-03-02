import styled from '@emotion/styled';
import Tooltip, { tooltipClasses } from '@mui/material/Tooltip';
import React from 'react';


//公共的鼠标指向提示
export const ItemTooltip = styled(({ className, ...props }) => (
    <Tooltip {...props} arrow classes={{ popper: className }} />
))(({ theme }) => ({
    [`& .${tooltipClasses.arrow}`]: {
        color: theme.palette ? theme.palette.grey[500] : '#ccc',
    },
    [`& .${tooltipClasses.tooltip}`]: {
        backgroundColor: theme.palette ? theme.palette.grey[500] : '#ccc',
        top: "8px"
    },
}));