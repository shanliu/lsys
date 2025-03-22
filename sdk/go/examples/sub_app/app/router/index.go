package router

import (
	"net/http"
	"sub_app/app/service"

	"github.com/gin-gonic/gin"
)

func Index(c *gin.Context) {
	url, err := service.GetLoginUrl(c, "http://"+c.Request.Host+"/callback")
	if err != nil {
		service.ErrorPage(c, err.Error())
		return
	}
	c.HTML(http.StatusOK, "index.html", gin.H{
		"oauth_url": url,
	})
}
