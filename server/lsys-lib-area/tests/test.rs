#[cfg(feature = "data-mysql")]
#[test]
fn test_mysql() {
    use lsys_lib_area::AreaStoreMemory;
    let uri = "mysql://root:@127.0.0.1:3306/test";
    let mysql = lsys_lib_area::MysqlAreaData::new(
        lsys_lib_area::MysqlAreaCodeData::from_uri(uri, None),
        Some(lsys_lib_area::MysqlAreaGeoData::from_uri(uri, None)),
    );
    let area = lsys_lib_area::AreaDao::from_mysql_mem(mysql, AreaStoreMemory::default()).unwrap();
    test_branch(&area);
    area.geo_reload().unwrap();
    area.code_reload().unwrap();
    test_branch(&area);
}

#[cfg(any(feature = "data-sqlite", feature = "data-sqlite-source"))]
#[test]
fn test_sqlite() {
    use std::path::PathBuf;

    use lsys_lib_area::AreaStoreMemory;
    //下载 area-data.db.zip 解压后得到 area-data.db
    let uri = "data/area-data.db";
    let sqlite = lsys_lib_area::SqliteAreaData::new(
        lsys_lib_area::SqliteAreaCodeData::from_path(PathBuf::from(uri)),
        Some(lsys_lib_area::SqliteAreaGeoData::from_path(PathBuf::from(
            uri,
        ))),
    );
    let area = lsys_lib_area::AreaDao::from_sqlite_mem(sqlite, AreaStoreMemory::default()).unwrap();
    test_branch(&area);
    area.geo_reload().unwrap();
    area.code_reload().unwrap();
    test_branch(&area);
}

#[cfg(feature = "data-csv")]
#[test]
fn test_csv() {
    use lsys_lib_area::{AreaStoreDisk, AreaStoreMemory};
    let code_path = std::path::PathBuf::from(format!(
        "{}/data/2023-7-area-code.csv.gz",
        env!("CARGO_MANIFEST_DIR")
    ));
    let geo_path = std::path::PathBuf::from(format!(
        "{}/data/2023-7-area-geo.csv.gz",
        env!("CARGO_MANIFEST_DIR")
    ));

    //mem
    let geo_data =
        { Some(lsys_lib_area::CsvAreaGeoData::from_inner_path(geo_path.clone(), true).unwrap()) };
    let data = lsys_lib_area::CsvAreaData::new(
        lsys_lib_area::CsvAreaCodeData::from_inner_path(code_path.clone(), true).unwrap(),
        geo_data,
    );
    test_branch(&lsys_lib_area::AreaDao::from_csv_mem(data, AreaStoreMemory::default()).unwrap());

    //disk
    let geo_data =
        { Some(lsys_lib_area::CsvAreaGeoData::from_inner_path(geo_path, true).unwrap()) };
    let data1 = lsys_lib_area::CsvAreaData::new(
        lsys_lib_area::CsvAreaCodeData::from_inner_path(code_path, true).unwrap(),
        geo_data,
    );
    test_branch(
        &lsys_lib_area::AreaDao::from_csv_disk(
            data1,
            AreaStoreDisk::new(
                format!("{}/data/tmp", env!("CARGO_MANIFEST_DIR"))
                    .as_str()
                    .into(),
                None,
            )
            .unwrap()
            .clear()
            .unwrap(),
        )
        .unwrap(),
    );
}

#[allow(dead_code)]
fn test_branch(area: &lsys_lib_area::AreaDao) {
    for _ in 0..10 {
        let start = std::time::Instant::now();
        area.code_childs("441403").unwrap();
        let duration = start.elapsed();
        println!("code_childs is: {:?}", duration);
    }

    for _ in 0..10 {
        let start = std::time::Instant::now();
        area.code_find("130731").unwrap();
        let duration = start.elapsed();
        println!("code_find is: {:?}", duration);
    }

    for _ in 0..10 {
        let start = std::time::Instant::now();
        let res = area.code_search("广东 榕岗", 10).unwrap();
        let duration = start.elapsed();
        println!("{:?}", res[0]);
        println!("code_search is: {:?}", duration);
        let start = std::time::Instant::now();
        let res = area.code_search("guang dong", 10).unwrap();
        let duration = start.elapsed();
        println!("{:?}", res[0]);
        println!("code_search is: {:?}", duration);
    }
    for i in 0..10 {
        let start = std::time::Instant::now();
        area.geo_search(
            26.61474 + (i as f64 / 1000.0),
            114.13548 + (i as f64 / 1000.0),
        )
        .unwrap();
        let duration = start.elapsed();
        println!("geo_search is: {:?}", duration);
    }
}
