import React from 'react';
import { useNavigate, useParams, useSearchParams } from 'react-router-dom';
import { TabLayout } from '../common/layout';




export default function UserAppBarCodePage() {
    const path = '/user/barcode/'
    const showNav = [
        {
            value: "create",
            name: "生成配置"
        },
        {
            value: "parse",
            name: "解析结果"
        }
    ];
    const [searchParam, _] = useSearchParams();
    let app_id = searchParam.get("app_id") ?? 0;
    const navigate = useNavigate();
    let param = useParams();
    let type = param['*'].split('/')[1];
    return <TabLayout value={type} onChange={
        (event, newValue) => {
            let find = showNav.find((item) => { return item.value == newValue })
            if (find) {
                let url = path + (find.to ? find.to : find.value);
                if (app_id > 0) url += "?app_id=" + app_id;
                navigate(url);
            }
        }
    } menus={showNav} />

}