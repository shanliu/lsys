
import React from 'react';
import { useParams } from 'react-router';
import { NavLink } from 'react-router-dom';

//根据URL判断是否选中的 NavLink
export const ActiveNavLink = React.forwardRef((props, ref) => {
    const { className } = props
    const params = useParams();
    const matchPath = params["*"].split("/")[0];
    let activeClass = "";
    if (typeof matchPath == 'string'
        && matchPath.length > 0
        && (
            props.to.indexOf(props.prefix + matchPath) >= 0
        )) {
        activeClass = "active"
    }
    return <NavLink {...props} ref={ref} className={({ isActive }) => {
        return [className, activeClass].join(" ")
    }} />
});