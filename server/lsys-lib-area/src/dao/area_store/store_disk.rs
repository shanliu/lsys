use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::{
    AreaCode, AreaCodeIndexData, AreaCodeIndexInfo, AreaCodeIndexTree, AreaCodeProvider,
    AreaCodeTantivy, AreaError, AreaGeo, AreaGeoIndexInfo, AreaGeoProvider, AreaResult,
    AreaStoreProvider,
};
use geo::{LineString, Point};
use memmap2::{Mmap, MmapMut};
use std::fs::{remove_dir_all, OpenOptions};
use std::io::{Seek, SeekFrom};
use tantivy::directory::MmapDirectory;
use tantivy::{schema::Schema, store::Compressor, Index, IndexBuilder, IndexSettings, IndexWriter};

fn mmap_find_version(mmap: &Mmap, ver_start_index: usize) -> AreaResult<String> {
    let ver_len = std::mem::size_of::<usize>();
    let start_index = ver_start_index - ver_len;
    let tmp = unsafe {
        let version_len = mmap[start_index..].as_ptr() as *const usize;
        let ver_start = ver_start_index;
        let ver_end = ver_start_index + version_len.read();
        String::from_utf8_lossy(&mmap[ver_start..ver_end]).to_string()
    };
    Ok(tmp)
}
fn mmap_check_index(max_len: usize, index: usize) -> AreaResult<()> {
    if max_len == 0 {
        return Err(AreaError::Store("data is empty".to_string()));
    }
    if index > max_len - 1 {
        return Err(AreaError::Store(format!(
            "max is :{},your submit index is:{}",
            max_len, index
        )));
    }
    Ok(())
}

pub struct AreaCodeIndexDataDisk {
    path: PathBuf,
    mmap: Option<Mmap>,
    hash: HashMap<String, AreaCodeIndexInfo>,
    index: Vec<u64>,
}

impl AreaCodeIndexDataDisk {
    pub fn new(path: PathBuf) -> Self {
        Self {
            mmap: None,
            path,
            hash: HashMap::new(),
            index: vec![],
        }
    }
}

fn index_search(index: &[u64], find: &u64) -> (Option<usize>, usize, usize) {
    let find_index = index.binary_search(find).ok();
    let mut start_index = 0;
    let mut end_index = find_index.unwrap_or_default();
    if let Some(tmp_find_index) = &find_index {
        for i in (0..=*tmp_find_index).rev() {
            if index[i] < *find {
                start_index = i;
                break;
            }
        }
        for (i, tmp) in index.iter().enumerate().skip(*tmp_find_index) {
            if *tmp > *find {
                end_index = i;
                break;
            }
        }
    }
    (find_index, start_index, end_index)
}

type AreaCodeInfo = (usize, usize, usize, usize);
type AreaCodeItemPrefix = (usize, bool, usize);
fn mmap_find_code_index_info(mmap: &Mmap, index: usize) -> AreaResult<(String, AreaCodeIndexInfo)> {
    let prefix = std::mem::size_of::<AreaCodeItemPrefix>();
    let (max_len, max_key_length, max_value_length, version_len) = unsafe {
        let ptr = mmap[0..].as_ptr() as *const AreaCodeInfo;
        ptr.read()
    };
    mmap_check_index(max_len, index)?;
    let info_len =
        std::mem::size_of::<AreaCodeInfo>() + version_len + std::mem::size_of::<u64>() * max_len;
    let item_len = prefix + max_key_length + max_value_length;
    let (tmp, key_tmp, val_tmp) = unsafe {
        let ptr = mmap[info_len + index * item_len..].as_ptr() as *const AreaCodeItemPrefix;
        let tmp: AreaCodeItemPrefix = ptr.read();
        // println!("{:?}:{}", tmp, index);
        // 读取20字节长度数据
        let key_start = index * item_len + info_len + prefix;
        let key_end = tmp.0;
        let val_start = index * item_len + info_len + prefix + max_key_length;
        let val_end = tmp.2;
        let key_tmp = String::from_utf8_lossy(&mmap[key_start..key_start + key_end]).to_string();
        let val_tmp = String::from_utf8_lossy(&mmap[val_start..val_start + val_end]).to_string();
        (tmp, key_tmp, val_tmp)
    };
    Ok((
        key_tmp,
        AreaCodeIndexInfo {
            hide: tmp.1,
            name: val_tmp,
        },
    ))
}

impl AreaCodeIndexData for AreaCodeIndexDataDisk {
    fn set(&mut self, key: &str, val: AreaCodeIndexInfo) -> AreaResult<()> {
        self.hash.insert(key.to_string(), val);
        Ok(())
    }
    fn clear(&mut self) -> AreaResult<()> {
        self.hash = HashMap::new();
        self.mmap = None;
        self.index = vec![];
        Ok(())
    }
    fn get(&self, index: &str) -> Option<AreaCodeIndexInfo> {
        if let Some(mmap) = &self.mmap {
            let max_len = unsafe {
                let ptr = mmap[0..].as_ptr() as *const usize;
                ptr.read()
            };
            if max_len == 0 {
                return None;
            }
            let find_start = index.parse::<u64>().unwrap_or(0);
            let (find_index_tmp, start_index, end_index) = index_search(&self.index, &find_start);
            let find_index = find_index_tmp?;
            if find_index > 0 {
                let mut pref_index = find_index;
                loop {
                    let (key, val) = mmap_find_code_index_info(mmap, pref_index).ok()?;
                    if key.as_str() == index {
                        return Some(val);
                    }
                    if pref_index <= start_index {
                        break;
                    }
                    pref_index -= 1;
                }
            }
            for i in find_index..=end_index {
                let (key, val) = mmap_find_code_index_info(mmap, i).ok()?;
                if key.as_str() == index {
                    return Some(val);
                }
            }
        }
        None
    }
    fn init(&mut self) -> AreaResult<()> {
        let mut file = match OpenOptions::new().read(true).open(&self.path) {
            Ok(file) => file,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => return Ok(()),
                _ => {
                    return Err(AreaError::Store(err.to_string()));
                }
            },
        };
        match file.metadata() {
            Ok(meta) => {
                if !meta.is_file() || meta.len() == 0 {
                    return Ok(());
                }
            }
            Err(err) => {
                return Err(AreaError::Store(err.to_string()));
            }
        }
        file.seek(SeekFrom::Start(0))?;
        let mmap = unsafe { Mmap::map(&file)? };
        let info_len = std::mem::size_of::<AreaCodeInfo>();
        let (max_len, _, _, version_len) = unsafe {
            let ptr = mmap[0..].as_ptr() as *const AreaCodeInfo;
            ptr.read()
        };
        if max_len == 0 {
            return Ok(());
        }
        let mut index_data = Vec::with_capacity(max_len);
        let tlen = std::mem::size_of::<u64>();
        for i in 0..max_len {
            unsafe {
                let ipd_start = info_len + version_len + tlen * i;
                let ipd_end = ipd_start + tlen;
                let ptr = (&mmap)[ipd_start..ipd_end].as_ptr() as *const u64;
                index_data.push(ptr.read());
            }
        }
        self.index = index_data;
        self.mmap = Some(mmap);
        self.hash = HashMap::new();
        Ok(())
    }
    fn save(&mut self, version: &str) -> AreaResult<()> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)?;
        let mut max_key_length = 0;
        let mut max_value_length = 0;
        let max_len = self.hash.len();
        let mut vec_data = Vec::with_capacity(max_len);
        for (key, value) in self.hash.iter() {
            if key.len() > max_key_length {
                max_key_length = key.len();
            }
            if value.name.len() > max_value_length {
                max_value_length = value.name.len();
            }
            let index = key.parse::<u64>().unwrap_or(0);
            vec_data.push((index, key, value));
        }
        vec_data.sort_by(|a, b| a.0.cmp(&b.0));
        let index_data = vec_data.iter().map(|e| e.0).collect::<Vec<u64>>();
        //元素数量，最大key长度，最大value长度，版本信息长度 + 版本内容长度
        let info_len = std::mem::size_of::<AreaCodeInfo>()
            + version.len()
            + std::mem::size_of::<u64>() * max_len;
        let prefix = std::mem::size_of::<AreaCodeItemPrefix>();
        let item_len = prefix + max_key_length + max_value_length;
        file.set_len((info_len + item_len * max_len) as u64)?;
        file.seek(SeekFrom::Start(0))?;
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        let ptr = mmap.as_mut_ptr();
        let tmp = &(max_len, max_key_length, max_value_length, version.len()) as *const AreaCodeInfo
            as *const u8;
        unsafe {
            std::ptr::copy_nonoverlapping(tmp, ptr, std::mem::size_of::<AreaCodeInfo>());
            std::ptr::copy_nonoverlapping(
                version.as_bytes().as_ptr(),
                ptr.add(std::mem::size_of::<AreaCodeInfo>()),
                version.len(),
            );
            let tlen = std::mem::size_of::<u64>();
            for (i, tmp) in index_data.iter().enumerate() {
                let tmpadd = tmp as *const u64 as *const u8;
                std::ptr::copy_nonoverlapping(
                    tmpadd,
                    ptr.add(std::mem::size_of::<AreaCodeInfo>() + version.len() + i * tlen),
                    tlen,
                );
            }
        }

        for (i, (_, key, value)) in vec_data.into_iter().enumerate() {
            let tmp = &(key.len(), value.hide, value.name.len()) as *const AreaCodeItemPrefix
                as *const u8;
            let ptr = mmap.as_mut_ptr();
            unsafe {
                std::ptr::copy_nonoverlapping(tmp, ptr.add(i * item_len + info_len), prefix);
                let key_start = i * item_len + info_len + prefix;
                std::ptr::copy_nonoverlapping(
                    key.as_bytes().as_ptr(),
                    ptr.add(key_start),
                    key.len(),
                );
                let val_start = i * item_len + info_len + prefix + max_key_length;
                std::ptr::copy_nonoverlapping(
                    value.name.as_bytes().as_ptr(),
                    ptr.add(val_start),
                    value.name.len(),
                );
            }
        }
        mmap.flush()?;
        file.seek(SeekFrom::Start(0))?;
        let mmap = unsafe { Mmap::map(&file)? };
        self.hash = HashMap::new();
        self.mmap = Some(mmap);
        self.index = index_data;
        Ok(())
    }
    fn version(&self) -> String {
        self.mmap
            .as_ref()
            .map(|e| mmap_find_version(e, std::mem::size_of::<AreaCodeInfo>()).unwrap_or_default())
            .unwrap_or_default()
    }
}

pub struct AreaCodeIndexTreeDisk {
    mmap: Option<Mmap>,
    path: PathBuf,
    data: HashMap<String, Vec<String>>,
    index: Vec<u64>,
}
impl AreaCodeIndexTreeDisk {
    pub fn new(path: PathBuf) -> Self {
        Self {
            mmap: None,
            path,
            data: HashMap::new(),
            index: vec![],
        }
    }
}
type AreaCodeTreeInfo = (usize, usize, usize, usize, usize);
type AreaCodeTreeItemPrefix = (usize, usize, usize);
fn mmap_find_code_tree(mmap: &Mmap, index: usize) -> AreaResult<(String, Vec<String>)> {
    let prefix = std::mem::size_of::<AreaCodeTreeItemPrefix>();
    let (max_len, max_key_length, max_tree_length, max_tree_count, version_len) = unsafe {
        let ptr = mmap[0..].as_ptr() as *const AreaCodeTreeInfo;
        ptr.read()
    };
    mmap_check_index(max_len, index)?;
    let info_len = std::mem::size_of::<AreaCodeTreeInfo>()
        + version_len
        + std::mem::size_of::<u64>() * max_len;
    let item_len = prefix + max_key_length + max_tree_length * max_tree_count;
    let (key_tmp, val_tmp) = unsafe {
        let ptr = mmap[info_len + index * item_len..].as_ptr() as *const AreaCodeTreeItemPrefix;
        // //index,key_length,sub-code-count,sub-code-len,
        let tmp: AreaCodeTreeItemPrefix = ptr.read();
        // 读取20字节长度数据
        let key_start = index * item_len + info_len + prefix;
        let key_end = tmp.0;
        let key_tmp = String::from_utf8_lossy(&mmap[key_start..key_start + key_end]).to_string();
        let mut val_tmp = Vec::with_capacity(tmp.1);
        if tmp.1 > 0 {
            for val_i in 0..tmp.1 {
                let val_start =
                    index * item_len + info_len + prefix + max_key_length + val_i * max_tree_length;
                let val_end = tmp.2;
                let sub_val =
                    String::from_utf8_lossy(&mmap[val_start..val_start + val_end]).to_string();
                val_tmp.push(sub_val);
            }
        }
        (key_tmp, val_tmp)
    };
    Ok((key_tmp, val_tmp))
}

fn mmap_code_tree_childs(index_data: &[u64], mmap: &Mmap, index: &str) -> Option<Vec<String>> {
    let max_len = unsafe {
        let ptr = mmap[0..].as_ptr() as *const usize;
        ptr.read()
    };

    if max_len == 0 {
        return None;
    }
    let find_start = index.parse::<u64>().unwrap_or(0);
    let (find_index_tmp, start_index, end_index) = index_search(index_data, &find_start);
    let find_index = find_index_tmp?;
    if find_index > 0 {
        let mut prev_index = find_index;
        loop {
            let (key, val) = mmap_find_code_tree(mmap, prev_index).ok()?;
            if key.as_str() == index {
                return Some(val);
            }
            if prev_index <= start_index {
                break;
            }
            prev_index -= 1;
        }
    }
    for i in find_index..=end_index {
        let (key, val) = mmap_find_code_tree(mmap, i).ok()?;
        if key.as_str() == index {
            return Some(val);
        }
    }
    None
}

impl AreaCodeIndexTree for AreaCodeIndexTreeDisk {
    fn clear(&mut self) -> AreaResult<()> {
        self.data = HashMap::new();
        self.mmap = None;
        self.index = vec![];
        Ok(())
    }
    fn init(&mut self) -> AreaResult<()> {
        let mut file = match OpenOptions::new().read(true).open(&self.path) {
            Ok(file) => file,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => return Ok(()),
                _ => {
                    return Err(AreaError::Store(err.to_string()));
                }
            },
        };
        match file.metadata() {
            Ok(meta) => {
                if !meta.is_file() || meta.len() == 0 {
                    return Ok(());
                }
            }
            Err(err) => {
                return Err(AreaError::Store(err.to_string()));
            }
        }
        file.seek(SeekFrom::Start(0))?;
        let mmap = unsafe { Mmap::map(&file)? };
        let info_len = std::mem::size_of::<AreaCodeTreeInfo>();
        let (max_len, _, _, _, version_len) = unsafe {
            let ptr = mmap[0..].as_ptr() as *const AreaCodeTreeInfo;
            ptr.read()
        };
        if max_len == 0 {
            return Ok(());
        }
        let mut index_data = Vec::with_capacity(max_len);
        let tlen = std::mem::size_of::<u64>();
        for i in 0..max_len {
            unsafe {
                let ipd_start = info_len + version_len + tlen * i;
                let ipd_end = ipd_start + tlen;
                let ptr = (&mmap)[ipd_start..ipd_end].as_ptr() as *const u64;
                index_data.push(ptr.read());
            }
        }
        self.index = index_data;
        self.mmap = Some(mmap);
        self.data = HashMap::new();
        Ok(())
    }
    fn add(&mut self, code_data: Vec<&str>) -> AreaResult<()> {
        let mut perv = self.data.entry("".to_string());
        for ddd in code_data {
            let code = ddd.to_string();
            match perv {
                Entry::Occupied(mut tmp) => {
                    if !tmp.get().contains(&code) {
                        tmp.get_mut().push(code.clone());
                    }
                }
                Entry::Vacant(tmp) => {
                    tmp.insert(vec![code.clone()]);
                }
            };
            perv = self.data.entry(code);
        }
        Ok(())
    }
    fn childs(&self, code_data: &[&str]) -> Option<Vec<(String, bool)>> {
        let index = match code_data.last() {
            Some(t) => t,
            None => "",
        };
        if let Some(mmap) = &self.mmap {
            let tmp = mmap_code_tree_childs(&self.index, mmap, index)?;
            return Some(
                tmp.into_iter()
                    .map(|t| {
                        let next = mmap_code_tree_childs(&self.index, mmap, &t)
                            .map(|tt| !tt.is_empty())
                            .unwrap_or(false);
                        (t, next)
                    })
                    .collect(),
            );
        }
        None
    }
    fn save(&mut self, version: &str) -> AreaResult<()> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)?;
        let mut max_key_length = 0;
        let mut max_tree_length = 0;
        let mut max_tree_count = 0;
        let max_len = self.data.len();
        let mut vec_data = Vec::with_capacity(max_len);

        for (key, value) in self.data.iter() {
            if key.len() > max_key_length {
                max_key_length = key.len();
            }
            if value.len() > max_tree_count {
                max_tree_count = value.len();
            }
            let max_str_len = value.iter().map(|e| e.len()).max().unwrap_or(0);
            if max_str_len > max_tree_length {
                max_tree_length = max_str_len;
            }
            let index = key.parse::<u64>().unwrap_or(0);
            vec_data.push((index, key, value, max_str_len));
        }
        vec_data.sort_by(|a, b| a.0.cmp(&b.0));
        let index_data = vec_data.iter().map(|e| e.0).collect::<Vec<u64>>();
        //index,key_length,sub-code-count,sub-code-len,
        let prefix = std::mem::size_of::<AreaCodeTreeItemPrefix>();
        let item_len = prefix + max_key_length + max_tree_length * max_tree_count;

        //max_len,max_key_length,max_tree_length,max_tree_count,version_len
        let info_len = std::mem::size_of::<AreaCodeTreeInfo>()
            + version.len()
            + std::mem::size_of::<u64>() * max_len;

        file.set_len((info_len + item_len * max_len) as u64)?;
        file.seek(SeekFrom::Start(0))?;
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        let ptr = mmap.as_mut_ptr();
        let tmp = &(
            max_len,
            max_key_length,
            max_tree_length,
            max_tree_count,
            version.len(),
        ) as *const AreaCodeTreeInfo as *const u8;
        unsafe {
            std::ptr::copy_nonoverlapping(tmp, ptr, std::mem::size_of::<AreaCodeTreeInfo>());
            std::ptr::copy_nonoverlapping(
                version.as_bytes().as_ptr(),
                ptr.add(std::mem::size_of::<AreaCodeTreeInfo>()),
                version.len(),
            );
            let tlen = std::mem::size_of::<u64>();
            for (i, tmpaa) in index_data.iter().enumerate() {
                let add_offset = std::mem::size_of::<AreaCodeTreeInfo>() + version.len() + i * tlen;
                let add_tmp = tmpaa as *const u64 as *const u8;
                std::ptr::copy_nonoverlapping(add_tmp, ptr.add(add_offset), tlen);
            }
        }

        for (i, (_, key, value, sub_value_max_length)) in vec_data.into_iter().enumerate() {
            //key_length,sub-code-count,sub-code-len,
            let tmp = &(key.len(), value.len(), sub_value_max_length)
                as *const AreaCodeTreeItemPrefix as *const u8;
            let ptr = mmap.as_mut_ptr();
            unsafe {
                std::ptr::copy_nonoverlapping(tmp, ptr.add(i * item_len + info_len), prefix);
                let key_start = i * item_len + info_len + prefix;
                std::ptr::copy_nonoverlapping(
                    key.as_bytes().as_ptr(),
                    ptr.add(key_start),
                    key.len(),
                );
                for (i_val, tmp_val) in value.iter().enumerate() {
                    // if tmp_val.starts_with("441403") {
                    //     println!("{}", tmp_val);
                    // }
                    //理论上code等长
                    let val_start =
                        i * item_len + info_len + prefix + max_key_length + i_val * max_tree_length;
                    std::ptr::copy_nonoverlapping(
                        tmp_val.as_bytes().as_ptr(),
                        ptr.add(val_start),
                        tmp_val.len(),
                    );
                }
            }
        }
        mmap.flush()?;
        file.seek(SeekFrom::Start(0))?;
        let mmap = unsafe { Mmap::map(&file)? };

        self.data = HashMap::new();
        self.mmap = Some(mmap);
        self.index = index_data;

        Ok(())
    }
    fn version(&self) -> String {
        self.mmap
            .as_ref()
            .map(|e| {
                mmap_find_version(e, std::mem::size_of::<AreaCodeTreeInfo>()).unwrap_or_default()
            })
            .unwrap_or_default()
    }
}

pub struct AreaCodeTantivyDisk {
    path: PathBuf,
    index_size: usize,
}

impl AreaCodeTantivyDisk {
    pub fn new(path: PathBuf, index_size: usize) -> Self {
        Self { path, index_size }
    }
}

impl AreaCodeTantivy for AreaCodeTantivyDisk {
    fn create_index(&self, schema: Schema) -> AreaResult<Index> {
        Ok(IndexBuilder::new()
            .schema(schema)
            .settings(IndexSettings {
                docstore_compression: Compressor::None,
                ..IndexSettings::default()
            })
            .open_or_create(MmapDirectory::open(&self.path)?)?)
    }
    fn index_writer(&self, index: &Index) -> AreaResult<IndexWriter> {
        Ok(index.writer(self.index_size)?)
    }
}
#[derive(Default)]
pub struct DiskAreaCodeProvider {}

impl AreaCodeProvider for DiskAreaCodeProvider {
    type CD = AreaCodeIndexDataDisk;
    type CT = AreaCodeIndexTreeDisk;
    type TT = AreaCodeTantivyDisk;
}

pub struct DiskAreaGeoProvider {
    max_distance: u64,
    polygon_data: Vec<(String, Point, LineString, Vec<LineString>)>,
    mmap: Option<Mmap>,
    path: PathBuf,
}
impl DiskAreaGeoProvider {
    pub fn new(path: PathBuf) -> Self {
        Self {
            max_distance: 0,
            mmap: None,
            path,
            polygon_data: vec![],
        }
    }
}

type DiskAreaGeoInfo = (usize, usize, usize, usize, u64, usize);
type DiskAreaGeoCenterPrefix = (usize, f64, f64);

impl AreaGeoProvider for DiskAreaGeoProvider {
    fn clear(&mut self) -> AreaResult<()> {
        self.polygon_data = vec![];
        Ok(())
    }
    fn push_data(
        &mut self,
        code: &str,
        center_geo: Point,
        exterior: LineString,
        interiors: Vec<LineString>,
    ) -> AreaResult<()> {
        self.polygon_data
            .push((code.to_string(), center_geo, exterior, interiors));
        Ok(())
    }
    fn get_center_data(&self) -> AreaResult<Vec<(usize, String, Point)>> {
        let mmap = match self.mmap.as_ref() {
            Some(t) => t,
            None => return Ok(vec![]),
        };
        let info_len = std::mem::size_of::<DiskAreaGeoInfo>();

        let (
            max_len,         //元素总量
            max_code_length, //code 最大长度
            _,               //最大线数量
            _,               //坐标总数量
            _,
            version_len, //版本字符串长度
        ) = unsafe {
            let ptr = mmap[0..].as_ptr() as *const DiskAreaGeoInfo;
            ptr.read()
        };
        let prefix = std::mem::size_of::<DiskAreaGeoCenterPrefix>();

        if max_len == 0 {
            return Ok(vec![]);
        }

        let item_len = prefix + max_code_length;
        let out = unsafe {
            let mut out = Vec::with_capacity(max_len);
            for index in 0..max_len {
                let ptr = mmap[info_len + version_len + index * item_len..].as_ptr()
                    as *const DiskAreaGeoCenterPrefix;
                // //key_length,sub-code-count,sub-code-len,
                let tmp: DiskAreaGeoCenterPrefix = ptr.read();
                // 读取20字节长度数据
                let key_start = index * item_len + info_len + version_len + prefix;
                let key_end = tmp.0;
                let key_tmp =
                    String::from_utf8_lossy(&mmap[key_start..key_start + key_end]).to_string();
                let point = Point::new(tmp.1, tmp.2);
                out.push((index, key_tmp, point))
            }
            out
        };
        Ok(out)
    }
    fn get_polygon_data(&self, index: &usize) -> Option<AreaGeoIndexInfo> {
        let mmap = self.mmap.as_ref()?;
        let info_len = std::mem::size_of::<DiskAreaGeoInfo>();

        let (
            max_len,          //元素总量
            max_code_length,  //code 最大长度
            max_line_len,     //最大线数量
            max_polygon_size, //坐标总数量
            _,
            version_len, //版本字符串长度
        ) = unsafe {
            let ptr = mmap[0..].as_ptr() as *const DiskAreaGeoInfo;
            ptr.read()
        };

        if max_len == 0 || *index >= max_len {
            return None;
        }

        let center_prefix = std::mem::size_of::<DiskAreaGeoCenterPrefix>();
        let mmap_center_size = (center_prefix + max_code_length) * max_len;
        let info_all_len = info_len + version_len + mmap_center_size;

        let polygon_prefix = std::mem::size_of::<usize>() * (max_line_len + 1);
        let polygon_geo_size = std::mem::size_of::<(f64, f64)>();
        let polygon_item_size = polygon_prefix + max_polygon_size * polygon_geo_size;

        let (exterior, interiors) = unsafe {
            let item_start = info_all_len + index * polygon_item_size;

            let ptr = mmap[item_start..].as_ptr() as *const (usize, usize);
            // //index,外框元素长度,内框数量
            let (wlen_tmp, ilen_tmp): (usize, usize) = ptr.read();
            let polygon_tmp_prefix = std::mem::size_of::<(usize, usize)>();
            let mut wout = Vec::with_capacity(wlen_tmp);
            if wlen_tmp > 0 {
                for sub_i in 0..wlen_tmp {
                    let tmp_ptr = mmap[item_start + polygon_prefix + polygon_geo_size * sub_i..]
                        .as_ptr() as *const (f64, f64);
                    let tmp_data = tmp_ptr.read();
                    wout.push(Point::new(tmp_data.0, tmp_data.1))
                }
            }
            let wline_str = LineString::from(wout);

            let mut iline_start = item_start + polygon_prefix + polygon_geo_size * wlen_tmp;
            let mut iline_str_vec = Vec::with_capacity(ilen_tmp);
            if ilen_tmp > 0 {
                for sub_i in 0..ilen_tmp {
                    let ptr = mmap
                        [item_start + polygon_tmp_prefix + std::mem::size_of::<usize>() * sub_i..]
                        .as_ptr() as *const usize;
                    let ilen = ptr.read();
                    let mut iline_str = Vec::with_capacity(ilen);
                    for sub_ii in 0..ilen {
                        let tmp_ptr = mmap[iline_start + sub_ii * polygon_geo_size..].as_ptr()
                            as *const (f64, f64);
                        let tmp_data = tmp_ptr.read();
                        iline_str.push(Point::new(tmp_data.0, tmp_data.1))
                    }
                    iline_str_vec.push(LineString::from(iline_str));
                    iline_start += ilen * polygon_geo_size;
                }
            }
            (wline_str, iline_str_vec)
        };
        Some(AreaGeoIndexInfo::new(exterior, interiors))
    }
    fn get_max_distance(&self) -> AreaResult<u64> {
        Ok(self.max_distance)
    }
    fn save(&mut self, max_distance: u64, version: &str) -> AreaResult<()> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)?;

        let mut max_code_length = 0;
        let mut max_polygon_size = 0;
        let max_len = self.polygon_data.len();
        let mut polygon_data = Vec::with_capacity(max_len);
        let mut max_line_len = 0;
        for (code, center_point, exc_point, inn_point) in self.polygon_data.iter() {
            if code.len() > max_code_length {
                max_code_length = code.len();
            }
            // 总坐标数
            let point_max_len =
                exc_point.0.len() + inn_point.iter().map(|t| t.0.len()).sum::<usize>();
            if max_polygon_size < point_max_len {
                max_polygon_size = point_max_len;
            }
            let line_len = 1 + inn_point.len(); //总line数量
            if max_line_len < line_len {
                max_line_len = line_len;
            }
            let mut data_vec = Vec::with_capacity(point_max_len);
            for tmp1 in &exc_point.0 {
                data_vec.push(tmp1);
            }
            for tmp2 in inn_point {
                for tmp3 in &tmp2.0 {
                    data_vec.push(tmp3);
                }
            }
            polygon_data.push((
                code,                                                        //编码字符串
                center_point,                                                //中间坐标点
                exc_point.0.len(),                                           //外部line的总元素数量
                inn_point.len(),                                             //内部line的总数量
                inn_point.iter().map(|t| t.0.len()).collect::<Vec<usize>>(), //内部line的分别总元素数量
                data_vec,
            ));
        }
        //geo数据头部信息：
        // 元素总量
        // code 最大长度
        // 最大线数量
        // 坐标总数量
        // 版本字符串长度
        let info_len = std::mem::size_of::<DiskAreaGeoInfo>() + version.len();

        //编码字符长度,中心点坐标[lat,lng],
        let center_prefix = std::mem::size_of::<DiskAreaGeoCenterPrefix>();
        //每个中心点坐标大小：前缀+最大编码字符长度
        let center_item_len = center_prefix + max_code_length;
        //所有中心点坐标的存储空间大小
        let mmap_center_size = center_item_len * max_len;

        //外框坐标数 内框线最大条数 每个线条的坐标数 1+1+n
        let polygon_prefix = std::mem::size_of::<usize>() * (max_line_len + 1);
        //坐标数据大小
        let polygon_geo_size = std::mem::size_of::<(f64, f64)>();
        //每个区域坐标总大小
        let polygon_item_size = polygon_prefix + max_polygon_size * polygon_geo_size;
        //所有区域存储需要大小
        let mmap_polygon_size = polygon_item_size * max_len;

        //设置映射总大小
        file.set_len((info_len + mmap_center_size + mmap_polygon_size) as u64)?;
        file.seek(SeekFrom::Start(0))?;

        //把数据写入映射
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        let ptr = mmap.as_mut_ptr();
        let tmp = &(
            max_len,          //元素总量
            max_code_length,  //code 最大长度
            max_line_len,     //最大线数量
            max_polygon_size, //坐标总数量
            max_distance,
            version.len(), //版本字符串长度
        ) as *const DiskAreaGeoInfo as *const u8;
        unsafe {
            std::ptr::copy_nonoverlapping(tmp, ptr, std::mem::size_of::<DiskAreaGeoInfo>());
            std::ptr::copy_nonoverlapping(
                version.as_bytes().as_ptr(),
                ptr.add(std::mem::size_of::<DiskAreaGeoInfo>()),
                version.len(),
            );
        }
        //write data to mmap
        // code,          //编码字符串
        // center_point,      //中间坐标点
        // exc_len,   //外部line的总元素数量
        // iner_len,  //内部line的总数量
        // iner_data, //内部line的分别总元素数量
        // data_vec,//总坐标数据
        for (i, (code, center_point, exc_len, iner_len, iner_data, data_vec)) in
            polygon_data.into_iter().enumerate()
        {
            //保存中间坐标点到mmap
            //key_len,lat,lng,
            unsafe {
                let center_start = i * center_item_len + info_len;
                let tmp = &(code.len(), center_point.0.x, center_point.0.y)
                    as *const DiskAreaGeoCenterPrefix as *const u8;
                std::ptr::copy_nonoverlapping(tmp, ptr.add(center_start), center_prefix);
                let key_start = center_start + center_prefix;
                std::ptr::copy_nonoverlapping(
                    code.as_bytes().as_ptr(),
                    ptr.add(key_start),
                    code.len(),
                );
            }
            //保存元素坐标点到mmap
            unsafe {
                let py_start = i * polygon_item_size + mmap_center_size + info_len;
                let tmp = &(exc_len, iner_len) as *const (usize, usize) as *const u8;
                let py_prefix_size = std::mem::size_of::<(usize, usize)>();
                std::ptr::copy_nonoverlapping(tmp, ptr.add(py_start), py_prefix_size);

                let line_size = std::mem::size_of::<usize>();
                for (iit, iitmp) in iner_data.iter().enumerate() {
                    let tmp = iitmp as *const usize as *const u8;
                    std::ptr::copy_nonoverlapping(
                        tmp,
                        ptr.add(py_start + py_prefix_size + iit * line_size),
                        line_size,
                    );
                }
                let point_size = std::mem::size_of::<(f64, f64)>();
                for (iit, iitmp) in data_vec.iter().enumerate() {
                    let tmp1 = &(iitmp.x, iitmp.y) as *const (f64, f64) as *const u8;
                    std::ptr::copy_nonoverlapping(
                        tmp1,
                        ptr.add(py_start + polygon_prefix + iit * point_size),
                        point_size,
                    );
                }
            }
        }
        mmap.flush()?;
        file.seek(SeekFrom::Start(0))?;
        let mmap = unsafe { Mmap::map(&file)? };
        self.polygon_data = vec![];
        self.mmap = Some(mmap);
        self.max_distance = max_distance;

        Ok(())
    }
    fn version(&self) -> String {
        self.mmap
            .as_ref()
            .map(|e| {
                mmap_find_version(e, std::mem::size_of::<DiskAreaGeoInfo>()).unwrap_or_default()
            })
            .unwrap_or_default()
    }
    fn init(&mut self) -> AreaResult<()> {
        let mut file = match OpenOptions::new().read(true).open(&self.path) {
            Ok(file) => file,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => return Ok(()),
                _ => {
                    return Err(AreaError::Store(err.to_string()));
                }
            },
        };
        match file.metadata() {
            Ok(meta) => {
                if !meta.is_file() || meta.len() == 0 {
                    return Ok(());
                }
            }
            Err(err) => {
                return Err(AreaError::Store(err.to_string()));
            }
        }
        file.seek(SeekFrom::Start(0))?;
        let mmap = unsafe { Mmap::map(&file)? };

        let (
            max_len, //元素总量
            _,       //code 最大长度
            _,       //最大线数量
            _,       //坐标总数量
            max_distance,
            _,
        ) = unsafe {
            let ptr = mmap[0..].as_ptr() as *const DiskAreaGeoInfo;
            ptr.read()
        };
        if max_len == 0 {
            return Ok(());
        }
        self.max_distance = max_distance;
        self.mmap = Some(mmap);
        self.polygon_data = vec![];
        Ok(())
    }
}

//把数据用mmap存储到磁盘
//可以节省运行时内存及重新启动时，用以前构建的索引，启动速度比较快
//响应速比纯内存方式要慢2倍以上
pub struct AreaStoreDisk {
    dir: PathBuf,
    index_size: usize,
}
impl AreaStoreDisk {
    pub fn new(dir: PathBuf, index_size: Option<usize>) -> AreaResult<Self> {
        if let Err(err) = std::fs::metadata(&dir) {
            match err.kind() {
                std::io::ErrorKind::NotFound => std::fs::create_dir(&dir)?,
                err => {
                    return Err(AreaError::Store(err.to_string()));
                }
            }
        }
        Ok(Self {
            dir,
            index_size: index_size.unwrap_or(500_000_000),
        })
    }
    /// 清理已生成的索引数据
    pub fn clear(self) -> AreaResult<Self> {
        remove_dir_all(&self.dir)?;
        if std::fs::metadata(&self.dir).is_err() {
            std::fs::create_dir(&self.dir)?;
        }
        Ok(self)
    }
}

impl Default for AreaStoreDisk {
    fn default() -> Self {
        let dir = std::env::temp_dir();
        Self {
            dir,
            index_size: 500_000_000,
        }
    }
}

impl AreaStoreProvider for AreaStoreDisk {
    type C = DiskAreaCodeProvider;
    type G = DiskAreaGeoProvider;
    fn create_code(&self) -> AreaResult<AreaCode<Self::C>> {
        let mut tantivy_dir = self.dir.clone();
        tantivy_dir.push("area_data_tantivy/");
        if std::fs::metadata(&tantivy_dir).is_err() {
            std::fs::create_dir(&tantivy_dir)?;
        }
        let mut info_dir = self.dir.clone();
        info_dir.push("area_data_info.bin");
        let mut tree_dir = self.dir.clone();
        tree_dir.push("area_data_tree.bin");
        AreaCode::new(
            AreaCodeTantivyDisk::new(tantivy_dir, self.index_size),
            AreaCodeIndexDataDisk::new(info_dir),
            AreaCodeIndexTreeDisk::new(tree_dir),
        )
    }
    fn create_geo(&self) -> AreaResult<AreaGeo<Self::G>> {
        let mut geo_dir = self.dir.clone();
        geo_dir.push("area_data_geo.bin");
        AreaGeo::new(DiskAreaGeoProvider::new(geo_dir))
    }
}
