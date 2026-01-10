package lsyslib

import (
	"context"
	"net/http"
	"rest_client"
	"time"
)

const (
	// Base
	AccessCheck     = 400
	AccessCheckList = 401
	AccessMapping   = 402
	// Op
	OpAdd    = 410
	OpEdit   = 411
	OpDelete = 412
	OpList   = 413
	// Res
	ResAdd       = 420
	ResEdit      = 421
	ResDelete    = 422
	ResList      = 423
	ResTypeData  = 424
	ResTypeOpAdd = 425
	ResTypeOpData = 426
	ResTypeOpDel  = 427
	// Role
	RoleAdd           = 430
	RoleEdit          = 431
	RoleDelete        = 432
	RoleList          = 433
	RolePermAdd       = 434
	RolePermData      = 435
	RolePermDelete    = 436
	RoleUserAddApi    = 437
	RoleUserDataApi   = 438
	RoleUserDeleteApi = 439
)

func init() {
	RestApiClientSetConfig(map[int]rest_client.RestBuild{
		// Base
		AccessCheck: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/base",
			Method:     "access",
			Timeout:    60 * time.Second,
		},
		AccessCheckList: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/base",
			Method:     "access_list",
			Timeout:    60 * time.Second,
		},
		AccessMapping: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/base",
			Method:     "mapping",
			Timeout:    60 * time.Second,
		},
		// Op
		OpAdd: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/op",
			Method:     "add",
			Timeout:    60 * time.Second,
		},
		OpEdit: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/op",
			Method:     "edit",
			Timeout:    60 * time.Second,
		},
		OpDelete: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/op",
			Method:     "delete",
			Timeout:    60 * time.Second,
		},
		OpList: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/op",
			Method:     "list",
			Timeout:    60 * time.Second,
		},
		// Res
		ResAdd: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/res",
			Method:     "add",
			Timeout:    60 * time.Second,
		},
		ResEdit: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/res",
			Method:     "edit",
			Timeout:    60 * time.Second,
		},
		ResDelete: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/res",
			Method:     "delete",
			Timeout:    60 * time.Second,
		},
		ResList: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/res",
			Method:     "list",
			Timeout:    60 * time.Second,
		},
		ResTypeData: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/res",
			Method:     "type_data",
			Timeout:    60 * time.Second,
		},
		ResTypeOpAdd: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/res",
			Method:     "type_op_add",
			Timeout:    60 * time.Second,
		},
		ResTypeOpData: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/res",
			Method:     "type_op_data",
			Timeout:    60 * time.Second,
		},
		ResTypeOpDel: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/res",
			Method:     "type_op_del",
			Timeout:    60 * time.Second,
		},
		// Role
		RoleAdd: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/role",
			Method:     "add",
			Timeout:    60 * time.Second,
		},
		RoleEdit: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/role",
			Method:     "edit",
			Timeout:    60 * time.Second,
		},
		RoleDelete: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/role",
			Method:     "delete",
			Timeout:    60 * time.Second,
		},
		RoleList: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/role",
			Method:     "list",
			Timeout:    60 * time.Second,
		},
		RolePermAdd: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/role",
			Method:     "perm_add",
			Timeout:    60 * time.Second,
		},
		RolePermData: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/role",
			Method:     "perm_data",
			Timeout:    60 * time.Second,
		},
		RolePermDelete: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/role",
			Method:     "perm_delete",
			Timeout:    60 * time.Second,
		},
		RoleUserAddApi: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/role",
			Method:     "user_add",
			Timeout:    60 * time.Second,
		},
		RoleUserDataApi: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/role",
			Method:     "user_data",
			Timeout:    60 * time.Second,
		},
		RoleUserDeleteApi: &RestClientBuild{
			Payload:    http.MethodPost,
			HttpMethod: http.MethodPost,
			Path:       "/rest/rbac/role",
			Method:     "user_delete",
			Timeout:    60 * time.Second,
		},
	})
}

// CheckRes 资源接口
type CheckRes interface {
	ToRbacRes(ctx context.Context) [][]map[string]interface{}
}

// CheckRelation 校验权限关系
type CheckRelation interface {
	ToCheckRelation(ctx context.Context) []map[string]interface{}
}
type EmptyCheckRelation struct {
}

func (EmptyCheckRelation) ToCheckRelation(_ context.Context) []map[string]interface{} {
	return make([]map[string]interface{}, 0)
}

// RbacCheck 权限校验
func (receiver *LsysApi) RbacCheck(ctx context.Context, userId string, relation CheckRelation, checkRes CheckRes, tokenData *string) error {
	data1 := (<-receiver.rest.Do(ctx, AccessCheck, map[string]interface{}{
		"user_param": userId,
		"token_data": tokenData,
		"access": map[string]interface{}{
			"role_key":  relation.ToCheckRelation(ctx),
			"check_res": checkRes.ToRbacRes(ctx),
		},
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}

// AccessCheckItem 批量检查权限项
type AccessCheckItem struct {
	Name     string
	CheckRes map[string]interface{}
}

// AccessCheckListResult 批量检查权限结果
type AccessCheckListResult struct {
	Name   string
	Status string
}

// RbacCheckList 批量权限校验
func (receiver *LsysApi) RbacCheckList(ctx context.Context, menuRes []AccessCheckItem) ([]AccessCheckListResult, error) {
	items := make([]map[string]interface{}, 0)
	for _, item := range menuRes {
		items = append(items, map[string]interface{}{
			"name":      item.Name,
			"check_res": item.CheckRes,
		})
	}
	data1 := (<-receiver.rest.Do(ctx, AccessCheckList, map[string]interface{}{
		"menu_res": items,
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	var results []AccessCheckListResult
	for _, item := range data1.GetData("response.result").Array() {
		results = append(results, AccessCheckListResult{
			Name:   item.Get("name").String(),
			Status: item.Get("status").String(),
		})
	}
	return results, nil
}

// MappingItem 映射项
type MappingItem struct {
	Key string
	Val string
}

// RbacMappingResult 映射数据结果
type RbacMappingResult struct {
	AuditResult   []MappingItem
	RoleResRange  []MappingItem
	RoleUserRange []MappingItem
}

// RbacMapping 获取基础映射数据
func (receiver *LsysApi) RbacMapping(ctx context.Context) (*RbacMappingResult, error) {
	data1 := (<-receiver.rest.Do(ctx, AccessMapping, map[string]interface{}{})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	result := &RbacMappingResult{}
	for _, item := range data1.GetData("response.audit_result").Array() {
		result.AuditResult = append(result.AuditResult, MappingItem{
			Key: item.Get("key").String(),
			Val: item.Get("val").String(),
		})
	}
	for _, item := range data1.GetData("response.role_res_range").Array() {
		result.RoleResRange = append(result.RoleResRange, MappingItem{
			Key: item.Get("key").String(),
			Val: item.Get("val").String(),
		})
	}
	for _, item := range data1.GetData("response.role_user_range").Array() {
		result.RoleUserRange = append(result.RoleUserRange, MappingItem{
			Key: item.Get("key").String(),
			Val: item.Get("val").String(),
		})
	}
	return result, nil
}

// ========== Op 操作相关接口 ==========

// RbacOpAdd 添加操作
func (receiver *LsysApi) RbacOpAdd(ctx context.Context, useAppUser bool, userParam string, opKey string, opName string) (string, error) {
	data1 := (<-receiver.rest.Do(ctx, OpAdd, map[string]interface{}{
		"use_app_user": useAppUser,
		"user_param":   userParam,
		"op_key":       opKey,
		"op_name":      opName,
	})).JsonResult()
	if data1.Err() != nil {
		return "", data1.Err()
	}
	return data1.GetData("response.id").String(), nil
}

// RbacOpEdit 编辑操作
func (receiver *LsysApi) RbacOpEdit(ctx context.Context, opId int, opKey string, opName string) error {
	data1 := (<-receiver.rest.Do(ctx, OpEdit, map[string]interface{}{
		"op_id":   opId,
		"op_key":  opKey,
		"op_name": opName,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}

// RbacOpDelete 删除操作
func (receiver *LsysApi) RbacOpDelete(ctx context.Context, opId int) error {
	data1 := (<-receiver.rest.Do(ctx, OpDelete, map[string]interface{}{
		"op_id": opId,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}

// OpListParams 操作列表参数
type OpListParams struct {
	UseAppUser   bool
	UserParam    string
	OpName       *string
	OpKey        *string
	Ids          []int
	CountNum     bool
	ResTypeCount bool
	CheckRoleUse bool
	Page         int
	Limit        int
}

// RbacOpList 操作列表
func (receiver *LsysApi) RbacOpList(ctx context.Context, params OpListParams) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, OpList, map[string]interface{}{
		"use_app_user":   params.UseAppUser,
		"user_param":     params.UserParam,
		"op_name":        params.OpName,
		"op_key":         params.OpKey,
		"ids":            params.Ids,
		"count_num":      params.CountNum,
		"res_type_count": params.ResTypeCount,
		"check_role_use": params.CheckRoleUse,
		"page": map[string]interface{}{
			"page":  params.Page,
			"limit": params.Limit,
		},
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
}

// ========== Res 资源相关接口 ==========

// RbacResAdd 添加资源
func (receiver *LsysApi) RbacResAdd(ctx context.Context, useAppUser bool, userParam string, resType string, resName string, resData string) (string, error) {
	data1 := (<-receiver.rest.Do(ctx, ResAdd, map[string]interface{}{
		"use_app_user": useAppUser,
		"user_param":   userParam,
		"res_type":     resType,
		"res_name":     resName,
		"res_data":     resData,
	})).JsonResult()
	if data1.Err() != nil {
		return "", data1.Err()
	}
	return data1.GetData("response.id").String(), nil
}

// RbacResEdit 编辑资源
func (receiver *LsysApi) RbacResEdit(ctx context.Context, resId int, resType string, resName string, resData string) error {
	data1 := (<-receiver.rest.Do(ctx, ResEdit, map[string]interface{}{
		"res_id":   resId,
		"res_type": resType,
		"res_name": resName,
		"res_data": resData,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}

// RbacResDelete 删除资源
func (receiver *LsysApi) RbacResDelete(ctx context.Context, resId int) error {
	data1 := (<-receiver.rest.Do(ctx, ResDelete, map[string]interface{}{
		"res_id": resId,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}

// ResListParams 资源列表参数
type ResListParams struct {
	UseAppUser bool
	UserParam  string
	ResType    *string
	ResData    *string
	ResName    *string
	PermCount  bool
	OpCount    bool
	Ids        []int
	CountNum   bool
	Page       int
	Limit      int
}

// RbacResList 资源列表
func (receiver *LsysApi) RbacResList(ctx context.Context, params ResListParams) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, ResList, map[string]interface{}{
		"use_app_user": params.UseAppUser,
		"user_param":   params.UserParam,
		"res_type":     params.ResType,
		"res_data":     params.ResData,
		"res_name":     params.ResName,
		"perm_count":   params.PermCount,
		"op_count":     params.OpCount,
		"ids":          params.Ids,
		"count_num":    params.CountNum,
		"page": map[string]interface{}{
			"page":  params.Page,
			"limit": params.Limit,
		},
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
}

// ResTypeDataParams 资源类型数据参数
type ResTypeDataParams struct {
	UseAppUser bool
	UserParam  string
	ResType    *string
	CountNum   bool
	Page       int
	Limit      int
}

// RbacResTypeData 获取资源类型数据
func (receiver *LsysApi) RbacResTypeData(ctx context.Context, params ResTypeDataParams) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, ResTypeData, map[string]interface{}{
		"use_app_user": params.UseAppUser,
		"user_param":   params.UserParam,
		"res_type":     params.ResType,
		"count_num":    params.CountNum,
		"page": map[string]interface{}{
			"page":  params.Page,
			"limit": params.Limit,
		},
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
}

// RbacResTypeOpAdd 添加资源类型操作
func (receiver *LsysApi) RbacResTypeOpAdd(ctx context.Context, useAppUser bool, userParam string, resType string, opIds []int) error {
	data1 := (<-receiver.rest.Do(ctx, ResTypeOpAdd, map[string]interface{}{
		"use_app_user": useAppUser,
		"user_param":   userParam,
		"res_type":     resType,
		"op_ids":       opIds,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}

// ResTypeOpDataParams 资源类型操作数据参数
type ResTypeOpDataParams struct {
	UseAppUser bool
	UserParam  string
	ResType    string
	CountNum   bool
	Page       int
	Limit      int
}

// RbacResTypeOpData 获取资源类型操作数据
func (receiver *LsysApi) RbacResTypeOpData(ctx context.Context, params ResTypeOpDataParams) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, ResTypeOpData, map[string]interface{}{
		"use_app_user": params.UseAppUser,
		"user_param":   params.UserParam,
		"res_type":     params.ResType,
		"count_num":    params.CountNum,
		"page": map[string]interface{}{
			"page":  params.Page,
			"limit": params.Limit,
		},
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
}

// RbacResTypeOpDel 删除资源类型操作
func (receiver *LsysApi) RbacResTypeOpDel(ctx context.Context, useAppUser bool, userParam string, resType string, opIds []int) error {
	data1 := (<-receiver.rest.Do(ctx, ResTypeOpDel, map[string]interface{}{
		"use_app_user": useAppUser,
		"user_param":   userParam,
		"res_type":     resType,
		"op_ids":       opIds,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}

// ========== Role 角色相关接口 ==========

// RbacRoleAdd 添加角色
func (receiver *LsysApi) RbacRoleAdd(ctx context.Context, useAppUser bool, userParam string, userRange int, resRange int, roleName string, roleKey string) (string, error) {
	data1 := (<-receiver.rest.Do(ctx, RoleAdd, map[string]interface{}{
		"use_app_user": useAppUser,
		"user_param":   userParam,
		"user_range":   userRange,
		"res_range":    resRange,
		"role_name":    roleName,
		"role_key":     roleKey,
	})).JsonResult()
	if data1.Err() != nil {
		return "", data1.Err()
	}
	return data1.GetData("response.id").String(), nil
}

// RbacRoleEdit 编辑角色
func (receiver *LsysApi) RbacRoleEdit(ctx context.Context, useAppUser bool, userParam string, roleId int, roleName string, roleKey string) error {
	data1 := (<-receiver.rest.Do(ctx, RoleEdit, map[string]interface{}{
		"use_app_user": useAppUser,
		"user_param":   userParam,
		"role_id":      roleId,
		"role_name":    roleName,
		"role_key":     roleKey,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}

// RbacRoleDelete 删除角色
func (receiver *LsysApi) RbacRoleDelete(ctx context.Context, useAppUser bool, userParam string, roleId int) error {
	data1 := (<-receiver.rest.Do(ctx, RoleDelete, map[string]interface{}{
		"use_app_user": useAppUser,
		"user_param":   userParam,
		"role_id":      roleId,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}

// RoleListParams 角色列表参数
type RoleListParams struct {
	UseAppUser bool
	UserParam  string
	RoleName   *string
	RoleKey    *string
	UserData   *string
	UserCount  bool
	Ids        []int
	UserRange  *int
	ResRange   *int
	ResCount   bool
	ResOpCount bool
	CountNum   bool
	Page       int
	Limit      int
}

// RbacRoleList 角色列表
func (receiver *LsysApi) RbacRoleList(ctx context.Context, params RoleListParams) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, RoleList, map[string]interface{}{
		"use_app_user": params.UseAppUser,
		"user_param":   params.UserParam,
		"role_name":    params.RoleName,
		"role_key":     params.RoleKey,
		"user_data":    params.UserData,
		"user_count":   params.UserCount,
		"ids":          params.Ids,
		"user_range":   params.UserRange,
		"res_range":    params.ResRange,
		"res_count":    params.ResCount,
		"res_op_count": params.ResOpCount,
		"count_num":    params.CountNum,
		"page": map[string]interface{}{
			"page":  params.Page,
			"limit": params.Limit,
		},
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
}

// PermData 权限数据
type PermData struct {
	OpId  int
	ResId int
}

// RbacRolePermAdd 添加角色权限
func (receiver *LsysApi) RbacRolePermAdd(ctx context.Context, useAppUser bool, userParam string, roleId int, permData []PermData) error {
	perms := make([]map[string]interface{}, 0)
	for _, perm := range permData {
		perms = append(perms, map[string]interface{}{
			"op_id":  perm.OpId,
			"res_id": perm.ResId,
		})
	}
	data1 := (<-receiver.rest.Do(ctx, RolePermAdd, map[string]interface{}{
		"use_app_user": useAppUser,
		"user_param":   userParam,
		"role_id":      roleId,
		"perm_data":    perms,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}

// RolePermDataParams 角色权限数据参数
type RolePermDataParams struct {
	UseAppUser bool
	UserParam  string
	RoleId     int
	CountNum   bool
	Page       int
	Limit      int
}

// RbacRolePermData 获取角色权限数据
func (receiver *LsysApi) RbacRolePermData(ctx context.Context, params RolePermDataParams) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, RolePermData, map[string]interface{}{
		"use_app_user": params.UseAppUser,
		"user_param":   params.UserParam,
		"role_id":      params.RoleId,
		"count_num":    params.CountNum,
		"page": map[string]interface{}{
			"page":  params.Page,
			"limit": params.Limit,
		},
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
}

// RbacRolePermDelete 删除角色权限
func (receiver *LsysApi) RbacRolePermDelete(ctx context.Context, useAppUser bool, userParam string, roleId int, permData []PermData) error {
	perms := make([]map[string]interface{}, 0)
	for _, perm := range permData {
		perms = append(perms, map[string]interface{}{
			"op_id":  perm.OpId,
			"res_id": perm.ResId,
		})
	}
	data1 := (<-receiver.rest.Do(ctx, RolePermDelete, map[string]interface{}{
		"use_app_user": useAppUser,
		"user_param":   userParam,
		"role_id":      roleId,
		"perm_data":    perms,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}

// RoleUserItem 角色用户数据
type RoleUserItem struct {
	UseAppUser bool
	UserParam  string
	Timeout    int
}

// RbacRoleUserAdd 添加角色用户
func (receiver *LsysApi) RbacRoleUserAdd(ctx context.Context, userParam string, roleId int, userData []RoleUserItem) error {
	users := make([]map[string]interface{}, 0)
	for _, user := range userData {
		users = append(users, map[string]interface{}{
			"use_app_user": user.UseAppUser,
			"user_param":   user.UserParam,
			"timeout":      user.Timeout,
		})
	}
	data1 := (<-receiver.rest.Do(ctx, RoleUserAddApi, map[string]interface{}{
		"user_param": userParam,
		"role_id":    roleId,
		"user_data":  users,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}

// RoleUserDataParams 角色用户数据参数
type RoleUserDataParams struct {
	UseAppUser bool
	UserParam  string
	RoleId     int
	All        bool
	CountNum   bool
	Page       int
	Limit      int
}

// RbacRoleUserData 获取角色用户数据
func (receiver *LsysApi) RbacRoleUserData(ctx context.Context, params RoleUserDataParams) (*rest_client.JsonData, error) {
	data1 := (<-receiver.rest.Do(ctx, RoleUserDataApi, map[string]interface{}{
		"use_app_user": params.UseAppUser,
		"user_param":   params.UserParam,
		"role_id":      params.RoleId,
		"all":          params.All,
		"count_num":    params.CountNum,
		"page": map[string]interface{}{
			"page":  params.Page,
			"limit": params.Limit,
		},
	})).JsonResult()
	if data1.Err() != nil {
		return nil, data1.Err()
	}
	return data1.GetData("response"), nil
}

// RbacRoleUserDelete 删除角色用户
func (receiver *LsysApi) RbacRoleUserDelete(ctx context.Context, useAppUser bool, userParam string, roleId int, userIds []int) error {
	data1 := (<-receiver.rest.Do(ctx, RoleUserDeleteApi, map[string]interface{}{
		"use_app_user": useAppUser,
		"user_param":   userParam,
		"role_id":      roleId,
		"user_data":    userIds,
	})).JsonResult()
	if data1.Err() != nil {
		return data1.Err()
	}
	return nil
}
