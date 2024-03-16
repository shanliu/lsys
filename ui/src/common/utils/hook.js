
import { useSearchParams } from 'react-router-dom';

//设置URL参数,未设置用默认值而非重置为初始值
export function useSearchChange(defaultInit) {
    const [searchParam, setSearchParam] = useSearchParams(defaultInit);
    return [searchParam, (nextInit, noChange, navigateOpts) => {
        if (typeof nextInit != 'object') {
            return setSearchParam(nextInit, navigateOpts)
        }
        let isChange = false;
        Object.keys(nextInit).map((key) => {
            let val = searchParam.get(key);
            val=val===null?'':val;
            val=val===undefined?'':val;
            let tval=nextInit[key];
            tval=tval===null?'':tval;
            tval=tval===undefined?'':tval;
            if (val != tval) {
                isChange = true;
            }
        })
        for (let key of searchParam.keys()) {
            if (!nextInit.hasOwnProperty(key)) {
                let val = searchParam.get(key);
                val=val===null?'':val;
                val=val===undefined?'':val;
                if ((val == '' || !val) && (defaultInit[key] || defaultInit[key] == '')) {
                    continue;
                }
                nextInit[key] = val;
            }
        }
        //过滤掉相同的值
        Object.keys(nextInit).map((key) => {
            let def = typeof defaultInit[key] == 'number' ? (defaultInit[key] + '') : defaultInit[key];
            let set = typeof nextInit[key] == 'number' ? (nextInit[key] + '') : nextInit[key];
            if (set === def ||nextInit[key]===null||nextInit[key]===undefined||nextInit[key]==='') {
                delete nextInit[key];
            }
        })
        if (!isChange && typeof noChange == 'function') {
            noChange()
        }
        return setSearchParam(nextInit, navigateOpts)
    }];
}