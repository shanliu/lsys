#[allow(dead_code)]
pub struct ParseParam {
    pub ean_add_on_symbol: u8,
    pub binaries: u8,
    pub format: u32,
    pub try_harder: bool,
    pub try_rotate: bool,
    pub is_pure: bool,
    pub character: String,
    pub try_code39_extended_mode: bool,
    pub assume_code39_check_digit: bool,
    pub assume_itfcheck_digit: bool,
    pub return_codabar_start_end: bool,
}

pub struct ParseDataPoint {
    pub x: i32,
    pub y: i32,
}

pub struct ParseData {
    pub width: u64,
    pub height: u64,
    pub status: u8,
    pub is_valid: bool,
    pub text: String,
    pub format: u8,
    pub symbology_identifier: String,
    pub position: Vec<ParseDataPoint>,
    pub orientation: u8,
    pub ec_level: String,
    pub line_count: u32,
    pub gtin_country: String,
    pub gtin_ean_add_on: String,
    pub gtin_price: String,
    pub gtin_issue_nr: String,
    pub is_part_of_sequence: bool,
    pub is_last_in_sequence: bool,
}
