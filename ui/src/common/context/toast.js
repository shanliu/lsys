import { Snackbar } from "@mui/material";
import React, { createContext, Fragment, useReducer } from "react";
import IconButton from '@mui/material/IconButton';
import CloseIcon from '@mui/icons-material/Close';
function initializer() {
    return {
        open: false,
        vertical: 'top',
        horizontal: 'right'

    }
}

//toast 上下文
export const ToastContext = createContext(initializer());

const reducer = (state, msg) => {
    if (!msg || msg.length == 0) {
        return {
            ...state,
            message: "",
            open: false,
        }
    } else {
        return {
            ...state,
            message: msg,
            open: true,
        }
    }
}

//提示信息 ToastProvider
export const ToastProvider = props => {
    let [toast, dispatch] = useReducer(reducer, null, initializer)
    const handleClose = (event, reason) => {
        dispatch("")
    };
    const action = (
        <React.Fragment>
            <IconButton color="secondary" size="small" onClick={handleClose}>
                <CloseIcon fontSize="small" />
            </IconButton>
        </React.Fragment>
    );


    return (
        <Fragment>
            <Snackbar
                action={action}
                onClose={handleClose}
                autoHideDuration={6000}
                anchorOrigin={{ vertical: toast.vertical, horizontal: toast.horizontal }}
                open={toast.open}
                message={toast.message}
                key={toast.vertical + toast.horizontal}
                sx={{
                    maxWidth: 0.5
                }}
            />
            <ToastContext.Provider
                value={{
                    toast: dispatch
                }}
            >
                {props.children}
            </ToastContext.Provider >
        </Fragment>
    )
}