use image::ImageBuffer;

pub use config::{ECCLevel, Encoding, Size};


//pub fn create_qr_code(content: &str)

mod config;
mod serialization;
mod rscoding;
mod bitcoding;


pub fn create_qr_code(content: &str,
                      size: Size,
                      level: ECCLevel,
                      encoding: Option<Encoding>) -> image::GrayImage {
    //assert!()
    // TODO
    image::GrayImage::new(20, 20)
}
