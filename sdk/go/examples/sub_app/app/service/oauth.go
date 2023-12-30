package service

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"lsysrest/lsysrest"
	"rest_client"

	"github.com/gin-contrib/sessions"
	"github.com/gin-gonic/gin"
)

func GetLoginUrl(c *gin.Context, callUrl string) (error, string) {
	b := make([]byte, 6)
	_, err := rand.Read(b)
	if err != nil {
		return err, ""
	}
	state := hex.EncodeToString(b)
	url := GetRestApi().OAuthAuthorizationUrl(context.Background(), callUrl, "user_info,user_mobile", state)
	session := sessions.Default(c)
	session.Set("oauth-state", state)
	session.Save()
	return nil, url
}

func GetToken(c *gin.Context, state string, code string) (error, *lsysrest.TokenData) {
	session := sessions.Default(c)
	tmp, ok := session.Get("oauth-state").(string)
	if ok && tmp != state {
		return errors.New("state wrong"), nil
	}
	return GetRestApi().OAuthAccessToken(context.Background(), code)
}

func GetUserData(token string) (error, *rest_client.JsonData) {
	tokenApi := GetRestApi().TokenRestApi(token)
	return tokenApi.OAuthUserInfo(context.Background(), true, true, false, false, false, false)

}

func RefreshToken(token string) (error, *lsysrest.TokenData) {
	tokenApi := GetRestApi().TokenRestApi(token)
	return tokenApi.OAuthRefreshToken(context.Background())
}
