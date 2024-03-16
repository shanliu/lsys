PHP_ARG_WITH([lib_area],
    [for lib_area support],
    [AS_HELP_STRING([--with-lib_area],[Include lib_area support])])

PHP_ARG_WITH(lib_area_dir,
    for lib_area,
    [AS_HELP_STRING([--with-lib_area_dir=PATH],[set area db lib dir])],
    yes)

PHP_ARG_WITH(lib_area_use_sqlite,
    for lib_area,
    [AS_HELP_STRING([--with-lib_area_use_sqlite],[enable sqlite data source support, You need to enable data-sqlite in cargo.toml first.])],
    no)
PHP_ARG_WITH(lib_area_use_mysql,
    for lib_area,
    [AS_HELP_STRING([--with-lib_area_use_mysql],[enable mysql data source support, You need to enable data-mysql in cargo.toml first.])],
    no)

if test "$PHP_LIB_AREA" != "no"; then

   PHP_ADD_LIBRARY(pthread)

   if test "$PHP_LIB_AREA_DIR" = "yes"; then
   		if test -r ./lib/lsys_lib_area.h;
   		then
   			PHP_ADD_INCLUDE(./lib)
   		else
   			AC_MSG_ERROR([area header file not found,please use the liblsys_lib_area.so directory with --with-lib_area_dir])
   		fi
        if test -r ./lib/liblsys_lib_area.so;
        then
            PHP_ADD_LIBRARY_WITH_PATH(lib_area, ./lib/, LIB_AREA_SHARED_LIBADD)
        else
            AC_MSG_ERROR([lib_area lib path ./lib/ not find liblsys_lib_area.so,please use the liblsys_lib_area.so directory with --with-lib_area_dir])
        fi
   	else
   		if test -r $PHP_LIB_AREA_DIR/lsys_lib_area.h;
   		then
   			PHP_ADD_INCLUDE($PHP_LIB_AREA_DIR)
   		else
   			AC_MSG_ERROR([area db path not find lsys_lib_area.h])
   		fi
   		if test -r $PHP_LIB_AREA_DIR/liblsys_lib_area.so;
        then
            PHP_ADD_LIBRARY_WITH_PATH(lsys_lib_area, $PHP_LIB_AREA_DIR, LIB_AREA_SHARED_LIBADD)
        else
            AC_MSG_ERROR([lib_area lib path $PHP_LIB_AREA_DIR not find liblsys_lib_area.so])
        fi
   	fi
   PHP_SUBST(LIB_AREA_SHARED_LIBADD)

   if test "$PHP_LIB_AREA_USE_SQLITE" != "no"; then
        AC_DEFINE(HAVE_LIB_AREA_USE_SQLITE, 1, [ Have lib_area SQLITE support ])
   fi

    if test "$PHP_LIB_AREA_USE_MYSQL" != "no"; then
        AC_DEFINE(HAVE_LIB_AREA_USE_MYSQL, 1, [ Have lib_area MYSQL support ])
    fi

   AC_DEFINE(HAVE_LIB_AREA, 1, [ Have lib_area support ])
   PHP_NEW_EXTENSION(lib_area, lib_area.c lib_area_class.c lib_area_exception_class.c, $ext_shared)
fi
