import { CircularProgress } from '@mui/material';
import Button from '@mui/material/Button';
import PropTypes from 'prop-types';
import React from 'react';


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
