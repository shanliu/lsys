/*
 * Author: shanliu  <shan.liu@msn.com>
 */
#include "zend.h"
#include "zend_API.h"
#include "zend_exceptions.h"
#include "php_lib_area.h"
#include "lib_area_exception_class.h"


zend_class_entry *lib_area_exception_ce_ptr;
void lib_area_exception_class_init(){
    zend_class_entry ce;
    INIT_NS_CLASS_ENTRY(ce,LIB_AREA_NS,"Exception",class_LsExt_Exception_methods);
    lib_area_exception_ce_ptr = zend_register_internal_class_ex(&ce, zend_ce_exception);
}

