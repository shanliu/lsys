use std::{collections::HashMap, path::PathBuf, sync::Arc};

use area::area_handler;
use axum::{extract::Path, extract::Query, routing::get, Router};
use lsys_lib_area::{AreaDao, CsvAreaCodeData, CsvAreaData, CsvAreaGeoData};
use serde_json::json;
mod area;
#[tokio::main]
async fn main() {
    let mut code_path = PathBuf::from("../../data/2023-7-area-code.csv.gz");
    if !code_path.is_file() {
        code_path = PathBuf::from("data/2023-7-area-code.csv.gz");
    }
    let mut geo_path = PathBuf::from("../../data/2023-7-area-geo.csv.gz");
    if !geo_path.is_file() {
        geo_path = PathBuf::from("data/2023-7-area-geo.csv.gz");
    }
    let data = CsvAreaData::new(
        CsvAreaCodeData::from_inner_path(code_path, true).unwrap(),
        CsvAreaGeoData::from_inner_path(geo_path, true).ok(),
    );
    //内存方式
    // let area_dao =
    //     Arc::new(AreaDao::from_csv_mem(data, lsys_lib_area::AreaStoreMemory::default()).unwrap());
    //磁盘方式
    let mut index_dir = std::env::temp_dir();
    index_dir.push("area-data");
    let area_dao = Arc::new(
        AreaDao::from_csv_disk(
            data,
            lsys_lib_area::AreaStoreDisk::new(index_dir, None).unwrap(),
        )
        .unwrap(),
    );
    let app = Router::new().route("/area/:path", {
        let area_dao = Arc::clone(&area_dao);
        get(
            |Path(path): Path<String>, Query(params): Query<HashMap<String, String>>| async {
                match area_handler(path, params, area_dao).await {
                    Ok(data) => {
                        json!({
                            "status":true,
                            "msg":"ok",
                            "data":data
                        })
                    }
                    Err(err) => json!({
                        "status":false,
                        "msg":err.to_string(),
                    }),
                }
                .to_string()
            },
        )
    });
    axum::Server::bind(&"0.0.0.0:8081".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
