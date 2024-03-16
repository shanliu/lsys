use crate::model::BarcodeOutputModel;

use super::BarCodeResult;

pub(crate) fn create_bar_code(_config: &BarcodeOutputModel, _data: &str) -> BarCodeResult<String> {
    Ok("".to_string())
}
