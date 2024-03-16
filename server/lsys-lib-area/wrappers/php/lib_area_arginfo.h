/* This is a generated file, edit the .stub.php file instead.
 * Stub hash: 70f014f50b90cc141945ffecb39da46362ef325e */

ZEND_BEGIN_ARG_WITH_RETURN_TYPE_INFO_EX(arginfo_class_LsExt_LibArea_initCsv, 0, 2, IS_VOID, 0)
	ZEND_ARG_TYPE_INFO(0, code_path, IS_STRING, 0)
	ZEND_ARG_TYPE_INFO(0, geo_path, IS_STRING, 0)
	ZEND_ARG_TYPE_INFO_WITH_DEFAULT_VALUE(0, index_path, IS_STRING, 0, "\"\"")
	ZEND_ARG_TYPE_INFO_WITH_DEFAULT_VALUE(0, index_size, IS_LONG, 0, "0")
	ZEND_ARG_TYPE_INFO_WITH_DEFAULT_VALUE(0, gz, _IS_BOOL, 0, "true")
ZEND_END_ARG_INFO()

ZEND_BEGIN_ARG_WITH_RETURN_TYPE_INFO_EX(arginfo_class_LsExt_LibArea_initSqlite, 0, 1, IS_VOID, 0)
	ZEND_ARG_TYPE_INFO(0, sqlite_sql, IS_STRING, 0)
	ZEND_ARG_TYPE_INFO_WITH_DEFAULT_VALUE(0, index_path, IS_STRING, 0, "\"\"")
	ZEND_ARG_TYPE_INFO_WITH_DEFAULT_VALUE(0, index_size, IS_LONG, 0, "0")
ZEND_END_ARG_INFO()

ZEND_BEGIN_ARG_WITH_RETURN_TYPE_INFO_EX(arginfo_class_LsExt_LibArea_initMysql, 0, 1, IS_VOID, 0)
	ZEND_ARG_TYPE_INFO(0, uri, IS_STRING, 0)
	ZEND_ARG_TYPE_INFO_WITH_DEFAULT_VALUE(0, index_path, IS_STRING, 0, "\"\"")
	ZEND_ARG_TYPE_INFO_WITH_DEFAULT_VALUE(0, index_size, IS_LONG, 0, "0")
ZEND_END_ARG_INFO()

ZEND_BEGIN_ARG_WITH_RETURN_TYPE_INFO_EX(arginfo_class_LsExt_LibArea_shutdown, 0, 0, IS_VOID, 0)
ZEND_END_ARG_INFO()

#define arginfo_class_LsExt_LibArea_geoReload arginfo_class_LsExt_LibArea_shutdown

#define arginfo_class_LsExt_LibArea_codeReload arginfo_class_LsExt_LibArea_shutdown

ZEND_BEGIN_ARG_WITH_RETURN_TYPE_INFO_EX(arginfo_class_LsExt_LibArea_codeChilds, 0, 1, IS_ARRAY, 0)
	ZEND_ARG_TYPE_INFO(0, code, IS_STRING, 0)
ZEND_END_ARG_INFO()

ZEND_BEGIN_ARG_WITH_RETURN_TYPE_INFO_EX(arginfo_class_LsExt_LibArea_codeSearch, 0, 1, IS_ARRAY, 0)
	ZEND_ARG_TYPE_INFO(0, code, IS_STRING, 0)
	ZEND_ARG_TYPE_INFO_WITH_DEFAULT_VALUE(0, limit, IS_LONG, 0, "10")
ZEND_END_ARG_INFO()

#define arginfo_class_LsExt_LibArea_codeFind arginfo_class_LsExt_LibArea_codeChilds

#define arginfo_class_LsExt_LibArea_codeRelated arginfo_class_LsExt_LibArea_codeChilds

ZEND_BEGIN_ARG_WITH_RETURN_TYPE_INFO_EX(arginfo_class_LsExt_LibArea_geoSearch, 0, 2, IS_ARRAY, 0)
	ZEND_ARG_TYPE_INFO(0, lat, IS_DOUBLE, 0)
	ZEND_ARG_TYPE_INFO(0, lng, IS_DOUBLE, 0)
ZEND_END_ARG_INFO()


ZEND_METHOD(LsExt_LibArea, initCsv);
ZEND_METHOD(LsExt_LibArea, initSqlite);
ZEND_METHOD(LsExt_LibArea, initMysql);
ZEND_METHOD(LsExt_LibArea, shutdown);
ZEND_METHOD(LsExt_LibArea, geoReload);
ZEND_METHOD(LsExt_LibArea, codeReload);
ZEND_METHOD(LsExt_LibArea, codeChilds);
ZEND_METHOD(LsExt_LibArea, codeSearch);
ZEND_METHOD(LsExt_LibArea, codeFind);
ZEND_METHOD(LsExt_LibArea, codeRelated);
ZEND_METHOD(LsExt_LibArea, geoSearch);


static const zend_function_entry class_LsExt_Exception_methods[] = {
	ZEND_FE_END
};


static const zend_function_entry class_LsExt_LibArea_methods[] = {
	ZEND_ME(LsExt_LibArea, initCsv, arginfo_class_LsExt_LibArea_initCsv, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, initSqlite, arginfo_class_LsExt_LibArea_initSqlite, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, initMysql, arginfo_class_LsExt_LibArea_initMysql, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, shutdown, arginfo_class_LsExt_LibArea_shutdown, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, geoReload, arginfo_class_LsExt_LibArea_geoReload, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, codeReload, arginfo_class_LsExt_LibArea_codeReload, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, codeChilds, arginfo_class_LsExt_LibArea_codeChilds, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, codeSearch, arginfo_class_LsExt_LibArea_codeSearch, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, codeFind, arginfo_class_LsExt_LibArea_codeFind, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, codeRelated, arginfo_class_LsExt_LibArea_codeRelated, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_ME(LsExt_LibArea, geoSearch, arginfo_class_LsExt_LibArea_geoSearch, ZEND_ACC_PUBLIC|ZEND_ACC_STATIC)
	ZEND_FE_END
};
