use std::collections::HashMap;

use geo::{LineString, Point};
use tantivy::{schema::Schema, store::Compressor, Index, IndexBuilder, IndexSettings, IndexWriter};

use crate::{
    AreaCode, AreaCodeIndexData, AreaCodeIndexInfo, AreaCodeIndexTree, AreaCodeProvider,
    AreaCodeTantivy, AreaGeo, AreaGeoIndexInfo, AreaGeoProvider, AreaResult, AreaStoreProvider,
};

//code store

#[derive(Default)]
pub struct AreaCodeIndexDataHashMap {
    hash: HashMap<String, AreaCodeIndexInfo>,
    version: String,
}

impl AreaCodeIndexData for AreaCodeIndexDataHashMap {
    fn set(&mut self, key: &str, val: AreaCodeIndexInfo) -> AreaResult<()> {
        self.hash.insert(key.to_string(), val);
        Ok(())
    }
    fn clear(&mut self) -> AreaResult<()> {
        self.hash = HashMap::new();
        Ok(())
    }
    fn get(&self, key: &str) -> Option<AreaCodeIndexInfo> {
        self.hash.get(key).map(|e| e.to_owned())
    }
    fn save(&mut self, version: &str) -> AreaResult<()> {
        version.clone_into(&mut self.version);
        Ok(())
    }
    fn init(&mut self) -> AreaResult<()> {
        Ok(())
    }
    fn version(&self) -> String {
        self.version.clone()
    }
}

pub struct AreaCodeIndexTreeItemHashMap {
    data: HashMap<String, AreaCodeIndexTreeItemHashMap>,
}
pub struct AreaCodeIndexTreeHashMap {
    data: AreaCodeIndexTreeItemHashMap,
    version: String,
}
impl Default for AreaCodeIndexTreeHashMap {
    fn default() -> Self {
        Self {
            data: AreaCodeIndexTreeItemHashMap {
                data: HashMap::new(),
            },
            version: "".to_string(),
        }
    }
}
impl AreaCodeIndexTree for AreaCodeIndexTreeHashMap {
    fn clear(&mut self) -> AreaResult<()> {
        self.data = AreaCodeIndexTreeItemHashMap {
            data: HashMap::new(),
        };
        Ok(())
    }
    fn init(&mut self) -> AreaResult<()> {
        Ok(())
    }
    fn add(&mut self, code_data: Vec<&str>) -> AreaResult<()> {
        let mut pe_ref = &mut self.data;
        for ddd in code_data {
            pe_ref = pe_ref
                .data
                .entry(ddd.to_string())
                .or_insert(AreaCodeIndexTreeItemHashMap {
                    data: HashMap::new(),
                });
        }
        Ok(())
    }
    fn childs(&self, code_data: &[&str]) -> Option<Vec<(String, bool)>> {
        let mut pe_ref = &self.data;
        for ddd in code_data {
            if let Some(tmp) = pe_ref.data.get(*ddd) {
                pe_ref = tmp
            } else {
                return None;
            }
        }
        Some(
            pe_ref
                .data
                .iter()
                .map(|(code, dat)| (code.to_owned(), !dat.data.is_empty()))
                .collect(),
        )
    }
    fn save(&mut self, version: &str) -> AreaResult<()> {
        version.clone_into(&mut self.version);
        Ok(())
    }
    fn version(&self) -> String {
        self.version.clone()
    }
}

//code 公共适配接口
pub struct AreaCodeTantivyMemory {
    index_size: usize,
}
impl AreaCodeTantivyMemory {
    pub fn new(index_size: usize) -> Self {
        Self { index_size }
    }
}
impl AreaCodeTantivy for AreaCodeTantivyMemory {
    fn create_index(&self, schema: Schema) -> AreaResult<Index> {
        Ok(IndexBuilder::new()
            .schema(schema)
            .settings(IndexSettings {
                docstore_compression: Compressor::None,
                ..IndexSettings::default()
            })
            .create_in_ram()?)
    }
    fn index_writer(&self, index: &Index) -> AreaResult<IndexWriter> {
        Ok(index.writer(self.index_size)?)
    }
}

#[derive(Default)]
pub struct MemoryAreaCodeProvider {}
impl AreaCodeProvider for MemoryAreaCodeProvider {
    type CD = AreaCodeIndexDataHashMap;
    type CT = AreaCodeIndexTreeHashMap;
    type TT = AreaCodeTantivyMemory;
}

//geo 公共适配接口

#[derive(Default)]
pub struct MemoryAreaGeoProvider {
    version: String,
    max_distance: u64,
    center_data: Vec<(usize, String, Point)>,
    polygon_data: HashMap<usize, (LineString, Vec<LineString>)>,
    max_index: usize,
}

impl AreaGeoProvider for MemoryAreaGeoProvider {
    fn clear(&mut self) -> AreaResult<()> {
        self.center_data = vec![];
        self.polygon_data = HashMap::new();
        Ok(())
    }
    fn save(&mut self, max_distance: u64, version: &str) -> AreaResult<()> {
        version.clone_into(&mut self.version);
        self.max_distance = max_distance;
        Ok(())
    }
    fn get_max_distance(&self) -> AreaResult<u64> {
        Ok(self.max_distance)
    }
    fn init(&mut self) -> AreaResult<()> {
        Ok(())
    }
    fn push_data(
        &mut self,
        code: &str,
        center_geo: Point,
        exterior: LineString,
        interiors: Vec<LineString>,
    ) -> AreaResult<()> {
        self.max_index += 1;
        self.center_data
            .push((self.max_index, code.to_string(), center_geo));
        self.polygon_data
            .insert(self.max_index, (exterior, interiors));
        Ok(())
    }
    fn get_center_data(&self) -> AreaResult<Vec<(usize, String, Point)>> {
        Ok(self.center_data.iter().map(|t| t.to_owned()).collect())
    }
    fn get_polygon_data(&self, i: &usize) -> Option<AreaGeoIndexInfo> {
        self.polygon_data
            .get(i)
            .map(|e| AreaGeoIndexInfo::new(e.0.to_owned(), e.1.to_owned()))
    }
    fn version(&self) -> String {
        self.version.clone()
    }
}

//把数据仅保存在内存
//每次启动都重新构建索引，启动比较慢
//索引数据都保存在内存中，启动后占用内存是磁盘的10倍以上
//好处是响应速度快，是mmap方式的2倍以上
pub struct AreaStoreMemory {
    index_size: usize,
}

impl AreaStoreMemory {
    pub fn new(index_size: usize) -> Self {
        Self { index_size }
    }
}

impl Default for AreaStoreMemory {
    fn default() -> Self {
        Self {
            index_size: 500_000_000,
        }
    }
}

impl AreaStoreProvider for AreaStoreMemory {
    type C = MemoryAreaCodeProvider;
    type G = MemoryAreaGeoProvider;
    fn create_code(&self) -> AreaResult<AreaCode<Self::C>> {
        AreaCode::new(
            AreaCodeTantivyMemory::new(self.index_size),
            AreaCodeIndexDataHashMap::default(),
            AreaCodeIndexTreeHashMap::default(),
        )
    }
    fn create_geo(&self) -> AreaResult<AreaGeo<Self::G>> {
        AreaGeo::new(MemoryAreaGeoProvider::default())
    }
}
