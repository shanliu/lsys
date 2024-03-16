use crate::{AreaCodeItem, AreaCodeRelatedItem, AreaDao, AreaError};
use std::ffi::{c_char, c_float, c_int, c_long, c_uchar, c_uint, CStr, CString};

#[repr(C)]
pub struct CAreaDao {
    dao: *mut AreaDao,
}

macro_rules! set_c_error {
    ($err:expr,$err_str:expr) => {
        let error_message = CString::new($err.to_string()).unwrap_or_default();
        let error_message_ptr = Box::into_raw(Box::new(error_message.into_raw()));
        unsafe { *$err_str = *error_message_ptr };
    };
}

macro_rules! cstr_to_string {
    ($c_str:expr,$err_str:expr) => {
        match unsafe { CStr::from_ptr($c_str) }.to_str() {
            Ok(e) => e.to_owned(),
            Err(err) => {
                set_c_error!(err, $err_str);
                return 1;
            }
        }
    };
}
#[allow(unused_macros)]
macro_rules! unwrap_or_c_error {
    ($warp_res:expr,$err_str:expr) => {
        match $warp_res {
            Ok(e) => e,
            Err(err) => {
                set_c_error!(err, $err_str);
                return 2;
            }
        }
    };
}

macro_rules! call_area_dao {
    ($area_dao:expr,$err_str:expr,$method:ident,[$($args:expr),*]) => {
        match $area_dao.as_ref() {
            Some(dao) => match dao.dao.as_ref() {
                Some(tdao) => match tdao.$method($($args),*) {
                    Ok(ok) => ok,
                    Err(AreaError::NotFind(_))=>{
                        vec![]
                    }
                    Err(err) => {
                        set_c_error!(err, $err_str);
                        return 3;
                    }
                },
                None => {
                    set_c_error!("ptr to dao fail", $err_str);
                    return 3;
                },
            },
            None => {
                set_c_error!("ptr as ref fail", $err_str);
                return 3;
            },
        }
    };
}
/// # Safety
///
/// 用于外部C函数调用进行初始化结构
/// 不要在RUST调用
///
#[cfg(feature = "data-csv")]
#[no_mangle]
pub unsafe extern "C" fn lib_area_init_csv(
    code_path: *const c_char,
    geo_path: *const c_char,
    index_size: c_long,
    gz: *const c_uchar,
    area_dao: *mut *mut CAreaDao,
    error: *mut *mut c_char,
) -> c_int {
    use crate::AreaStoreMemory;
    use std::path::PathBuf;
    *error = std::ptr::null_mut();
    *area_dao = std::ptr::null_mut();
    let code_file = cstr_to_string!(code_path, error);
    let code_config = Some(unwrap_or_c_error!(
        crate::CsvAreaCodeData::from_inner_path(PathBuf::from(code_file), *gz != 0),
        error
    ));
    let code_config = match code_config {
        Some(c) => c,
        None => {
            set_c_error!("csv path can't be empty", error);
            return 1;
        }
    };
    let geo_file = cstr_to_string!(geo_path, error);
    let geo_config = Some(unwrap_or_c_error!(
        crate::CsvAreaGeoData::from_inner_path(PathBuf::from(geo_file), *gz != 0),
        error
    ));

    let index_size = index_size as usize;

    let area_obj = unwrap_or_c_error!(
        AreaDao::from_csv_mem(
            crate::CsvAreaData::new(code_config, geo_config),
            if index_size > 0 {
                AreaStoreMemory::new(index_size)
            } else {
                AreaStoreMemory::default()
            }
        ),
        error
    );
    init_area(area_dao, area_obj)
}

/// # Safety
///
/// 用于外部C函数调用进行初始化结构
/// 不要在RUST调用
///
#[cfg(all(feature = "data-csv", feature = "index-disk"))]
#[no_mangle]
pub unsafe extern "C" fn lib_area_init_csv_on_disk(
    code_path: *const c_char,
    geo_path: *const c_char,
    index_size: c_long,
    index_disk: *const c_char,
    gz: *const c_uchar,
    area_dao: *mut *mut CAreaDao,
    error: *mut *mut c_char,
) -> c_int {
    use crate::AreaStoreDisk;
    use std::path::PathBuf;
    *error = std::ptr::null_mut();
    *area_dao = std::ptr::null_mut();
    let code_file = cstr_to_string!(code_path, error);
    let code_config = Some(unwrap_or_c_error!(
        crate::CsvAreaCodeData::from_inner_path(PathBuf::from(code_file), *gz != 0),
        error
    ));
    let code_config = match code_config {
        Some(c) => c,
        None => {
            set_c_error!("csv path can't be empty", error);
            return 1;
        }
    };
    let geo_file = cstr_to_string!(geo_path, error);
    let geo_config = Some(unwrap_or_c_error!(
        crate::CsvAreaGeoData::from_inner_path(PathBuf::from(geo_file), *gz != 0),
        error
    ));

    let index_size = index_size as usize;
    let index_disk_dir = cstr_to_string!(index_disk, error);

    let store_obj = unwrap_or_c_error!(
        if index_size > 0 {
            AreaStoreDisk::new(PathBuf::from(index_disk_dir.trim()), Some(index_size))
        } else {
            AreaStoreDisk::new(PathBuf::from(index_disk_dir.trim()), None)
        },
        error
    );

    let area_obj = unwrap_or_c_error!(
        AreaDao::from_csv_disk(crate::CsvAreaData::new(code_config, geo_config), store_obj),
        error
    );
    init_area(area_dao, area_obj)
}

/// # Safety
///
/// 用于外部C函数调用进行初始化结构
/// 不要在RUST调用
///
#[cfg(feature = "data-sqlite")]
#[no_mangle]
pub unsafe extern "C" fn lib_area_init_sqlite(
    db_path: *const c_char,
    index_size: c_long,
    area_dao: *mut *mut CAreaDao,
    error: *mut *mut c_char,
) -> c_int {
    use crate::AreaStoreMemory;
    use std::path::PathBuf;
    *error = std::ptr::null_mut();
    *area_dao = std::ptr::null_mut();
    let file_config = cstr_to_string!(db_path, error);
    if file_config.trim().is_empty() {
        set_c_error!("sqlite path can't be empty", error);
        return 1;
    }
    let code_config = crate::SqliteAreaCodeData::from_path(PathBuf::from(&file_config));
    let geo_config = Some(crate::SqliteAreaGeoData::from_path(PathBuf::from(
        &file_config,
    )));

    let index_size = index_size as usize;

    let area_obj = unwrap_or_c_error!(
        AreaDao::from_sqlite_mem(
            crate::SqliteAreaData::new(code_config, geo_config),
            if index_size > 0 {
                AreaStoreMemory::new(index_size)
            } else {
                AreaStoreMemory::default()
            }
        ),
        error
    );
    init_area(area_dao, area_obj)
}

/// # Safety
///
/// 用于外部C函数调用进行初始化结构
/// 不要在RUST调用
///
#[cfg(all(feature = "data-sqlite", feature = "index-disk"))]
#[no_mangle]
pub unsafe extern "C" fn lib_area_init_sqlite_on_disk(
    db_path: *const c_char,
    index_size: c_long,
    index_disk: *const c_char,
    area_dao: *mut *mut CAreaDao,
    error: *mut *mut c_char,
) -> c_int {
    use crate::AreaStoreDisk;
    use std::path::PathBuf;
    *error = std::ptr::null_mut();
    *area_dao = std::ptr::null_mut();
    let file_config = cstr_to_string!(db_path, error);
    if file_config.trim().is_empty() {
        set_c_error!("sqlite path can't be empty", error);
        return 1;
    }
    let code_config = crate::SqliteAreaCodeData::from_path(PathBuf::from(&file_config));
    let geo_config = Some(crate::SqliteAreaGeoData::from_path(PathBuf::from(
        &file_config,
    )));

    let index_size = index_size as usize;
    let index_disk_dir = cstr_to_string!(index_disk, error);

    let store_obj = unwrap_or_c_error!(
        if index_size > 0 {
            AreaStoreDisk::new(PathBuf::from(index_disk_dir.trim()), Some(index_size))
        } else {
            AreaStoreDisk::new(PathBuf::from(index_disk_dir.trim()), None)
        },
        error
    );

    let area_obj = unwrap_or_c_error!(
        AreaDao::from_sqlite_disk(
            crate::SqliteAreaData::new(code_config, geo_config),
            store_obj
        ),
        error
    );
    init_area(area_dao, area_obj)
}

/// # Safety
///
/// 用于外部C函数调用进行初始化结构
/// 不要在RUST调用
///
#[cfg(feature = "data-mysql")]
#[no_mangle]
pub unsafe extern "C" fn lib_area_init_mysql(
    db_uri: *const c_char,
    index_size: c_long,
    area_dao: *mut *mut CAreaDao,
    error: *mut *mut c_char,
) -> c_int {
    use crate::AreaStoreMemory;
    *error = std::ptr::null_mut();
    *area_dao = std::ptr::null_mut();
    let uri_config = cstr_to_string!(db_uri, error);
    if uri_config.trim().is_empty() {
        set_c_error!("mysql uri can't be empty", error);
        return 1;
    }
    let code_config = crate::MysqlAreaCodeData::from_uri(&uri_config, None);
    let geo_config = Some(crate::MysqlAreaGeoData::from_uri(&uri_config, None));

    let index_size = index_size as usize;

    let area_obj = unwrap_or_c_error!(
        AreaDao::from_mysql_mem(
            crate::MysqlAreaData::new(code_config, geo_config),
            if index_size > 0 {
                AreaStoreMemory::new(index_size)
            } else {
                AreaStoreMemory::default()
            }
        ),
        error
    );
    init_area(area_dao, area_obj)
}

/// # Safety
///
/// 用于外部C函数调用进行初始化结构
/// 不要在RUST调用
///
#[cfg(all(feature = "data-mysql", feature = "index-disk"))]
#[no_mangle]
pub unsafe extern "C" fn lib_area_init_mysql_on_disk(
    db_uri: *const c_char,
    index_size: c_long,
    index_disk: *const c_char,
    area_dao: *mut *mut CAreaDao,
    error: *mut *mut c_char,
) -> c_int {
    use std::path::PathBuf;

    use crate::AreaStoreDisk;
    *error = std::ptr::null_mut();
    *area_dao = std::ptr::null_mut();
    let uri_config = cstr_to_string!(db_uri, error);
    if uri_config.trim().is_empty() {
        set_c_error!("mysql uri can't be empty", error);
        return 1;
    }
    let code_config = crate::MysqlAreaCodeData::from_uri(&uri_config, None);
    let geo_config = Some(crate::MysqlAreaGeoData::from_uri(&uri_config, None));

    let index_size = index_size as usize;

    let index_disk_dir = cstr_to_string!(index_disk, error);

    let store_obj = unwrap_or_c_error!(
        if index_size > 0 {
            AreaStoreDisk::new(PathBuf::from(index_disk_dir.trim()), Some(index_size))
        } else {
            AreaStoreDisk::new(PathBuf::from(index_disk_dir.trim()), None)
        },
        error
    );

    let area_obj = unwrap_or_c_error!(
        AreaDao::from_mysql_disk(
            crate::MysqlAreaData::new(code_config, geo_config),
            store_obj
        ),
        error
    );
    init_area(area_dao, area_obj)
}

#[allow(dead_code)]
unsafe fn init_area(area_dao: *mut *mut CAreaDao, area_obj: AreaDao) -> c_int {
    let area_ptr = Box::into_raw(Box::new(area_obj));
    let area_box = Box::into_raw(Box::new(CAreaDao { dao: area_ptr }));
    *area_dao = area_box;
    0
}

/// # Safety
///
/// 用于外部C函数调用进行初始化结构
/// 不要在RUST调用
///
#[no_mangle]
pub unsafe extern "C" fn lib_area_code_reload(
    area_dao: *mut *mut CAreaDao,
    error: *mut *mut c_char,
) -> c_int {
    *error = std::ptr::null_mut();
    if area_dao.is_null() {
        return 0;
    }
    let boxed_vec_wrapper = unsafe { Box::from_raw(*area_dao) };
    let boxed_my_struct = unsafe { Box::from_raw(boxed_vec_wrapper.dao) };
    unwrap_or_c_error!(boxed_my_struct.code_reload(), error);
    let area_ptr = Box::into_raw(boxed_my_struct);
    *area_dao = Box::into_raw(Box::new(CAreaDao { dao: area_ptr }));
    drop(boxed_vec_wrapper);
    0
}

/// # Safety
///
/// 用于外部C函数调用进行初始化结构
/// 不要在RUST调用
///
#[no_mangle]
pub unsafe extern "C" fn lib_area_geo_reload(
    area_dao: *mut *mut CAreaDao,
    error: *mut *mut c_char,
) -> c_int {
    *error = std::ptr::null_mut();
    if area_dao.is_null() {
        return 0;
    }
    let boxed_vec_wrapper = unsafe { Box::from_raw(*area_dao) };
    let boxed_my_struct = unsafe { Box::from_raw(boxed_vec_wrapper.dao) };
    unwrap_or_c_error!(boxed_my_struct.geo_reload(), error);
    let area_ptr = Box::into_raw(boxed_my_struct);
    let area_box = Box::into_raw(Box::new(CAreaDao { dao: area_ptr }));
    drop(boxed_vec_wrapper);
    *area_dao = area_box;
    0
}

/// # Safety
///
/// 释放 AreaDao 内存用
/// 不要在RUST调用
///
#[no_mangle]
pub unsafe extern "C" fn lib_area_release_area_dao(ptr: *mut CAreaDao) {
    let boxed_vec_wrapper = unsafe { Box::from_raw(ptr) };
    let boxed_my_struct = unsafe { Box::from_raw(boxed_vec_wrapper.dao) };
    drop(boxed_my_struct);
    drop(boxed_vec_wrapper);
}

/// # Safety
///
/// 释放 错误消息用
/// 不要在RUST调用
///
#[no_mangle]
pub unsafe extern "C" fn lib_area_release_error_str(ptr: *mut c_char) {
    let boxed_vec_wrapper = unsafe { Box::from_raw(ptr) };
    drop(boxed_vec_wrapper);
}

#[repr(C)]
pub struct CAreaItem {
    pub name: *const c_char,
    pub code: *const c_char,
    pub leaf: c_uchar,
}

#[repr(C)]
pub struct CAreaItemVec {
    pub data: *mut CAreaItem,
    pub len: usize,
    pub capacity: usize,
}

/// # Safety
///
/// 释放 CAreaItemVec 内存
/// 不要在RUST调用
///
#[no_mangle]
pub unsafe extern "C" fn lib_area_release_item_vec(ptr: *mut CAreaItemVec) {
    let boxed_vec_wrapper = unsafe { Box::from_raw(ptr) };
    for i in 0..boxed_vec_wrapper.len {
        let item = &mut *boxed_vec_wrapper.data.add(i);
        let name = CString::from_raw(item.name as *mut c_char);
        let code = CString::from_raw(item.code as *mut c_char);
        drop(name);
        drop(code);
    }
    if boxed_vec_wrapper.len > 0 {
        let item = &mut *boxed_vec_wrapper.data;
        drop(Box::from_raw(item));
    }
    drop(boxed_vec_wrapper);
}

//转换RUST的AreaCodeItem 为C的结构
fn area_item_to_ptr(data: Vec<AreaCodeItem>) -> CAreaItemVec {
    let mut c_data = vec![];
    for tmp in data {
        let name = CString::new(tmp.name).unwrap_or_default();
        let name_ptr = name.as_ptr();
        let code = CString::new(tmp.code).unwrap_or_default();
        let code_ptr = code.as_ptr();
        std::mem::forget(name);
        std::mem::forget(code);
        c_data.push(CAreaItem {
            name: name_ptr,
            code: code_ptr,
            leaf: if tmp.leaf { 1 } else { 0 },
        });
    }
    let data_ptr = c_data.as_mut_ptr();
    let len = c_data.len();
    let capacity = c_data.capacity();
    std::mem::forget(c_data);
    // create a VecWrapper instance
    CAreaItemVec {
        data: data_ptr,
        len,
        capacity,
    }
}

/// # Safety
///
/// 查询指定CODE的子节点
/// 不要在RUST调用
///
#[no_mangle]
pub unsafe extern "C" fn lib_area_code_childs(
    code_str: *const c_char,
    area_dao: *mut CAreaDao,
    out_data: *mut *mut CAreaItemVec,
    error: *mut *mut c_char,
) -> c_int {
    *error = std::ptr::null_mut();
    let rust_string = cstr_to_string!(code_str, error);
    let data = call_area_dao!(area_dao, error, code_childs, [&rust_string]);
    *out_data = Box::into_raw(Box::new(area_item_to_ptr(data)));
    0
}

/// # Safety
///
/// 查询指定CODE的详细
/// 不要在RUST调用
///
#[no_mangle]
pub unsafe extern "C" fn lib_area_code_find(
    code_str: *const c_char,
    area_dao: *mut CAreaDao,
    out_data: *mut *mut CAreaItemVec,
    error: *mut *mut c_char,
) -> c_int {
    *error = std::ptr::null_mut();
    let rust_string = cstr_to_string!(code_str, error);
    let data = call_area_dao!(area_dao, error, code_find, [&rust_string]);
    *out_data = Box::into_raw(Box::new(area_item_to_ptr(data)));
    0
}

#[repr(C)]
pub struct CAreaItemVecs {
    pub data: *mut CAreaItemVec,
    pub len: usize,
    pub capacity: usize,
}
/// # Safety
///
/// 释放 CAreaItemVecs 内存
/// 不要在RUST调用
///
#[no_mangle]
pub unsafe extern "C" fn lib_area_release_item_vecs(ptr: *mut CAreaItemVecs) {
    let boxed_vec_wrapper = unsafe { Box::from_raw(ptr) };
    for i in 0..boxed_vec_wrapper.len {
        let item = &mut *boxed_vec_wrapper.data.add(i);
        for j in 0..item.len {
            let sitem = &mut *item.data.add(j);
            let name = CString::from_raw(sitem.name as *mut c_char);
            let code = CString::from_raw(sitem.code as *mut c_char);
            drop(name);
            drop(code);
        }
        if item.len > 0 {
            let vitem = &mut *item.data;
            drop(Box::from_raw(vitem));
        }
    }
    if boxed_vec_wrapper.len > 0 {
        let item = &mut *boxed_vec_wrapper.data;
        drop(Box::from_raw(item));
    }
    drop(boxed_vec_wrapper);
}

//转换RUST的AreaCodeItem数组为 为C的结构
fn area_item_vec_to_ptr(data: Vec<Vec<AreaCodeItem>>) -> CAreaItemVecs {
    let mut c_data = vec![];
    for tmp in data {
        c_data.push(area_item_to_ptr(tmp));
    }
    let data_ptr = c_data.as_mut_ptr();
    let len = c_data.len();
    let capacity = c_data.capacity();
    std::mem::forget(c_data);
    CAreaItemVecs {
        data: data_ptr,
        len,
        capacity,
    }
}

/// # Safety
///
/// 搜索指定关键字
/// 不要在RUST调用
///
#[no_mangle]
pub unsafe extern "C" fn lib_area_code_search(
    code_str: *const c_char,
    limit: c_uint,
    area_dao: *mut CAreaDao,
    out_data: *mut *mut CAreaItemVecs,
    error: *mut *mut c_char,
) -> c_int {
    *error = std::ptr::null_mut();
    let rust_string = cstr_to_string!(code_str, error);
    let data = call_area_dao!(area_dao, error, code_search, [&rust_string, limit as usize]);
    *out_data = Box::into_raw(Box::new(area_item_vec_to_ptr(
        data.into_iter().map(|e| e.item).collect::<Vec<_>>(),
    )));
    0
}

#[repr(C)]
pub struct CAreaRelatedItem {
    pub name: *const c_char,
    pub code: *const c_char,
    pub selected: c_uchar,
    pub leaf: c_uchar,
}

#[repr(C)]
pub struct CAreaRelatedItemVec {
    pub data: *mut CAreaRelatedItem,
    pub len: usize,
    pub capacity: usize,
}

#[repr(C)]
pub struct CAreaRelatedItemVecs {
    pub data: *mut CAreaRelatedItemVec,
    pub len: usize,
    pub capacity: usize,
}
/// # Safety
///
/// 释放 CAreaRelatedItemVecs 内存
/// 不要在RUST调用
///
#[no_mangle]
pub unsafe extern "C" fn lib_area_release_related_vecs(ptr: *mut CAreaRelatedItemVecs) {
    let boxed_vec_wrapper = unsafe { Box::from_raw(ptr) };
    for i in 0..boxed_vec_wrapper.len {
        let item = &mut *boxed_vec_wrapper.data.add(i);
        for j in 0..item.len {
            let vitem = &mut *item.data.add(j);
            let name = CString::from_raw(vitem.name as *mut c_char);
            let code = CString::from_raw(vitem.code as *mut c_char);
            drop(name);
            drop(code);
        }
        if item.len > 0 {
            let item = &mut *item.data;
            drop(Box::from_raw(item));
        }
    }
    if boxed_vec_wrapper.len > 0 {
        let item = &mut *boxed_vec_wrapper.data;
        drop(Box::from_raw(item));
    }
    drop(boxed_vec_wrapper);
}

//转换RUST的AreaCodeItem 为C的结构
fn area_related_item_to_ptr(data: Vec<AreaCodeRelatedItem>) -> CAreaRelatedItemVec {
    let mut c_data = vec![];
    for tmp in data {
        let name = CString::new(tmp.item.name).unwrap_or_default();
        let name_ptr = name.as_ptr();
        let code = CString::new(tmp.item.code).unwrap_or_default();
        let code_ptr = code.as_ptr();
        std::mem::forget(name);
        std::mem::forget(code);
        c_data.push(CAreaRelatedItem {
            name: name_ptr,
            code: code_ptr,
            selected: if tmp.selected { 1 } else { 0 },
            leaf: if tmp.item.leaf { 1 } else { 0 },
        });
    }
    let data_ptr = c_data.as_mut_ptr();
    let len = c_data.len();
    let capacity = c_data.capacity();
    std::mem::forget(c_data);
    // create a VecWrapper instance
    CAreaRelatedItemVec {
        data: data_ptr,
        len,
        capacity,
    }
}

//转换RUST的AreaCodeItem数组为 为C的结构
fn area_related_item_vec_to_ptr(data: Vec<Vec<AreaCodeRelatedItem>>) -> CAreaRelatedItemVecs {
    let mut c_data = vec![];
    for tmp in data {
        c_data.push(area_related_item_to_ptr(tmp));
    }
    let data_ptr = c_data.as_mut_ptr();
    let len = c_data.len();
    let capacity = c_data.capacity();
    std::mem::forget(c_data);
    CAreaRelatedItemVecs {
        data: data_ptr,
        len,
        capacity,
    }
}

/// # Safety
///
/// 根据地区CODE查询地址数据
/// 不要在RUST调用
///
#[no_mangle]
pub unsafe extern "C" fn lib_area_code_related(
    code_str: *const c_char,
    area_dao: *mut CAreaDao,
    out_data: *mut *mut CAreaRelatedItemVecs,
    error: *mut *mut c_char,
) -> c_int {
    *error = std::ptr::null_mut();
    let rust_string = cstr_to_string!(code_str, error);
    let data = call_area_dao!(area_dao, error, code_related, [&rust_string]);
    *out_data = Box::into_raw(Box::new(area_related_item_vec_to_ptr(data)));
    0
}

/// # Safety
///
/// 根据地区CODE查询地址数据
/// 不要在RUST调用
///
#[no_mangle]
pub unsafe extern "C" fn lib_area_geo_search(
    lat: c_float,
    lng: c_float,
    area_dao: *mut CAreaDao,
    out_data: *mut *mut CAreaItemVec,
    error: *mut *mut c_char,
) -> c_int {
    *error = std::ptr::null_mut();
    let data = call_area_dao!(area_dao, error, geo_search, [lat as f64, lng as f64]);
    *out_data = Box::into_raw(Box::new(area_item_to_ptr(data)));
    0
}
