import React, { Fragment } from 'react';
import { CircularProgress, LinearProgress, Skeleton, Typography } from '@mui/material';
import Button from '@mui/material/Button';
import PropTypes from 'prop-types';
import { Box } from '@mui/system';


//带加载图标的按钮
export const LoadingButton = (props) => {
    const { loading, children, ...other } = props;
    if (loading) {
        return (
            <Button {...other}>
                <CircularProgress size="1em" sx={{
                    marginRight: 1,
                }} color="inherit" /> {children}
            </Button>
        );
    } else {
        return (
            <Button {...other}  >{children}</Button>
        );
    }
}

LoadingButton.defaultProps = {
    loading: false,
};

LoadingButton.propTypes = {
    loading: PropTypes.bool
};


//进度条
export const Progress = (props) => {
    return <LinearProgress {...props} />
}


//页面进度条
export const PageProgress = (props) => {
    return  <Fragment>
        <LinearProgress {...props} />
        <Box sx={{ m: 2 }}>
            <Typography variant="h1"> <Skeleton /></Typography>
        </Box>
    </Fragment>
}
