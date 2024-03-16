//read image class

ZEND_BEGIN_ARG_INFO_EX(arginfo_class_LibArea_reload_arginfo, 0, 0, 0)
ZEND_END_ARG_INFO()

ZEND_BEGIN_ARG_INFO_EX(lib_area_code_childs_arginfo, 0, 0, 1)
ZEND_ARG_INFO(0, code)
ZEND_END_ARG_INFO()


ZEND_BEGIN_ARG_INFO_EX(lib_area_code_detail_arginfo, 0, 0, 1)
ZEND_ARG_INFO(0, code)
ZEND_END_ARG_INFO()


ZEND_BEGIN_ARG_INFO_EX(lib_area_code_search_arginfo, 0, 0, 1)
ZEND_ARG_INFO(0, code)
ZEND_ARG_INFO(0, limit)
ZEND_END_ARG_INFO()

ZEND_BEGIN_ARG_INFO_EX(area_geo_search_arginfo, 0, 0, 2)
ZEND_ARG_INFO(0, lat)
ZEND_ARG_INFO(0, lng)
ZEND_END_ARG_INFO()




ZEND_BEGIN_ARG_INFO_EX(arginfo_class_LibArea_initCsv, 0,0, 2)
	ZEND_ARG_INFO(0,code_path)
	ZEND_ARG_INFO(0, geo_path)
	ZEND_ARG_INFO(0, index_path)
	ZEND_ARG_INFO(0, index_size)
	ZEND_ARG_INFO(0, gz)
ZEND_END_ARG_INFO()

ZEND_BEGIN_ARG_INFO_EX(arginfo_class_LibArea_initSqlite, 0, 0,1)
	ZEND_ARG_INFO(0, sqlite_sql)
	ZEND_ARG_INFO(0, index_path)
	ZEND_ARG_INFO(0, index_size)
ZEND_END_ARG_INFO()

ZEND_BEGIN_ARG_INFO_EX(arginfo_class_LibArea_initMysql, 0, 0,1)
	ZEND_ARG_INFO(0, uri)
	ZEND_ARG_INFO(0, index_path)
	ZEND_ARG_INFO(0, index_size)
ZEND_END_ARG_INFO()

ZEND_BEGIN_ARG_INFO_EX(arginfo_class_LibArea_shutdown, 0, 0, 0)
ZEND_END_ARG_INFO()

ZEND_BEGIN_ARG_INFO_EX(arginfo_class_LibArea_codeChilds, 0, 0,  1)
	ZEND_ARG_INFO(0, code)
ZEND_END_ARG_INFO()

ZEND_BEGIN_ARG_INFO_EX(arginfo_class_LibArea_codeDetail, 0, 0,  1)
ZEND_ARG_INFO(0, code)
ZEND_END_ARG_INFO()


ZEND_BEGIN_ARG_INFO_EX(arginfo_class_LibArea_codeFind, 0, 0,  1)
ZEND_ARG_INFO(0, code)
ZEND_END_ARG_INFO()


ZEND_BEGIN_ARG_INFO_EX(arginfo_class_LibArea_codeSearch, 0, 0,  1)
	ZEND_ARG_INFO(0, code)
	ZEND_ARG_INFO(0, limit)
ZEND_END_ARG_INFO()


ZEND_BEGIN_ARG_INFO_EX(arginfo_class_LibArea_geoSearch, 0,0, 2)
	ZEND_ARG_INFO(0, lat)
	ZEND_ARG_INFO(0, lng)
ZEND_END_ARG_INFO()


ZEND_METHOD(LsExt_LibArea, initCsv);
ZEND_METHOD(LsExt_LibArea, initSqlite);
ZEND_METHOD(LsExt_LibArea, initMysql);
ZEND_METHOD(LsExt_LibArea, shutdown);
ZEND_METHOD(LsExt_LibArea, codeReload);
ZEND_METHOD(LsExt_LibArea, geoReload);
ZEND_METHOD(LsExt_LibArea, codeChilds);
ZEND_METHOD(LsExt_LibArea, codeSearch);
ZEND_METHOD(LsExt_LibArea, codeFind);
ZEND_METHOD(LsExt_LibArea, codeRelated);
ZEND_METHOD(LsExt_LibArea, geoSearch);

static const zend_function_entry class_LsExt_Exception_methods[] = {
		ZEND_FE_END
};

static const zend_function_entry class_LsExt_LibArea_methods[] = {
	ZEND_ME(LsExt_LibArea, initCsv, arginfo_class_LibArea_initCsv, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, initSqlite, arginfo_class_LibArea_initSqlite, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, initMysql, arginfo_class_LibArea_initMysql, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, shutdown, arginfo_class_LibArea_shutdown, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, geoReload, arginfo_class_LibArea_reload_arginfo, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, codeReload, arginfo_class_LibArea_reload_arginfo, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, codeChilds, arginfo_class_LibArea_codeChilds, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, codeSearch, arginfo_class_LibArea_codeSearch, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, codeFind, arginfo_class_LibArea_codeFind, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, codeRelated, arginfo_class_LibArea_codeDetail, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, geoSearch, arginfo_class_LibArea_geoSearch, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_FE_END
};
