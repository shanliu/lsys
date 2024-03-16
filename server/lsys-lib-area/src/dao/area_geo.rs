use crate::{AreaError, AreaResult};
use geo::coordinate_position::{CoordPos, CoordinatePosition};
use geo::{coord, BoundingRect, Centroid, Coord, GeodesicDistance, LineString, Point, Polygon};
use rayon::prelude::*;
// 初始化

// OutlierDetection 坐标异常检测 :日志

// Centroid [省,市,县都预先算一遍],基于中心点构建B*TREE

// CoordPos, CoordinatePosition 坐标是否在某区域检测

#[derive(Debug, Clone)]
pub struct AreaGeoIndexInfo {
    detail: Polygon,
}

impl AreaGeoIndexInfo {
    pub fn new(exterior: LineString, interiors: Vec<LineString>) -> Self {
        AreaGeoIndexInfo {
            detail: Polygon::new(exterior, interiors),
        }
    }
}

pub struct AreaGeoDataItem {
    pub center: String,
    pub polygon: String,
}
pub struct AreaGeoData {
    pub code: String,
    pub item: Vec<AreaGeoDataItem>,
}

pub trait AreaGeoProvider {
    fn clear(&mut self) -> AreaResult<()>;
    fn save(&mut self, max_distance: u64, version: &str) -> AreaResult<()>;
    fn get_max_distance(&self) -> AreaResult<u64>;
    fn push_data(
        &mut self,
        code: &str,
        center_geo: Point,
        exterior: LineString,
        interiors: Vec<LineString>,
    ) -> AreaResult<()>;
    //返回所有权数据，如果返回引用，否则返回的数据当非内存数据时会麻烦的很
    fn get_center_data(&self) -> AreaResult<Vec<(usize, String, Point)>>;
    fn get_polygon_data(&self, i: &usize) -> Option<AreaGeoIndexInfo>;
    fn version(&self) -> String;
    fn init(&mut self) -> AreaResult<()>;
}

pub struct AreaGeo<AP: AreaGeoProvider> {
    geo_data: AP,
}

impl<AP: AreaGeoProvider> AreaGeo<AP> {
    pub fn new(mut geo_data: AP) -> AreaResult<Self> {
        geo_data.init()?;
        Ok(Self { geo_data })
    }
    pub fn version_match(&self, version: &str) -> bool {
        self.geo_data.version().as_str() == version
    }
    pub fn load_data(&mut self, area_geo_data: Vec<AreaGeoData>, version: &str) -> AreaResult<()> {
        self.geo_data.clear()?;
        let mut max_distance = 0;
        // let mut i = 0;
        for tmp_area in area_geo_data {
            for tmp_item in tmp_area.item.iter() {
                let ps = tmp_item.polygon.split(',').collect::<Vec<_>>();
                let mut cs = Vec::with_capacity(ps.len());
                for pt in ps {
                    let mut iter = pt.split_whitespace();
                    if let (Some(x), Some(y)) = (iter.next(), iter.next()) {
                        if let Ok(x) = x.parse::<f64>() {
                            if let Ok(y) = y.parse::<f64>() {
                                cs.push(coord! { x:x, y:y});
                            }
                        }
                    }
                }
                if cs.is_empty() {
                    continue;
                }
                let mut exterior = cs.into_iter().collect::<LineString<f64>>();
                exterior.close();
                let detail = Polygon::new(exterior.clone(), vec![]);

                if let Some(rc_tmp) = detail.bounding_rect() {
                    let top_left = std::convert::Into::<Point>::into(rc_tmp.min());
                    let bottom_right = std::convert::Into::<Point>::into(rc_tmp.max());
                    let longest_distance = top_left.geodesic_distance(&bottom_right);
                    if longest_distance > 0.0 && max_distance < longest_distance as u64 {
                        max_distance = longest_distance as u64;
                    }
                }

                let mut iter = tmp_item.center.split_whitespace();
                let center = if let (Some(x), Some(y)) = (iter.next(), iter.next()) {
                    if let Ok(x) = x.parse::<f64>() {
                        if let Ok(y) = y.parse::<f64>() {
                            Some(coord! { x:x, y:y})
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                let center = if center.is_none() {
                    detail.centroid()
                } else {
                    center.map(|e| e.into())
                };
                let center = match center {
                    Some(e) => e,
                    None => continue,
                };
                self.geo_data
                    .push_data(&tmp_area.code, center, exterior, vec![])?;
            }
        }
        self.geo_data.save(max_distance, version)?;
        Ok(())
    }
    /// 通过坐标获取可能区域
    pub fn search(&self, coord: &Coord) -> AreaResult<String> {
        let point = std::convert::Into::<Point>::into(coord.to_owned());
        let get_data = self.geo_data.get_center_data()?;
        let mut max_distance = self.geo_data.get_max_distance().unwrap_or(0);
        if max_distance == 0 {
            max_distance = 5500000
        }
        let mut dit_data = get_data
            .par_iter()
            .flat_map(|(i, code, center)| {
                let distance = center.geodesic_distance(&point).round() as u64;
                if distance > max_distance {
                    return None;
                }
                Some((*i, code.to_owned(), distance))
            })
            .collect::<Vec<_>>();
        dit_data.sort_by_key(|&(_, _, dit)| dit);
        dit_data.truncate(20);
        for (i, code, _) in dit_data {
            if let Some(ply_data) = self.geo_data.get_polygon_data(&i) {
                if ply_data.detail.coordinate_position(coord) == CoordPos::Inside {
                    return Ok(code);
                }
            }
        }
        Err(AreaError::NotFind(format!(
            "not any area :{},{}",
            coord.x, coord.y
        )))
    }
}
