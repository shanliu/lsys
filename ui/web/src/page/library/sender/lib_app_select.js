import { FormControl, InputLabel, MenuItem } from '@mui/material';
import React, { useContext, useEffect, useState } from 'react';
import { ToastContext } from '../../../context/toast';
import { LoadSelect } from '../../../library/input';
import { appList } from '../../../rest/app';


export function AppSelect(props) {
    const { appId, userId, onChange, onLoad, checkSms, checkMail,...other } = props;
    const [appData, setAppData] = useState({
        init: false,
        loading: false,
        next: true,
        page: 0,
        page_size: 25,
        items: [],
        value: '',
        error: null
    });
    useEffect(() => {
        if (appData.items.length > 0) {
            setAppData({ ...appData, value: appId })
        } else {
            if (appId > 0) {
                setAppData({ ...appData, loading: true })
                appList({
                    status: 2,
                    app_id: appId,
                    page: 0,
                    page_size: 1,
                    user_id: userId,
                    check_sms: checkSms,
                    check_mail: checkMail,
                }).then((data) => {
                    if (!data.status || !data.data[0]) {
                        if (data.message) {
                            toast(data.message)
                        }
                        setAppData({
                            ...appData,
                            loading: false,
                            next: false,
                            error: "app 查找失败"
                        })
                        return;
                    }
                    onLoad && onLoad(data.data[0])
                    setAppData({
                        ...appData,
                        ...data,
                        loading: false,
                        value: appId,
                        items: [...appData.items, ...data.data]
                    })
                })
            }
        }
    }, [props.appId])
    const { toast } = useContext(ToastContext);
    return <FormControl fullWidth  {...other}>
        <InputLabel size="small" id="user-res-select-label">选择App</InputLabel>
        <LoadSelect
            size="small"
            label="选择App"
            labelId="user-app-select-label"
            id="user-app-select"
            loading={appData.loading}
            next={appData.next}
            value={appData.value}
            error={appData.error}
            onChange={(e) => {
                onChange(e)
                let find = appData.items.find((i) => { if (i.id == e.target.value) return i });
                if (!find) return
                onLoad && onLoad(find)
            }}
            onLoad={() => {
                setAppData({ ...appData, loading: true })
                appList({
                    status: 2,
                    user_id: userId,
                    page: appData.page,
                    page_size: appData.page_size,
                    check_sms: checkSms,
                    check_mail: checkMail,
                }).then((data) => {
                    if (!data.status) {
                        setAppData({
                            ...appData,
                            loading: false,
                            next: false,
                            error: appData.items.length > 0 ? null : data.message
                        })
                        return;
                    }
                    let rowData = (data.data ?? []).filter((e) => { return e.id != appId });
                    setAppData({
                        ...appData,
                        items: [...appData.items, ...rowData],
                        loading: false,
                        page: appData.page + 1,
                        next: appData.page * appData.page_size < data.total
                    })
                })
            }}
        >   <MenuItem value=''>全部</MenuItem>
            {appData.items.map((item) => {
                let accok = "";
                if ((checkMail && item.is_mail)
                    || (checkSms && item.is_sms)) {
                    accok = "[已授权]"
                }
                return <MenuItem key={`app-${appId}-${item.id}`} value={item.id}>{item.name}({item.id}){accok}</MenuItem>
            })}
        </LoadSelect>
    </FormControl>
}
