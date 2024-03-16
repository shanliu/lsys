import { Button, Dialog, DialogActions, DialogContent, DialogContentText } from '@mui/material';
import React, { Fragment, useContext, useState } from 'react';
import { ToastContext } from '../context/toast';
import { LoadingButton } from '../../library/loading';
import PropTypes from 'prop-types';

//待确认的按钮
export function ConfirmButton(props) {
    const { renderButton, message, onAction, ...other } = props
    const { toast } = useContext(ToastContext);
    const [delBox, setDelBox] = useState({
        open: false,
        loading: false,
    });
    return <Fragment>
        <Dialog
            open={delBox.open}
            onClose={() => { setDelBox({ ...delBox, open: false }) }}
        >
            <DialogContent sx={{
                minWidth: 350
            }}>
                <DialogContentText>
                    {message}
                </DialogContentText>
            </DialogContent>
            <DialogActions>
                <LoadingButton loading={delBox.loading} disabled={delBox.loading} onClick={() => {
                    setDelBox({ ...delBox, loading: true })
                    onAction().then((data) => {
                        if (data.status) {
                            setDelBox({ ...delBox, loading: false, open: false })
                        } else {
                            setDelBox({ ...delBox, loading: false })
                            toast(data.message || "系统异常");
                        }
                    })
                }} autoFocus >确认</LoadingButton>
                <Button onClick={() => { setDelBox({ ...delBox, open: false }) }} >
                    取消
                </Button>
            </DialogActions>
        </Dialog>
        {renderButton({
            ...other,
            onClick: () => {
                setDelBox({ ...delBox, open: true })
            }
        })}
    </Fragment>;
}

ConfirmButton.propTypes = {
    message: PropTypes.any.isRequired,
    renderButton: PropTypes.func.isRequired,
    onAction: PropTypes.func.isRequired,
};
