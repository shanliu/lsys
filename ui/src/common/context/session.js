
import React, { createContext, useContext, useReducer } from "react";
import { useUpdateEffect } from 'usehooks-ts';
import { userSessionClear, userSessionGet, userSessionSet } from "../utils/rest";
import { loginData } from "../rest/login";
import { ConfigContext, ConfigTipPassword } from "./config";
function initializer() {
    return userSessionGet()
}


//登录信息上下文
export const UserSessionContext = createContext({
    reload: false,
    //   login_token: null,
    jwt_header: null,
    user_data: null,
    session: false
});

export const SessionReload = 'reload'
export const SessionClear = 'clear'
export const SessionSet = 'set'

//设置登录信息辅助函数,生成 SessionSet 用
export const SessionSetData = (auth_data, keep_login) => {
    return {
        type: SessionSet,
        playload: {
            keep: keep_login,
            user: auth_data,
            jwt: auth_data.jwt
        }
    }
}
//清除登录信息辅助函数,生成 SessionClear 用
export const SessionClearData = () => {
    return {
        type: SessionClear
    }
}
//重新加载登录信息辅助函数,生成 SessionReload 用
export const SessionReloadData = () => {
    return {
        type: SessionReload
    }
}

//登录信息处理 reducer 
const reducer = (data, action) => {
    switch (action.type) {
        case SessionReload:
            return {
                ...data,
                reload: true
            };
        case SessionClear:
            return null;
        case SessionSet:
            if (!action.playload?.user || !action.playload?.jwt) {
                return null;
            }
            return {
                reload: false,
                //  login_token: action.playload.user.token,
                jwt_header: action.playload.jwt,
                user_data: action.playload.user.auth_data,
                session: !action.playload.keep
            }
    }
}

//用户登录信息 UserProvider
export const UserProvider = (props) => {
    const configctx = useContext(ConfigContext)
    let [userData, dispatch] = useReducer(reducer, null, initializer)
    useUpdateEffect(() => {
        if (!userData) {
            userSessionClear()
        } else {
            if (userData.reload) {
                loginData({
                    auth: true,
                    password_timeout: true
                }).then((data) => {
                    if (data.status && data.auth_data) {
                        dispatch({
                            type: SessionSet,
                            playload: {
                                keep: !userData.session,
                                user: {
                                    //    token: userData.login_token,
                                    auth_data: data.auth_data,
                                },
                                jwt: data.jwt
                            }
                        })
                        if (data.password_timeout) {
                            configctx.dispatch(ConfigTipPassword())
                        }
                    } else {
                        dispatch({
                            type: SessionSet,
                            playload: {
                                keep: !userData.session,
                                user: {
                                    //    token: userData.login_token,
                                    auth_data: userData.user_data,
                                },
                                jwt: userData.jwt_header
                            }
                        })
                    }
                })
            } else {
                userSessionSet(userData)
            }
        }
    }, [userData])
    return (
        <UserSessionContext.Provider
            value={{
                userData: userData,
                dispatch: dispatch
            }}
        >
            {props.children}
        </UserSessionContext.Provider>
    )
}

