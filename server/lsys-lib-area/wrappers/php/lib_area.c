/* lib_area extension for PHP */

#ifdef HAVE_CONFIG_H
# include "config.h"
#endif
#include "php.h"
#include "ext/standard/info.h"
#include "php_lib_area.h"
#include "lib_area_class.h"


#ifdef PHP_WIN32
#include <windows.h>
extern SRWLOCK lock;
#else
#include <pthread.h>
extern pthread_rwlock_t lock;
#endif



/* {{{ PHP_MINIT_FUNCTION
 */
PHP_MINIT_FUNCTION(lib_area)
{
	#ifdef _WIN32
        InitializeSRWLock(&lock);
	#else
		pthread_rwlock_init(&lock, NULL);
	#endif

	lib_area_class_init();
	return SUCCESS;
}
/* }}} */

/* {{{ PHP_MINIT_FUNCTION
 */
PHP_MSHUTDOWN_FUNCTION(lib_area)
{
#ifdef _WIN32

#else
	pthread_rwlock_destroy(&lock);
#endif

    return SUCCESS;
}
/* }}} */

/* {{{ PHP_RINIT_FUNCTION */
PHP_RINIT_FUNCTION(lib_area)
{
#if defined(ZTS) && defined(COMPILE_DL_LIB_AREA)
	ZEND_TSRMLS_CACHE_UPDATE();
#endif

	return SUCCESS;
}
/* }}} */

/* {{{ PHP_MINFO_FUNCTION */
PHP_MINFO_FUNCTION(lib_area)
{
	php_info_print_table_start();
	php_info_print_table_header(2, "area db support", "enabled");
	php_info_print_table_end();
}
/* }}} */

/* {{{ lib_area_module_entry */
zend_module_entry lib_area_module_entry = {
	STANDARD_MODULE_HEADER,
	"lib_area",					/* Extension name */
	NULL,					/* zend_function_entry */
	PHP_MINIT(lib_area),							/* PHP_MINIT - Module initialization */
    PHP_MSHUTDOWN(lib_area),							/* PHP_MSHUTDOWN - Module shutdown */
	PHP_RINIT(lib_area),			/* PHP_RINIT - Request initialization */
	NULL,							/* PHP_RSHUTDOWN - Request shutdown */
	PHP_MINFO(lib_area),			/* PHP_MINFO - Module info */
	PHP_LIB_AREA_VERSION,		/* Version */
	STANDARD_MODULE_PROPERTIES
};
/* }}} */

#ifdef COMPILE_DL_LIB_AREA
# ifdef ZTS
ZEND_TSRMLS_CACHE_DEFINE()
# endif
ZEND_GET_MODULE(lib_area)
#endif
