package main

import (
	"context"
	"fmt"
	"lsysrest/lsyslib"
	"testing"
)

//权限校验示例

type UserPageCheckRes struct {
}

func (res UserPageCheckRes) ToRbacRes(_ context.Context) [][]map[string]interface{} {
	return [][]map[string]interface{}{{
		map[string]interface{}{
			"res_type":     "user-page", //资源key
			"res_data":     "",          //资源key
			"use_app_user": "0",         //是否使用APP所属的用户,为1时 user_param 被忽略
			"user_param":   "ccc",       //资源用户ID
			"ops": []map[string]string{
				{
					"op_key":   "xx", //操作标识
					"req_auth": "1",  //无符合权限配置时,是否通过授权,为true时,无匹配授权将返回授权失败
				},
			}, //必须权限
		},
	}}
}

type UserPageCheckRelation struct {
}

func (res UserPageCheckRelation) ToCheckRelation(_ context.Context) []map[string]interface{} {
	return []map[string]interface{}{
		{
			"role_key":     "friend", //关系角色KEY
			"user_param":   "ccc",    //关系角色属于用户标识
			"use_app_user": "0",      //是否使用APP所属的用户,为1时 user_param 被忽略
		},
	}
}

func TestRbacCheck(t *testing.T) {
	sysApi := GetRestApi()

	//示例1
	//校验权限
	err1 := sysApi.RbacCheck(
		context.Background(),
		"sss",
		&UserPageCheckRelation{},
		&UserPageCheckRes{},
		nil,
	)
	if err1 == nil {
		fmt.Printf("ok\n")
	} else {
		fmt.Printf("err :%s \n", err1)
	}
}

// 获取RBAC映射数据
func TestRbacMapping(t *testing.T) {
	sysApi := GetRestApi()

	result, err := sysApi.RbacMapping(context.Background())
	if err == nil {
		fmt.Printf("Audit Result:\n")
		for _, item := range result.AuditResult {
			fmt.Printf("  %s: %s\n", item.Key, item.Val)
		}
		fmt.Printf("Role Res Range:\n")
		for _, item := range result.RoleResRange {
			fmt.Printf("  %s: %s\n", item.Key, item.Val)
		}
		fmt.Printf("Role User Range:\n")
		for _, item := range result.RoleUserRange {
			fmt.Printf("  %s: %s\n", item.Key, item.Val)
		}
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

// 批量检查权限
func TestRbacCheckList(t *testing.T) {
	sysApi := GetRestApi()

	items := []lsyslib.AccessCheckItem{
		{
			Name: "test_check",
			CheckRes: map[string]interface{}{
				"user_param": "account_11",
				"token_data": nil,
				"access": map[string]interface{}{
					"role_key": []map[string]interface{}{
						{
							"role_key":     "xxxp",
							"use_app_user": false,
							"user_param":   "account_11",
						},
					},
					"check_res": [][]map[string]interface{}{
						{
							{
								"res_type":     "xx1",
								"res_data":     "",
								"use_app_user": false,
								"user_param":   "account_11",
								"ops": []map[string]interface{}{
									{"op_key": "xx5", "req_auth": "1"},
								},
							},
						},
					},
				},
			},
		},
	}

	results, err := sysApi.RbacCheckList(context.Background(), items)
	if err == nil {
		for _, result := range results {
			fmt.Printf("Name: %s, Status: %s\n", result.Name, result.Status)
		}
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

// ========== Op 操作相关测试 ==========

func TestRbacOpAdd(t *testing.T) {
	sysApi := GetRestApi()

	// 添加操作
	opId, err := sysApi.RbacOpAdd(context.Background(), false, "account_11", "test_op_key", "测试操作")
	if err == nil {
		fmt.Printf("添加操作成功, ID: %s\n", opId)
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacOpList(t *testing.T) {
	sysApi := GetRestApi()

	// 获取操作列表
	result, err := sysApi.RbacOpList(context.Background(), lsyslib.OpListParams{
		UseAppUser: false,
		UserParam:  "account_11",
		CountNum:   true,
		Page:       1,
		Limit:      10,
	})
	if err == nil {
		fmt.Printf("操作列表: %s\n", result)
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacOpEdit(t *testing.T) {
	sysApi := GetRestApi()

	// 编辑操作 (需要有效的op_id)
	err := sysApi.RbacOpEdit(context.Background(), 1, "updated_op_key", "更新后的操作名称")
	if err == nil {
		fmt.Printf("编辑操作成功\n")
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacOpDelete(t *testing.T) {
	sysApi := GetRestApi()

	// 删除操作 (需要有效的op_id)
	err := sysApi.RbacOpDelete(context.Background(), 1)
	if err == nil {
		fmt.Printf("删除操作成功\n")
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

// ========== Res 资源相关测试 ==========

func TestRbacResAdd(t *testing.T) {
	sysApi := GetRestApi()

	// 添加资源
	resId, err := sysApi.RbacResAdd(context.Background(), false, "account_11", "test_res_type", "测试资源", "test_data")
	if err == nil {
		fmt.Printf("添加资源成功, ID: %s\n", resId)
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacResList(t *testing.T) {
	sysApi := GetRestApi()

	// 获取资源列表
	result, err := sysApi.RbacResList(context.Background(), lsyslib.ResListParams{
		UseAppUser: false,
		UserParam:  "account_11",
		CountNum:   true,
		PermCount:  true,
		OpCount:    true,
		Page:       1,
		Limit:      10,
	})
	if err == nil {
		fmt.Printf("资源列表: %s\n", result)
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacResEdit(t *testing.T) {
	sysApi := GetRestApi()

	// 编辑资源 (需要有效的res_id)
	err := sysApi.RbacResEdit(context.Background(), 1, "updated_res_type", "更新后的资源名称", "updated_data")
	if err == nil {
		fmt.Printf("编辑资源成功\n")
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacResDelete(t *testing.T) {
	sysApi := GetRestApi()

	// 删除资源 (需要有效的res_id)
	err := sysApi.RbacResDelete(context.Background(), 1)
	if err == nil {
		fmt.Printf("删除资源成功\n")
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacResTypeData(t *testing.T) {
	sysApi := GetRestApi()

	// 获取资源类型数据
	result, err := sysApi.RbacResTypeData(context.Background(), lsyslib.ResTypeDataParams{
		UseAppUser: false,
		UserParam:  "account_11",
		CountNum:   true,
		Page:       1,
		Limit:      10,
	})
	if err == nil {
		fmt.Printf("资源类型数据: %s\n", result)
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacResTypeOpAdd(t *testing.T) {
	sysApi := GetRestApi()

	// 添加资源类型操作
	err := sysApi.RbacResTypeOpAdd(context.Background(), false, "account_11", "test_res_type", []int{1, 2})
	if err == nil {
		fmt.Printf("添加资源类型操作成功\n")
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacResTypeOpData(t *testing.T) {
	sysApi := GetRestApi()

	// 获取资源类型操作数据
	result, err := sysApi.RbacResTypeOpData(context.Background(), lsyslib.ResTypeOpDataParams{
		UseAppUser: false,
		UserParam:  "account_11",
		ResType:    "test_res_type",
		CountNum:   true,
		Page:       1,
		Limit:      10,
	})
	if err == nil {
		fmt.Printf("资源类型操作数据: %s\n", result)
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacResTypeOpDel(t *testing.T) {
	sysApi := GetRestApi()

	// 删除资源类型操作
	err := sysApi.RbacResTypeOpDel(context.Background(), false, "account_11", "test_res_type", []int{1})
	if err == nil {
		fmt.Printf("删除资源类型操作成功\n")
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

// ========== Role 角色相关测试 ==========

func TestRbacRoleAdd(t *testing.T) {
	sysApi := GetRestApi()

	// 添加角色 (user_range: 1=指定用户, res_range: 1=包含指定授权 2=访问任意资源 3=禁止指定授权)
	roleId, err := sysApi.RbacRoleAdd(context.Background(), false, "account_11", 1, 1, "测试角色", "test_role_key")
	if err == nil {
		fmt.Printf("添加角色成功, ID: %s\n", roleId)
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacRoleList(t *testing.T) {
	sysApi := GetRestApi()

	// 获取角色列表
	result, err := sysApi.RbacRoleList(context.Background(), lsyslib.RoleListParams{
		UseAppUser: false,
		UserParam:  "account_11",
		CountNum:   true,
		UserCount:  true,
		ResCount:   true,
		ResOpCount: true,
		Page:       1,
		Limit:      10,
	})
	if err == nil {
		fmt.Printf("角色列表: %s\n", result)
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacRoleEdit(t *testing.T) {
	sysApi := GetRestApi()

	// 编辑角色 (需要有效的role_id)
	err := sysApi.RbacRoleEdit(context.Background(), false, "account_11", 1, "更新后的角色名称", "updated_role_key")
	if err == nil {
		fmt.Printf("编辑角色成功\n")
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacRoleDelete(t *testing.T) {
	sysApi := GetRestApi()

	// 删除角色 (需要有效的role_id)
	err := sysApi.RbacRoleDelete(context.Background(), false, "account_11", 1)
	if err == nil {
		fmt.Printf("删除角色成功\n")
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

// ========== Role Permission 角色权限相关测试 ==========

func TestRbacRolePermAdd(t *testing.T) {
	sysApi := GetRestApi()

	// 添加角色权限
	permData := []lsyslib.PermData{
		{OpId: 1, ResId: 1},
	}
	err := sysApi.RbacRolePermAdd(context.Background(), false, "account_11", 1, permData)
	if err == nil {
		fmt.Printf("添加角色权限成功\n")
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacRolePermData(t *testing.T) {
	sysApi := GetRestApi()

	// 获取角色权限数据
	result, err := sysApi.RbacRolePermData(context.Background(), lsyslib.RolePermDataParams{
		UseAppUser: false,
		UserParam:  "account_11",
		RoleId:     1,
		CountNum:   true,
		Page:       1,
		Limit:      10,
	})
	if err == nil {
		fmt.Printf("角色权限数据: %s\n", result)
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacRolePermDelete(t *testing.T) {
	sysApi := GetRestApi()

	// 删除角色权限
	permData := []lsyslib.PermData{
		{OpId: 1, ResId: 1},
	}
	err := sysApi.RbacRolePermDelete(context.Background(), false, "account_11", 1, permData)
	if err == nil {
		fmt.Printf("删除角色权限成功\n")
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

// ========== Role User 角色用户相关测试 ==========

func TestRbacRoleUserAdd(t *testing.T) {
	sysApi := GetRestApi()

	// 添加角色用户
	userData := []lsyslib.RoleUserItem{
		{UseAppUser: false, UserParam: "test_user", Timeout: 0},
	}
	err := sysApi.RbacRoleUserAdd(context.Background(), "account_11", 1, userData)
	if err == nil {
		fmt.Printf("添加角色用户成功\n")
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacRoleUserData(t *testing.T) {
	sysApi := GetRestApi()

	// 获取角色用户数据
	result, err := sysApi.RbacRoleUserData(context.Background(), lsyslib.RoleUserDataParams{
		UseAppUser: false,
		UserParam:  "account_11",
		RoleId:     1,
		All:        true,
		CountNum:   true,
		Page:       1,
		Limit:      10,
	})
	if err == nil {
		fmt.Printf("角色用户数据: %s\n", result)
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}

func TestRbacRoleUserDelete(t *testing.T) {
	sysApi := GetRestApi()

	// 删除角色用户
	err := sysApi.RbacRoleUserDelete(context.Background(), false, "account_11", 1, []int{1})
	if err == nil {
		fmt.Printf("删除角色用户成功\n")
	} else {
		fmt.Printf("err: %s\n", err.Error())
	}
}
