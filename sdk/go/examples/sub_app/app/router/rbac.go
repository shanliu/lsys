package router

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"html/template"
	"lsysrest/lsysrest"
	"net/http"
	"strings"
	"sub_app/app/service"

	"github.com/gin-gonic/gin"
)

type UserPageCheckRes struct {
}

func (res UserPageCheckRes) ToRbacRes(_ context.Context) [][]map[string]interface{} {
	//这里定义访问资源数据.
	//这里可以实现为查数据库或配置等,最后组装出以下MAP数据即可
	return [][]map[string]interface{}{
		//以下两个其中一个校验通过即通过
		{
			//以下两个校验必须都通过才算通过
			map[string]interface{}{
				"res":     "view-user-page", //资源key
				"user_id": 0,                //0为系统资源
				"ops":     []string{"view"}, //必须权限
			},
			map[string]interface{}{
				"res":        "user-page",              //资源key
				"user_id":    1,                        //资源用户ID
				"ops":        []string{"view", "edit"}, //必须权限
				"option_ops": []string{"review"},       //可选权限,当系统有配置改资源时,进入校验,未配置时不校验
			},
		},
		{
			map[string]interface{}{
				"res":     "admin-user-page", //资源key
				"user_id": 0,                 //0为系统资源
				"ops":     []string{"edit"},  //必须权限
			},
		},
	}
}

type UserPageCheckRelation struct {
}

func (res UserPageCheckRelation) ToCheckRelation(_ context.Context) []map[string]interface{} {
	//这里定义跟访问资源的所属用户的关系
	//这里可以实现为查数据库或配置等,最后组装出以下MAP数据即可
	return []map[string]interface{}{
		{
			"role_key": "friend", //关系角色KEY
			"user_id":  2,        //关系角色属于用户ID,如用户间粉丝关系
		},
		{
			"role_key": "vip", //关系角色KEY
			"user_id":  0,     //为0时,该关系为用户跟系统关系
		},
	}
}

func AppRbac(c *gin.Context) {
	//校验权限
	resStr, _ := json.MarshalIndent(UserPageCheckRes{}.ToRbacRes(c), "", "   ")
	relStr, _ := json.MarshalIndent(UserPageCheckRelation{}.ToCheckRelation(c), "", "   ")
	param := gin.H{
		"res": string(resStr),
		"rel": string(relStr),
	}
	if c.Request.Method == "POST" {
		_, e := c.GetQuery("no_relation")
		var err error
		if e {
			err = service.GetRestApi().RbacCheck(
				c,
				1,
				&lsysrest.EmptyCheckRelation{}, //无任何关系用
				&UserPageCheckRes{},            //资源定义,自定义实现
			)
		} else {
			err = service.GetRestApi().RbacCheck(
				c,
				100,
				&UserPageCheckRelation{}, //自定义访问用户1跟访问资源的所属用户的关系
				&UserPageCheckRes{},      //资源定义,自定义实现
			)
		}
		if err == nil {
			param["status"] = "通过"
			param["info"] = "无"
		} else {
			var info []string
			var tmpe *lsysrest.RestClientError
			ok := errors.As(err, &tmpe)
			if ok {
				for _, tmp := range tmpe.Data.Get("check_detail").Array() {
					for ckey, cval := range tmp.Map() {
						info = append(info, fmt.Sprintf("校验资源[%s]失败,详细:<br/>%s <br/>", ckey, cval.String()))
					}

				}
			}
			param["status"] = "失败: " + err.Error()
			param["info"] = template.HTML(strings.Join(info, "<br/>"))
		}
	}
	c.HTML(http.StatusOK, "rbac.html", param)

}
