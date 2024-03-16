/* lib_area extension for PHP */

#ifndef PHP_LIB_AREA_H
# define PHP_LIB_AREA_H

extern zend_module_entry lib_area_module_entry;
# define phpext_lib_area_ptr &lib_area_module_entry

# define PHP_LIB_AREA_VERSION "0.1.1"

# if defined(ZTS) && defined(COMPILE_DL_LIB_AREA)
ZEND_TSRMLS_CACHE_EXTERN()
# endif

#define LIB_AREA_NS  "LsExt"
#include "php_version.h"
#if (PHP_VERSION_ID >= 80000)
#include "lib_area_arginfo.h"
#else
#include "lib_area_arginfo_7.h"
#endif



#ifdef PHP_WIN32
#include <windows.h>
SRWLOCK lock;
#else
#include <pthread.h>
pthread_rwlock_t lock;
#endif






#endif	/* PHP_LIB_AREA_H */
