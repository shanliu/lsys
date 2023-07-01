import { Box, FormControl, InputLabel, MenuItem, Typography } from '@mui/material';
import React, { useContext, useEffect, useState } from 'react';
import { ToastContext } from '../../../context/toast';
import { LoadSelect } from '../../../library/input';



export function TplSelect(props) {
    const { tplId, userId, onChange, onLoad, loadData, MenuProps, ...other } = props;
    const [tplData, setTplData] = useState({
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
        if (tplData.items.length > 0) {
            setTplData({ ...tplData, value: tplId })
        } else {
            if (tplId > 0) {
                setTplData({ ...tplData, loading: true })
                loadData({
                    page: 0,
                    page_size: 1,
                    id: parseInt(tplId),
                    user_id: userId,
                    app_info: true
                }).then((data) => {
                    if (!data.status || !data.data[0]) {
                        if (data.message) {
                            toast(data.message)
                        }
                        setTplData({
                            ...tplData,
                            loading: false,
                            next: false,
                            error: "模板查找失败"
                        })
                        return;
                    }
                    onLoad && onLoad(data.data[0])
                    setTplData({
                        ...tplData,
                        ...data,
                        loading: false,
                        value: tplId,
                        items: [...tplData.items, ...data.data]
                    })
                })
            }
        }
    }, [props.tplId])
    const { toast } = useContext(ToastContext);
    return <FormControl fullWidth  {...other}>
        <InputLabel size="small" >选择模板</InputLabel>
        <LoadSelect
            MenuProps={MenuProps}
            size="small"
            label="选择模板"
            loading={tplData.loading}
            next={tplData.next}
            value={tplData.value}
            error={tplData.error}
            onChange={(e) => {
                onChange(e)
                let find = tplData.items.find((i) => { if (i.id == e.target.value) return i });
                if (!find) return
                onLoad && onLoad(find)
            }}
            onLoad={() => {
                setTplData({ ...tplData, loading: true })
                loadData({
                    page: tplData.page,
                    page_size: tplData.page_size,
                    user_id: userId,
                    app_info: true
                }).then((data) => {
                    if (!data.status) {
                        setTplData({
                            ...tplData,
                            loading: false,
                            next: false,
                            error: tplData.items.length > 0 ? null : data.message
                        })
                        return;
                    }
                    let rowData = (data.data ?? []).filter((e) => { return e.id != tplId });
                    setTplData({
                        ...tplData,
                        items: [...tplData.items, ...rowData],
                        loading: false,
                        page: tplData.page + 1,
                        next: tplData.page * tplData.page_size < data.total
                    })
                })
            }}
        >
            {tplData.items.map((item) => {
                return <MenuItem key={`tpl-select-${item.id}`} value={item.id}>
                    <Box>
                        <Typography variant="button" display="block" gutterBottom>
                            应用({item.app_id}):{item.app_name} 模板:{item.name}
                        </Typography>
                        <Typography variant="caption" display="block" gutterBottom>
                            模板ID:{item.tpl_id} 配置:({item.setting_key}){item.setting_name}
                        </Typography>
                    </Box>
                </MenuItem>
            })}
        </LoadSelect>
    </FormControl>
}
