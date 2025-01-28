use image::{ImageBuffer, ImageFormat, Rgb, RgbImage};
use rxing::common::HybridBinarizer;
use rxing::multi::{GenericMultipleBarcodeReader, MultipleBarcodeReader};
use rxing::{
    BarcodeFormat, BinaryBitmap, BufferedImageLuminanceSource, DecodeHints, EncodeHintType,
    EncodeHintValue, EncodeHints, EncodingHintDictionary, MultiFormatReader, MultiFormatWriter,
    MultiUseMultiFormatReader, RXingResult, Reader, Writer,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tokio::io::{AsyncReadExt, BufReader};

use crate::model::BarcodeCreateModel;

use super::BarCodeResult;

#[derive(Default)]
pub struct BarCodeCore {}

fn hex_to_rgb(hex_code: &str) -> Option<Rgb<u8>> {
    if hex_code.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex_code[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex_code[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex_code[4..6], 16).unwrap_or(0);
    Some(Rgb([r, g, b]))
}
impl BarCodeCore {
    pub fn render(
        &self,
        create: &BarcodeCreateModel,
        contents: &str,
    ) -> BarCodeResult<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        let mut hints: EncodingHintDictionary = HashMap::new();
        hints.insert(
            EncodeHintType::CHARACTER_SET,
            EncodeHintValue::CharacterSet("UTF-8".to_owned()),
        );
        let format = create.barcode_type.to_owned().into();
        if format == BarcodeFormat::QR_CODE {
            hints.insert(
                EncodeHintType::MARGIN,
                EncodeHintValue::Margin(create.margin.to_string()),
            );
        }
        let bit_matrix = MultiFormatWriter.encode_with_hints(
            contents,
            &format,
            create.image_width,
            create.image_height,
            &EncodeHints::from(hints),
        )?;
        let width = bit_matrix.width();
        let height = bit_matrix.height();
        let mut img = RgbImage::new(width as u32, height as u32);

        let bg = hex_to_rgb(&create.image_background).unwrap_or(Rgb([255, 255, 255]));
        let fg = hex_to_rgb(&create.image_color).unwrap_or(Rgb([0, 0, 0]));
        for x in 0..width {
            for y in 0..height {
                img.put_pixel(x, y, if bit_matrix.get(x, y) { fg } else { bg });
            }
        }
        Ok(img)
    }
}

#[allow(dead_code)]
pub struct ParseParam<'t> {
    pub try_harder: Option<bool>,
    pub decode_multi: Option<bool>,
    pub barcode_types: Option<Vec<&'t str>>,
    pub other: Option<&'t str>,
    pub pure_barcode: Option<bool>,
    pub character_set: Option<&'t str>,
    pub allowed_lengths: Option<&'t [u32]>,
    pub assume_code_39_check_digit: Option<bool>,
    pub assume_gs1: Option<bool>,
    pub return_codabar_start_end: Option<bool>,
    pub allowed_ean_extensions: Option<&'t [u32]>,
    pub also_inverted: Option<bool>,
}

impl BarCodeCore {
    pub async fn decode(
        &self,
        file_name: impl AsRef<Path>,
        extension: &str,
        param: &ParseParam<'_>,
    ) -> Result<Vec<RXingResult>, String> {
        let mut hints: rxing::DecodingHintDictionary = HashMap::new();
        if let Some(ref other) = param.other {
            hints.insert(
                rxing::DecodeHintType::OTHER,
                rxing::DecodeHintValue::Other(other.to_string()),
            );
        }
        if let Some(pure_barcode) = param.pure_barcode {
            hints.insert(
                rxing::DecodeHintType::PURE_BARCODE,
                rxing::DecodeHintValue::PureBarcode(pure_barcode),
            );
        }

        if let Some(character_set) = param.character_set {
            hints.insert(
                rxing::DecodeHintType::CHARACTER_SET,
                rxing::DecodeHintValue::CharacterSet(character_set.to_string()),
            );
        }
        if let Some(allowed_lengths) = param.allowed_lengths {
            hints.insert(
                rxing::DecodeHintType::ALLOWED_LENGTHS,
                rxing::DecodeHintValue::AllowedLengths(allowed_lengths.to_vec()),
            );
        }
        if let Some(assume_code_39_check_digit) = param.assume_code_39_check_digit {
            hints.insert(
                rxing::DecodeHintType::ASSUME_CODE_39_CHECK_DIGIT,
                rxing::DecodeHintValue::AssumeCode39CheckDigit(assume_code_39_check_digit),
            );
        }
        if let Some(assume_gs1) = param.assume_gs1 {
            hints.insert(
                rxing::DecodeHintType::ASSUME_GS1,
                rxing::DecodeHintValue::AssumeGs1(assume_gs1),
            );
        }
        if let Some(return_codabar_start_end) = param.return_codabar_start_end {
            hints.insert(
                rxing::DecodeHintType::RETURN_CODABAR_START_END,
                rxing::DecodeHintValue::ReturnCodabarStartEnd(return_codabar_start_end),
            );
        }
        if let Some(allowed_ean_extensions) = param.allowed_ean_extensions {
            hints.insert(
                rxing::DecodeHintType::ALLOWED_EAN_EXTENSIONS,
                rxing::DecodeHintValue::AllowedEanExtensions(allowed_ean_extensions.to_vec()),
            );
        }
        if let Some(also_inverted) = param.also_inverted {
            hints.insert(
                rxing::DecodeHintType::ALSO_INVERTED,
                rxing::DecodeHintValue::AlsoInverted(also_inverted),
            );
        }
        if let Some(try_header) = param.try_harder {
            hints.insert(
                rxing::DecodeHintType::TRY_HARDER,
                rxing::DecodeHintValue::TryHarder(try_header),
            );
        }
        if let Some(ref barcode_type) = param.barcode_types {
            let mut hashset = HashSet::new();
            for e in barcode_type.iter() {
                let tt = e.to_owned().into();
                hashset.insert(tt);
            }
            hints.insert(
                rxing::DecodeHintType::POSSIBLE_FORMATS,
                rxing::DecodeHintValue::PossibleFormats(hashset),
            );
        }

        if extension == "svg" {
            if param.decode_multi.unwrap_or(true) {
                Ok(rxing::helpers::detect_multiple_in_svg_with_hints(
                    file_name.as_ref().to_string_lossy().as_ref(),
                    &mut DecodeHints::from(hints),
                )
                .map_err(|e| format!("parse fail:{}", e))?)
            } else {
                let result = rxing::helpers::detect_in_svg_with_hints(
                    file_name.as_ref().to_string_lossy().as_ref(),
                    None,
                    &mut DecodeHints::from(hints),
                )
                .map_err(|e| format!("parse fail:{}", e));
                Ok(vec![result?])
            }
        } else {
            let file = tokio::fs::File::open(&file_name)
                .await
                .map_err(|e| format!("{} open fail:{}", file_name.as_ref().display(), e))?;
            let mut reader = BufReader::new(file);
            let mut buffer = Vec::new();
            reader
                .read_to_end(&mut buffer)
                .await
                .map_err(|e| format!("{} read fail:{}", file_name.as_ref().display(), e))?;
            let image_format = ImageFormat::from_extension(extension)
                .ok_or_else(|| format!("format not support:{}", extension))?;

            let img = image::load_from_memory_with_format(&buffer, image_format)
                .map_err(|e| format!("load image fail:{}", e))?;

            if param.decode_multi.unwrap_or(false) {
                let multi_format_reader = MultiUseMultiFormatReader::default();
                let mut scanner = GenericMultipleBarcodeReader::new(multi_format_reader);

                Ok(scanner
                    .decode_multiple_with_hints(
                        &mut BinaryBitmap::new(HybridBinarizer::new(
                            BufferedImageLuminanceSource::new(img),
                        )),
                        &DecodeHints::from(hints),
                    )
                    .map_err(|e| format!("parse fail:{}", e))?)
            } else {
                let mut multi_format_reader = MultiFormatReader::default();
                let result = multi_format_reader
                    .decode_with_hints(
                        &mut BinaryBitmap::new(HybridBinarizer::new(
                            BufferedImageLuminanceSource::new(img),
                        )),
                        &DecodeHints::from(hints),
                    )
                    .map_err(|e| format!("parse fail:{}", e));
                Ok(vec![result?])
            }
        }
    }
}
