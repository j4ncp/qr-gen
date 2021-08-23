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

/// Creates an alignment pattern image
fn create_alignment_pattern() -> image::GrayImage {
    image::GrayImage::from_fn(5, 5, |x, y| {
        let r = cmp::max((x as i32 - 2).abs(), (y as i32 - 2).abs());
        if r % 2 == 0 {
            image::Luma([0u8])
        } else {
            image::Luma([255u8])
        }
    })
}

/// Creates a vector with alignment coordinates, i.e. the
/// numbers from the row of the table E.1 in Annex E
fn create_alignment_pattern_coord_list(size: u8) -> Vec<i32> {
    let mut row = Vec::new();
    row.push(6);
    if size >= 2 && size < 7 {
        row.push((size as i32 - 2) * 4 + 18);
    } else if size >= 7 && size < 14  {
        row.push((size as i32 - 7) * 2 + 22);
        row.push((size as i32 - 7) * 4 + 38);
    } else if size >= 14 && size < 21  {
        let a = ((size as i32 - 14) / 3) * 4 + 26;
        let b = (size as i32 - 14) * 4 + 66;
        row.push(a);
        row.push((a+b) / 2);
        row.push(b);
    } else if size >= 21 && size < 28 {
        // TODO
        let b = ((size as i32 - 21) / 2) * 4 + 50;
        let d = (size as i32 - 21) * 4 + 94;
        row.push(match size {
            21 => 28,
            22 => 26,
            23 => 30,
            24 => 28,
            25 => 32,
            26 => 30,
            27 => 34,
            _ => panic!("Can never get here")
        });
        row.push(b);
        row.push((b+d) / 2);
        row.push(d);
    } else if size >= 28 && size < 35 {
        row.extend_from_slice(match size {
            28 => &[26, 50, 74, 98, 122],
            29 => &[30, 54, 78, 102, 126],
            30 => &[26, 52, 78, 104, 130],
            31 => &[30, 56, 82, 108, 134],
            32 => &[34, 60, 86, 112, 138],
            33 => &[30, 58, 86, 114, 142],
            34 => &[34, 62, 90, 118, 146],
            _ => panic!("Can never get here")
        });
    } else if size >= 35 && size <= 40 {
        row.extend_from_slice(match size {
            35 => &[30, 54, 78, 102, 126, 150],
            36 => &[24, 50, 76, 102, 128, 154],
            37 => &[28, 54, 80, 106, 132, 158],
            38 => &[32, 58, 84, 110, 136, 162],
            39 => &[26, 54, 82, 110, 138, 166],
            40 => &[30, 58, 86, 114, 142, 170],
            _ => panic!("Can never get here")
        });
    }
    row
}


/// Creates a vector with alignment coordinate pairs (x,y),
/// from the entries returned by create_alignment_pattern_coord_list
fn get_alignment_pattern_points(size: u8) -> Vec<(i32, i32)> {
    let coords = create_alignment_pattern_coord_list(size);
    let last_coord_index = coords.len() - 1;
    let mut points = Vec::new();
    for (i, &s) in coords[..].iter().enumerate() {
        for (j, &t) in coords[..].iter().enumerate() {
            if (i == 0 && j == 0) ||
               (i == 0 && j == last_coord_index) ||
               (i == last_coord_index && j == 0) {
                continue;
            }
            points.push((s, t));
        }
    }
    points
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

    // mark timing patterns
    for i in 10..s-12 {
        let val = if i % 2 == 0 {image::Luma([0u8])} else {image::Luma([255u8])};
        mask[(10, i)] = val;
        mask[(i, 10)] = val;
    }

    // alignment patterns only for version >= 2
    if size >= 2 {
        // retrieve point list of alignment pattern center points
        let points = get_alignment_pattern_points(size);
        // get a pattern image
        let pattern = create_alignment_pattern();
        // paint them onto canvas
        for (x, y) in points {
            // the offset +2 we get by +4 from the quiet border
            // and -2 from the pattern center offset
            image::imageops::overlay(&mut mask, &pattern, x as u32 + 2, y as u32 + 2);
        }
    }

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

    // mark timing patterns
    for i in 10..s-2 {
        let val = if i % 2 == 0 {image::Luma([0u8])} else {image::Luma([255u8])};
        mask[(2, i)] = val;
        mask[(i, 2)] = val;
    }

    // no alignment patterns

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


//-------------------------------------------------------------------
// TESTS
//-------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImageView, ImageResult};

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
        create_qr_canvas(Size::Standard(7)).save("./tmp_standard.png");
    }

    #[test]
    fn test_tableE1() {
        assert_eq!(create_alignment_pattern_coord_list(3), [6, 22]);
        assert_eq!(create_alignment_pattern_coord_list(10), [6, 28, 50]);
        assert_eq!(create_alignment_pattern_coord_list(15), [6, 26, 48, 70]);
        assert_eq!(create_alignment_pattern_coord_list(20), [6, 34, 62, 90]);
        assert_eq!(create_alignment_pattern_coord_list(27), [6, 34, 62, 90, 118]);
        assert_eq!(create_alignment_pattern_coord_list(33), [6, 30, 58, 86, 114, 142]);
        assert_eq!(create_alignment_pattern_coord_list(40), [6, 30, 58, 86, 114, 142, 170]);
    }
}
