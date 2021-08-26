use image::ImageBuffer;

pub use config::{ECCLevel, Encoding, Size};

#[macro_use]
extern crate lazy_static;

//pub fn create_qr_code(content: &str)

mod config;
mod serialization;
mod rscoding;
mod bitcoding;
mod tables;


pub fn create_qr_code(content: &str,
                      size: Size,
                      level: ECCLevel,
                      encoding: Option<Encoding>) -> image::GrayImage {
    //assert!()
    // TODO
    image::GrayImage::new(20, 20)
}
