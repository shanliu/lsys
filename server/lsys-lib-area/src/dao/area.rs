use parking_lot::RwLock;

use crate::{
    AreaCode, AreaCodeData, AreaCodeItem, AreaCodeProvider, AreaCodeRelatedItem, AreaGeo,
    AreaGeoData, AreaGeoProvider, AreaResult, AreaSearchItem,
};

pub trait AreaDataProvider: Sized + 'static {
    fn code_data(&self) -> AreaResult<Vec<AreaCodeData>>;
    fn geo_data(&self) -> AreaResult<Vec<AreaGeoData>>;
    fn code_data_version(&self) -> String;
    fn geo_data_version(&self) -> String;
}

pub trait AreaStoreProvider: Sized + 'static {
    type C: AreaCodeProvider;
    type G: AreaGeoProvider;
    fn create_code(&self) -> AreaResult<AreaCode<Self::C>>;
    fn create_geo(&self) -> AreaResult<AreaGeo<Self::G>>;
}

pub struct Area<DO: AreaStoreProvider, DD: AreaDataProvider> {
    data_provider: DD,
    code: RwLock<AreaCode<DO::C>>,
    geo: RwLock<AreaGeo<DO::G>>,
}

impl<DO: AreaStoreProvider, DD: AreaDataProvider> Area<DO, DD> {
    pub fn new(dao_provider: DO, data_provider: DD) -> AreaResult<Self> {
        let code = dao_provider.create_code()?;
        let geo = dao_provider.create_geo()?;
        let out = Self {
            data_provider,
            code: RwLock::new(code),
            geo: RwLock::new(geo),
        };
        out.code_reload()?;
        out.geo_reload()?;
        Ok(out)
    }
    pub fn code_reload(&self) -> AreaResult<()> {
        let data_ver = self.data_provider.code_data_version();
        if data_ver.is_empty() || !self.code.read().version_match(&data_ver) {
            self.code
                .write()
                .load_data(self.data_provider.code_data()?, &data_ver)?;
        }
        Ok(())
    }
    pub fn geo_reload(&self) -> AreaResult<()> {
        let data_ver = self.data_provider.geo_data_version();
        if data_ver.is_empty() || !self.geo.read().version_match(&data_ver) {
            self.geo
                .write()
                .load_data(self.data_provider.geo_data()?, &data_ver)?;
        }
        Ok(())
    }
    pub fn code_childs(&self, code: &str) -> AreaResult<Vec<AreaCodeItem>> {
        self.code.read().childs(code).map(|mut e| {
            e.sort_by(|a, b| a.code.cmp(&b.code));
            e
        })
    }
    pub fn code_find(&self, code: &str) -> AreaResult<Vec<AreaCodeItem>> {
        self.code.read().find(code)
    }
    pub fn code_related(&self, code: &str) -> AreaResult<Vec<Vec<AreaCodeRelatedItem>>> {
        self.code.read().related(code).map(|e| {
            e.into_iter()
                .map(|mut ie| {
                    ie.sort_by(|a, b| a.item.code.cmp(&b.item.code));
                    ie
                })
                .collect::<Vec<_>>()
        })
    }
    pub fn code_search(&self, name: &str, limit: usize) -> AreaResult<Vec<AreaSearchItem>> {
        if name.trim().is_empty() {
            let mut out = Vec::with_capacity(limit);
            for tmp in self.code.read().childs("")? {
                out.push(AreaSearchItem {
                    item: vec![tmp],
                    source: 1.0,
                })
            }
            out.truncate(limit);
            return Ok(out);
        }
        self.code.read().search(name, limit)
    }
    pub fn geo_search(&self, lat: f64, lng: f64) -> AreaResult<Vec<AreaCodeItem>> {
        if !(0.0..=90.0).contains(&lat) || !(0.0..=180.0).contains(&lng) {
            return Ok(vec![]);
        }
        let tmp = self.geo.read();
        let code = tmp.search(&geo::coord! { x:lng, y:lat})?;
        self.code.read().find(&code)
    }
}
