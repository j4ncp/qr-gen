use std::cmp;

use crate::config::{Size, ECCLevel, Encoding};

use image;

/// During the assembly of the QR code pixel matrix
/// there are different value codes used as pixel values
/// to indicate pixels that will be filled in later.
/// As such those later stages can identify those pixels
/// easier. Final values are only 0 (black) and 255 (white).
/// All other values are codes, and are used in the following way:
///   100: everything not filled by the canvas creation function


/// Creates a finder pattern image (concentric squares
/// including the white separator around the finder
/// pattern)
fn create_finder_pattern() -> image::GrayImage {
    image::GrayImage::from_fn(9,9, |x, y| {
        let r = cmp::max((x as i32 - 4).abs(), (y as i32 - 4).abs());
        if r < 2 || r == 3 {
            image::Luma([0u8])
        } else {
            image::Luma([255u8])
        }
    })
}

/// Creates
fn create_standard_qt_canvas(size: u8) -> image::GrayImage {
    assert!(size >= 1 && size <= 40);
    let s = 17 + 4 * size as u32 + 8; // the +8 is for the quiet zone, 4 to each side
    let mut mask = image::GrayImage::from_pixel(s, s, image::Luma([100u8]));

    // mark quiet area
    for i in 0..s {
        for j in 0..4 {
            mask[(j, i)] = image::Luma([255u8]);
            mask[(i, j)] = image::Luma([255u8]);
            mask[(s - j - 1, i)] = image::Luma([255u8]);
            mask[(i, s - j - 1)] = image::Luma([255u8]);
        }
    }

    // apply 3 finder patterns in top and left corners
    let finder = create_finder_pattern();
    image::imageops::overlay(&mut mask, &finder, 3, 3);
    image::imageops::overlay(&mut mask, &finder, 3, s - 12);
    image::imageops::overlay(&mut mask, &finder, s - 12, 3);

    // return canvas
    mask
}


fn create_mini_qr_canvas(size: u8) -> image::GrayImage {
    assert!(size >= 1 && size <= 4);
    let s = 9 + 2 * size as u32 + 4;  // the +4 is for the quiet zone, 2 to each side
    let mut mask = image::GrayImage::from_pixel(s, s, image::Luma([100u8]));

    // mark quiet area
    for i in 0..s {
        for j in 0..2 {
            mask[(j, i)] = image::Luma([255u8]);
            mask[(i, j)] = image::Luma([255u8]);
            mask[(s - j - 1, i)] = image::Luma([255u8]);
            mask[(i, s - j - 1)] = image::Luma([255u8]);
        }
    }

    // apply finder pattern
    image::imageops::overlay(&mut mask, &create_finder_pattern(), 1, 1);

    // return canvas
    mask
}

/// Return a basic QR image with all the basic furnishings
/// of a QR code: the finder patterns, alignment patterns
/// and timing patterns.
fn create_qr_canvas(size: Size) -> image::GrayImage {
    match size {
        Size::Micro(s) => create_mini_qr_canvas(s),
        Size::Standard(s) => create_standard_qt_canvas(s)
    }
}

pub fn create_qr_code(content: &str,
                      size: Size,
                      level: ECCLevel,
                      encoding: Option<Encoding>) -> image::GrayImage {
    //assert!()
    // TODO
    image::GrayImage::new(20, 20)
}


//-------------------------------------------------------------------
// TESTS
//-------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;

    #[test]
    fn test_canvas_sizes() {
        assert_eq!(create_qr_canvas(Size::Micro(1)).dimensions(), (11+4, 11+4));
        assert_eq!(create_qr_canvas(Size::Micro(2)).dimensions(), (13+4, 13+4));
        assert_eq!(create_qr_canvas(Size::Micro(3)).dimensions(), (15+4, 15+4));
        assert_eq!(create_qr_canvas(Size::Micro(4)).dimensions(), (17+4, 17+4));
        assert_eq!(create_qr_canvas(Size::Standard(1)).dimensions(), (21+8, 21+8));
        assert_eq!(create_qr_canvas(Size::Standard(2)).dimensions(), (25+8, 25+8));
        assert_eq!(create_qr_canvas(Size::Standard(40)).dimensions(), (177+8, 177+8));
    }

    #[test]
    #[should_panic]
    fn test_invalid_size1() {
        create_qr_canvas(Size::Micro(0));
    }

    #[test]
    #[should_panic]
    fn test_invalid_size2() {
        create_qr_canvas(Size::Micro(5));
    }

    #[test]
    #[should_panic]
    fn test_invalid_size3() {
        create_qr_canvas(Size::Standard(0));
    }

    #[test]
    #[should_panic]
    fn test_invalid_size4() {
        create_qr_canvas(Size::Standard(41));
    }

    #[test]
    fn test_make_canvasses() {
        create_qr_canvas(Size::Micro(4)).save("./tmp_micro.png");
        create_qr_canvas(Size::Standard(4)).save("./tmp_standard.png");
    }
}
