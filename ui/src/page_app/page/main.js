import React, { useContext } from 'react';
import { Navigate } from 'react-router';
import { UserSessionContext } from '../../common/context/session';

export default function MainPage() {
    const { userData } = useContext(UserSessionContext)
    if (!userData) {
        return <Navigate to={"/login/main"} />
    } else {
        return <Navigate to={"/user"} />
    }
}

