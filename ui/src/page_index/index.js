import { default as React, Fragment, useContext, useEffect, useState, useRef } from 'react';
import { createRoot } from 'react-dom/client';
import Starback from 'starback'
import GitHubIcon from '@mui/icons-material/GitHub';
import "../common/style/index.css";
import logo from "../../../server/examples/lsys-actix-web/static/logo.png"
export default function IndexAppPage() {
    let bg = useRef(null)
    useEffect(() => {
        if (bg) {
            new Starback(bg.current, {
                type: 'dot',
                width: window.innerWidth,
                height: window.innerHeight,
                quantity: 80,
                direction: 240,
                backgroundColor: ['#0e1118', '#232b3e'],
                randomOpacity: true,
            })
        }
    }, [])
    return <Fragment >

        <canvas className='home_bg' ref={bg}></canvas>
        <header className='home_head'>
            <div className='home_logo'> <img style={{
                width: 42,
                marginTop: 8
            }} src={logo} /></div>
            <div className='home_menu'>
                <a href="https://github.com/shanliu/lsys">
                    <GitHubIcon fontSize='small' />
                    <span>源码</span>
                </a>
            </div>
        </header>
        <div className="content">
            <h1>轻量级～开放平台及内部应用管理系统</h1>
            <div className="buttons">
                <a href="app.html" className="btn btn-light">在线示例</a>
                <a href="app.html#/doc" className="btn btn-light">开发文档</a>
            </div>
        </div>

    </Fragment>
}


const container = document.getElementById('root');
const root = createRoot(container);
root.render(<IndexAppPage />);

