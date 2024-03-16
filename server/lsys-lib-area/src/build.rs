fn main() {
    #[cfg(feature = "data-csv")]
    {
        create_csv_zip_file("data/2023-7-area-geo.csv");
        create_csv_zip_file("data/2023-7-area-code.csv");
    }
    #[cfg(feature = "data-sqlite-source")]
    build_sqlite_form_source();
    #[cfg(feature = "lib-clib")]
    build_c_lib();
}
#[cfg(feature = "data-csv")]
fn create_csv_zip_file(file_path: &str) {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::fs::File;
    use std::io::{Read, Write};
    let file_lock = format!("{}.lock", file_path);
    let file_gz = format!("{}.gz", file_path);
    if std::path::Path::new(&file_path).exists() && !std::path::Path::new(&file_lock).exists() {
        let mut file = File::open(file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let mut encoder = GzEncoder::new(File::create(file_gz).unwrap(), Compression::best());
        encoder.write_all(contents.as_bytes()).unwrap();
        File::create(file_lock).unwrap();
    }
}
#[cfg(feature = "data-sqlite-source")]
fn build_sqlite_form_source() {
    let dir_path = format!("{}/sqlite-amalgamation", env!("CARGO_MANIFEST_DIR"));
    match std::fs::metadata(&dir_path) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                panic!("Path exists but is not a directory: {}", dir_path);
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                panic!("Please run {}{}sqlite_source.{} to download the sqlite source code, or use data-sqlite.",
                    env!("CARGO_MANIFEST_DIR"),
                    if cfg!(windows){'\\'}else{'/'},
                    if cfg!(windows){"cmd"}else{"sh"}
                );
            } else {
                panic!("Error getting metadata: {}", e);
            }
        }
    }
    use std::path::Path;
    println!("cargo:rustc-link-search=native=sqlite-amalgamation");
    println!("cargo:rustc-link-lib=static=sqlite3");
    let c_api_src_dir = Path::new("sqlite-amalgamation");
    cc::Build::new()
        .flag("-DSQLITE_ENABLE_RTREE=1")
        .include(c_api_src_dir)
        .files(&[c_api_src_dir.join("sqlite3.c")])
        .compile("sqlite3");
}

#[cfg(feature = "lib-clib")]
fn build_c_lib() {
    use std::{env, path::PathBuf};
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let header_file = out_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("lsys_lib_area.h");

    // Generate the C header file using cbindgen
    cbindgen::generate("")
        .expect("Unable to generate bindings")
        .write_to_file(&header_file);
    println!("cargo:rerun-if-changed=src/c_lib.rs");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:warning=Generated C header file: {:?}", header_file);
}
