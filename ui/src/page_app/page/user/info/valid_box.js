import ShortcutIcon from '@mui/icons-material/Shortcut';
import { Button, Divider, Grid, TextField, Tooltip, Typography } from '@mui/material';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { CaptchaInput } from '../../../../library/input';
import { LoadingButton } from '../../../../library/loading';

export function ConfirmBox(props) {
    const {
        title,
        label,
        button,
        onSubmit,
        onChange,
        name,
        codeError,
        code,
        loading,
        onBack
    } = props;
    return (
        <Fragment>
            <Typography
                align="center"
                variant="subtitle1"
                noWrap
                sx={{
                    mt: 5,
                    mb: 2,
                    fontWeight: 100,
                    alignItems: "center",
                    letterSpacing: '.3rem',
                    color: 'inherit',
                    textDecoration: 'none',
                }}
            >
                {title}
            </Typography>
            <Divider variant="middle" />
            <Form method="post" onSubmit={(e) => {
                e.preventDefault();
                onSubmit();
            }}>
                <Grid
                    sx={{
                        mt: 5,
                    }}
                    container
                    justifyContent="center"
                    alignItems="center"
                >
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label={label}
                            type="text"
                            name="name"
                            size="small"
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            value={name}
                            disabled={loading}
                        />
                        <TextField
                            variant="outlined"
                            label="验证码"
                            type="text"
                            name="code"
                            size="small"
                            onChange={onChange}
                            value={code}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={loading}
                            error={!!codeError}
                            helperText={codeError}
                        />
                    </Grid>

                    <Grid item container xs={10}>
                        <Grid item xs={2}>
                            <Tooltip title={`重新获取验证码`} placement="right">
                                <Button variant="outlined" onClick={onBack} sx={{
                                    width: 10,
                                    minWidth: 50,
                                    paddingBottom: "7px",
                                    textAlign: "center"
                                }}>
                                    <ShortcutIcon size="1em" color="inherit" sx={{
                                        transform: "rotateY(180deg)"
                                    }} />
                                </Button>
                            </Tooltip>
                        </Grid>
                        <Grid item xs={10}>
                            <LoadingButton sx={{
                                width: 1,
                            }} variant="contained" type="submit" loading={loading} disabled={loading} >{button}</LoadingButton>
                        </Grid>
                    </Grid>
                </Grid>
            </Form >
        </Fragment>)
}


export function CodeBox(props) {
    const {
        title,
        label,
        button,
        onSubmit,
        onChange,
        name,
        codeError,
        code,
        loading,
        captchaSrc
    } = props;
    return (
        <Fragment>
            <Typography
                align="center"
                variant="subtitle1"
                noWrap
                sx={{
                    mt: 5,
                    mb: 2,
                    fontWeight: 100,
                    alignItems: "center",
                    letterSpacing: '.3rem',
                    color: 'inherit',
                    textDecoration: 'none',
                }}
            >
                {title}
            </Typography>
            <Divider variant="middle" />
            <Form method="post" onSubmit={(e) => {
                e.preventDefault();
                onSubmit();
            }}>
                <Grid
                    sx={{
                        mt: 5,
                    }}
                    container
                    justifyContent="center"
                    alignItems="center"
                >
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label={label}
                            name="name"
                            size="small"
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            value={name}
                            disabled={loading}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <CaptchaInput
                            value={code}
                            onChange={onChange}
                            src={captchaSrc}
                            variant="outlined"
                            label="验证码"
                            type="text"
                            size="small"
                            required
                            disabled={loading}
                            error={!!codeError}
                            helperText={codeError}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <LoadingButton sx={{
                            width: 1,
                        }} variant="contained" type="submit" loading={loading} disabled={loading} >{button}</LoadingButton>
                    </Grid>
                </Grid>
            </Form ></Fragment>)
}


export function AddBox(props) {
    const {
        title,
        type,
        label,
        placeholder,
        button,
        onSubmit,
        onChange,
        name,
        nameError,
        loading
    } = props;
    return (
        <Fragment>
            <Typography
                align="center"
                variant="subtitle1"
                noWrap
                sx={{
                    mt: 5,
                    mb: 2,
                    fontWeight: 100,
                    alignItems: "center",
                    letterSpacing: '.3rem',
                    color: 'inherit',
                    textDecoration: 'none',
                }}
            >
                {title}
            </Typography>
            <Divider variant="middle" />
            <Form method="post" onSubmit={(e) => {
                e.preventDefault();
                onSubmit();
            }}>
                <Grid
                    sx={{
                        mt: 5,
                    }}
                    container
                    justifyContent="center"
                    alignItems="center"
                >
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label={label}
                            type={type}
                            name="name"
                            size="small"
                            onChange={onChange}
                            value={name}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={loading}
                            placeholder={placeholder}
                            error={!!nameError}
                            helperText={nameError}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <LoadingButton sx={{
                            width: 1,
                        }} variant="contained" type="submit" loading={loading} disabled={loading} >{button}</LoadingButton>
                    </Grid>
                </Grid>
            </Form ></Fragment>)
}