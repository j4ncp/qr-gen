use image;

pub use config::{ECCLevel, Encoding, Size};

#[macro_use]
extern crate lazy_static;

//pub fn create_qr_code(content: &str)

pub mod config;
pub mod serialization;
pub mod reedsolomon;
pub mod bitcoding;
pub mod tables;



use bitcoding::*;
use reedsolomon::*;
use serialization::*;
use serialization::masking::apply_best_mask;



pub fn create_qr_code(content: &[u8],
                      size: Size,
                      level: ECCLevel,
                      encoding: Option<Encoding>) -> image::GrayImage {

    // TODO: guess best encoding

    // encode some data
    let (data_bytes, ecc_bytes) = {
        let mut encoder = QrBitRecorder::new();
        encode_data_segment(&mut encoder, content, encoding.unwrap(), size);
        let data_content = finalize_bitstream(&mut encoder, size, level);
        construct_codewords(&data_content, size, level)  // compute ecc bytes + interleave
    };

    // create a canvas
    let mut canvas = create_qr_canvas(size);
    insert_data_payload(&mut canvas, size, &data_bytes, &ecc_bytes);

    // determine best mask and apply it
    let (mask_code, mut masked_symbol) = apply_best_mask(&canvas, size);

    // apply format bits
    insert_format_info(&mut masked_symbol, size, level, mask_code);

    // apply version info
    insert_version_info(&mut masked_symbol, size);

    // done, return
    masked_symbol
}
