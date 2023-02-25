import React from 'react';
import { Navigate } from 'react-router';
import { UserSessionContext } from '../context/session';

export default function MainPage() {
    const { userData } = React.useContext(UserSessionContext)
    if (!userData) {
        return <Navigate to={"/login/main"} />
    } else {
        return <Navigate to={"/user"} />
    }
}

