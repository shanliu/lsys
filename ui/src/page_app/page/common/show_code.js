import { Box, Button, Dialog, DialogActions, DialogContent, DialogTitle } from '@mui/material';
const CodeEditor = React.lazy(() => import('@uiw/react-textarea-code-editor'));
import React, { Fragment, useState } from 'react';

export function ShowCode(props) {
    const { title, dataCallback, language, ...other } = props
    const [showBox, setShowBox] = useState({
        open: false,
        code: ''
    });
    return <Fragment>
        <Dialog
            open={showBox.open}
            onClose={() => { setShowBox({ ...showBox, open: false }) }}
        >
            <DialogTitle>{title}</DialogTitle>
            <DialogContent {...other}>
                <CodeEditor
                    minHeight={180}
                    language={language}
                    value={showBox.code}
                    style={{
                        fontSize: 12,
                        backgroundColor: "#f5f5f5",
                        fontFamily: 'ui-monospace,SFMono-Regular,SF Mono,Consolas,Liberation Mono,Menlo,monospace',
                    }}
                    readOnly={true}
                />
            </DialogContent>
            <DialogActions>

                <Button onClick={() => { setShowBox({ ...showBox, open: false }) }} >
                    关闭
                </Button>
            </DialogActions>
        </Dialog>
        <Box onClick={() => {
            setShowBox({ code: dataCallback(), open: true })
        }
        }>{props.children}</Box>

    </Fragment>


}