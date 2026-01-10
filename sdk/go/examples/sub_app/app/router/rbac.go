package router

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"html/template"
	"lsysrest/lsyslib"
	"net/http"
	"strings"
	"sub_app/app/service"

	"github.com/gin-gonic/gin"
)

// 本系统 通过接口 实现 本系统的RBAC权限管理

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
				"res_type":     "view-user-page", //资源key
				"res_data":     "1",              //资源key
				"use_app_user": "0",              //是否使用APP所属的用户,为1时 user_param 被忽略
				"user_param":   "ccc",            //资源用户ID
				"ops": []map[string]string{
					{
						"op_key":   "view", //操作标识
						"req_auth": "1",    //无符合权限配置时,是否通过授权,为true时,无匹配授权将返回授权失败
					},
				},
			},
			map[string]interface{}{
				"res_type":     "global-user-page", //资源key
				"res_data":     "",                 //资源key
				"use_app_user": "0",                //是否使用APP所属的用户,为1时 user_param 被忽略
				"user_param":   "ccc",              //资源用户ID
				"ops": []map[string]string{
					{
						"op_key":   "view", //操作标识
						"req_auth": "1",    //无符合权限配置时,是否通过授权,为true时,无匹配授权将返回授权失败
					},
					{
						"op_key":   "edit", //操作标识
						"req_auth": "1",    //无符合权限配置时,是否通过授权,为true时,无匹配授权将返回授权失败
					},
				},
			},
		},
		{
			map[string]interface{}{
				"res_type":     "admin-user-page", //资源key
				"res_data":     "",                //资源key
				"use_app_user": "0",               //是否使用APP所属的用户,为1时 user_param 被忽略
				"user_param":   "ccc",             //资源用户ID
				"ops": []map[string]string{
					{
						"op_key":   "view", //操作标识
						"req_auth": "1",    //无符合权限配置时,是否通过授权,为true时,无匹配授权将返回授权失败
					},
				},
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
			"role_key":     "friend", //关系角色KEY
			"user_param":   "ccc",    //关系角色属于用户标识
			"use_app_user": "0",      //是否使用APP所属的用户,为1时 user_param 被忽略
		},
		{
			"role_key":     "vip", //关系角色KEY
			"use_app_user": "1",   //是否使用APP所属的用户,为1时 user_param 被忽略
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
				"",
				&lsyslib.EmptyCheckRelation{}, //无任何关系用
				&UserPageCheckRes{},           //资源定义,自定义实现
				nil,
			)
		} else {
			err = service.GetRestApi().RbacCheck(
				c,
				"100",
				&UserPageCheckRelation{}, //自定义访问用户1跟访问资源的所属用户的关系
				&UserPageCheckRes{},      //资源定义,自定义实现
				nil,
			)
		}
		if err == nil {
			param["status"] = "通过"
			param["info"] = "无"
		} else {
			var info []string
			var tmpe *lsyslib.RestClientError
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
