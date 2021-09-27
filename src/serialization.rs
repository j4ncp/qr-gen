use bitstream_io::{BigEndian, BitRead, BitReader};
use image;

use std::cmp;
use std::io::{Cursor, Read};

use crate::config::{ECCLevel, Size};

// CONSTANTS
pub const MARKER_ENCODING_REGION: image::Luma<u8> = image::Luma([100u8]);
pub const MARKER_FORMAT_INFORMATION: image::Luma<u8> = image::Luma([120u8]);
pub const MARKER_VERSION_INFORMATION: image::Luma<u8> = image::Luma([140u8]);

pub const BIT_WHITE: image::Luma<u8> = image::Luma([255u8]);
pub const BIT_BLACK: image::Luma<u8> = image::Luma([0u8]);


pub mod masking;

/// Creates a finder pattern image (concentric squares
/// including the white separator around the finder
/// pattern)
fn create_finder_pattern() -> image::GrayImage {
    image::GrayImage::from_fn(9,9, |x, y| {
        let r = cmp::max((x as i32 - 4).abs(), (y as i32 - 4).abs());
        if r < 2 || r == 3 {
            BIT_BLACK
        } else {
            BIT_WHITE
        }
    })
}

/// Creates an alignment pattern image
fn create_alignment_pattern() -> image::GrayImage {
    image::GrayImage::from_fn(5, 5, |x, y| {
        let r = cmp::max((x as i32 - 2).abs(), (y as i32 - 2).abs());
        if r % 2 == 0 {
            BIT_BLACK
        } else {
            BIT_WHITE
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
    let mut mask = image::GrayImage::from_pixel(s, s, MARKER_ENCODING_REGION);

    // mark quiet area
    for i in 0..s {
        for j in 0..4 {
            mask[(j, i)] = BIT_WHITE;
            mask[(i, j)] = BIT_WHITE;
            mask[(s - j - 1, i)] = BIT_WHITE;
            mask[(i, s - j - 1)] = BIT_WHITE;
        }
    }

    // apply 3 finder patterns in top and left corners
    let finder = create_finder_pattern();
    image::imageops::overlay(&mut mask, &finder, 3, 3);
    image::imageops::overlay(&mut mask, &finder, 3, s - 12);
    image::imageops::overlay(&mut mask, &finder, s - 12, 3);

    // mark timing patterns
    for i in 10..s-12 {
        let val = if i % 2 == 0 {BIT_BLACK} else {BIT_WHITE};
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

    // mark format bits
    for i in 0..6 {
        mask[(12, 4+i)] = MARKER_FORMAT_INFORMATION;
        mask[(4+i, 12)] = MARKER_FORMAT_INFORMATION;
        mask[(s-5-i, 12)] = MARKER_FORMAT_INFORMATION;
        mask[(12, s-5-i)] = MARKER_FORMAT_INFORMATION;
    }
    mask[(12, 11)] = MARKER_FORMAT_INFORMATION;
    mask[(11, 12)] = MARKER_FORMAT_INFORMATION;
    mask[(12, 12)] = MARKER_FORMAT_INFORMATION;
    mask[(12, s-11)] = MARKER_FORMAT_INFORMATION;
    mask[(12, s-12)] = MARKER_FORMAT_INFORMATION;
    mask[(s-11, 12)] = MARKER_FORMAT_INFORMATION;
    mask[(s-12, 12)] = MARKER_FORMAT_INFORMATION;

    // mark version bits if applicable
    if size >= 7 {
        for i in 0..6 {
            for j in 0..3 {
                mask[(4+i, s-13-j)] = MARKER_FORMAT_INFORMATION;
                mask[(s-13-j, 4+i)] = MARKER_FORMAT_INFORMATION;
            }
        }
    }

    // return canvas
    mask
}


fn create_micro_qr_canvas(size: u8) -> image::GrayImage {
    assert!(size >= 1 && size <= 4);
    let s = 9 + 2 * size as u32 + 4;  // the +4 is for the quiet zone, 2 to each side
    let mut mask = image::GrayImage::from_pixel(s, s, MARKER_ENCODING_REGION);

    // mark quiet area
    for i in 0..s {
        for j in 0..2 {
            mask[(j, i)] = BIT_WHITE;
            mask[(i, j)] = BIT_WHITE;
            mask[(s - j - 1, i)] = BIT_WHITE;
            mask[(i, s - j - 1)] = BIT_WHITE;
        }
    }

    // apply finder pattern
    image::imageops::overlay(&mut mask, &create_finder_pattern(), 1, 1);

    // mark timing patterns
    for i in 10..s-2 {
        let val = if i % 2 == 0 {BIT_BLACK} else {BIT_WHITE};
        mask[(2, i)] = val;
        mask[(i, 2)] = val;
    }

    // no alignment patterns

    // mark format bits
    for i in 3..11 {
        mask[(10, i)] = MARKER_FORMAT_INFORMATION;
        mask[(i, 10)] = MARKER_FORMAT_INFORMATION;
    }

    // return canvas
    mask
}

/// Return a basic QR image with all the functional patterns
/// painted in: the finder patterns, alignment patterns
/// and timing patterns.
///
/// During the assembly of the QR code pixel matrix
/// there are different value codes used as pixel values
/// to indicate pixels that will be filled in later.
/// As such those later stages can identify those pixels
/// easier. Final values are only 0 (black) and 255 (white).
/// All other values are codes, and are used in the following way:
///   100: the encoding region, which receives the binary code
///   120: marks the format information bits (stripes along finders),
///        2x 15 bits
///   140: marks the version information bits (blocks near upper
///        right and lower left finder) 2x 18bits
///        (only present in codes of version 7 or up)
pub fn create_qr_canvas(size: Size) -> image::GrayImage {
    match size {
        Size::Micro(s) => create_micro_qr_canvas(s),
        Size::Standard(s) => create_standard_qt_canvas(s)
    }
}


/// Insert the data into the encoding region of a QR canvas created by the create_qr_canvas function
///
pub fn insert_data_payload(canvas: &mut image::GrayImage, size: Size, data_words: &[u8], ecc_words: &[u8]) {
    // the variables used to step through the cells/modules of the QR symbol.
    // x_step inverts from 1 to -1 and back in each step, no matter whether the symbol could be placed or not,
    // y_step inverts only when reaching the borders of the symbol.
    let mut x_step: i32 = -1;
    let mut y_step: i32 = -1;

    let mut x_cur: i32 = match size {
        Size::Micro(i) => 2 + 8 + 2*i as i32,
        Size::Standard(i) => 4 + 16 + 4*i as i32
    };
    let mut y_cur: i32 = x_cur;  // the symbol is square, and we start off from the lower right corner

    // write all data bits
    {
        // the number of bits to read from the data_words. For M1 and M3, only the first four bits of
        // the last byte is used.
        let bits_to_read = match size {
            Size::Micro(1) | Size::Micro(3) => data_words.len() * 8 - 4,
            _ => data_words.len() * 8
        };

        // create reader and start the process
        let mut reader =  BitReader::endian(Cursor::new(&data_words), BigEndian);

        for _i in 0..bits_to_read {
            let bit = reader.read_bit().unwrap();

            // place bit
            canvas[(x_cur as u32, y_cur as u32)] = if bit { BIT_BLACK } else { BIT_WHITE };

            // find next valid place for next bit
            loop {
                // check next candidate. Next step is either applying
                // just x_step (if it is negative) or both x_step and y_step (if x_step is positive)
                if x_step == -1 {
                    x_cur = x_cur + x_step;
                } else {
                    x_cur = x_cur + x_step;
                    y_cur = y_cur + y_step;
                }

                // flip x_step
                x_step = -x_step;

                // see if we need to turn around on the borders
                if y_cur < 0 {
                    y_cur = 0;
                    y_step = 1;
                    x_cur = x_cur - 2;
                } else if y_cur >= canvas.height() as i32 {
                    y_cur = canvas.height() as i32 - 1;
                    y_step = -1;
                    x_cur = x_cur - 2;
                }

                // if x_cur is negative, there is no chance of finding another
                // valid encoding pixel
                if x_cur < 0 {
                    // this should never happen here, since the EC blocks are not even placed yet!
                    panic!("Should never get here!");
                }

                if canvas[(x_cur as u32, y_cur as u32)] == MARKER_ENCODING_REGION {
                    // found a valid pixel!
                    break;
                }
                // else: go on looping
            }
        }
    }

    // now write all ECC bits. Very similar to data bits. We will also just take the current position
    // x_cur, y_cur and just go on from there.
    {
        // the number of bits to read from the data_words. For M1 and M3, only the first four bits of
        // the last byte is used.
        let bits_to_read = ecc_words.len() * 8;

        // create reader and start the process
        let mut reader =  BitReader::endian(Cursor::new(&ecc_words), BigEndian);

        for _i in 0..bits_to_read {
            let bit = reader.read_bit().unwrap();

            // place bit
            canvas[(x_cur as u32, y_cur as u32)] = if bit { BIT_BLACK } else { BIT_WHITE };

            // find next valid place for next bit
            loop {
                // check next candidate. Next step is either applying
                // just x_step (if it is negative) or both x_step and y_step (if x_step is positive)
                if x_step == -1 {
                    x_cur = x_cur + x_step;
                } else {
                    x_cur = x_cur + x_step;
                    y_cur = y_cur + y_step;
                }

                // flip x_step
                x_step = -x_step;

                // see if we need to turn around on the borders
                if y_cur < 0 {
                    y_cur = 0;
                    y_step = 1;
                    x_cur = x_cur - 2;
                } else if y_cur >= canvas.height() as i32 {
                    y_cur = canvas.height() as i32 - 1;
                    y_step = -1;
                    x_cur = x_cur - 2;
                }

                // if x_cur is negative, there is no chance of finding another
                // valid encoding pixel
                if x_cur < 0 {
                    // we can only get here under normal circumstances if the total number of
                    // codewords fits the symbol exactly, ie. there are no zero padding bits.
                    break;
                }

                if canvas[(x_cur as u32, y_cur as u32)] == MARKER_ENCODING_REGION {
                    // found a valid pixel!
                    break;
                }
                // else: go on looping
            }
        }
    }

    if x_cur > 0 {
        // if there are still encoding region bits, find the rest of them and zero them out (padding)
        loop {
            if canvas[(x_cur as u32, y_cur as u32)] == MARKER_ENCODING_REGION {
                // found a valid pixel!
                // set to zero
                canvas[(x_cur as u32, y_cur as u32)] = BIT_WHITE;
            }

            // check next candidate. Next step is either applying
            // just x_step (if it is negative) or both x_step and y_step (if x_step is positive)
            if x_step == -1 {
                x_cur = x_cur + x_step;
            } else {
                x_cur = x_cur + x_step;
                y_cur = y_cur + y_step;
            }

            // flip x_step
            x_step = -x_step;

            // see if we need to turn around on the borders
            if y_cur < 0 {
                y_cur = 0;
                y_step = 1;
                x_cur = x_cur - 2;
            } else if y_cur >= canvas.height() as i32 {
                y_cur = canvas.height() as i32 - 1;
                y_step = -1;
                x_cur = x_cur - 2;
            }

            // now we are really done
            if x_cur < 0 {
                break;
            }
        }
    }
}

//-------------------------------------------------------------------
// FORMAT & VERSION INFO BITS
//-------------------------------------------------------------------
// lookup tables for the added 10 ECC bits plus XORing for both standard and
// micro QR codes. See table C.1 in Annex C of the standard.
// The value of the 5 data bits is the index into the lookup table.

const FORMAT_INFOS_QR: [u16; 32] = [
    0x5412, 0x5125, 0x5e7c, 0x5b4b, 0x45f9, 0x40ce, 0x4f97, 0x4aa0, 0x77c4, 0x72f3, 0x7daa, 0x789d, 0x662f, 0x6318,
    0x6c41, 0x6976, 0x1689, 0x13be, 0x1ce7, 0x19d0, 0x0762, 0x0255, 0x0d0c, 0x083b, 0x355f, 0x3068, 0x3f31, 0x3a06,
    0x24b4, 0x2183, 0x2eda, 0x2bed,
];

const FORMAT_INFOS_MICRO_QR: [u16; 32] = [
    0x4445, 0x4172, 0x4e2b, 0x4b1c, 0x55ae, 0x5099, 0x5fc0, 0x5af7, 0x6793, 0x62a4, 0x6dfd, 0x68ca, 0x7678, 0x734f,
    0x7c16, 0x7921, 0x06de, 0x03e9, 0x0cb0, 0x0987, 0x1735, 0x1202, 0x1d5b, 0x186c, 0x2508, 0x203f, 0x2f66, 0x2a51,
    0x34e3, 0x31d4, 0x3e8d, 0x3bba,
];

// lookup table for version info bits, works similar to format info
static VERSION_INFOS: [u32; 34] = [
    0x07c94, 0x085bc, 0x09a99, 0x0a4d3, 0x0bbf6, 0x0c762, 0x0d847, 0x0e60d, 0x0f928, 0x10b78, 0x1145d, 0x12a17,
    0x13532, 0x149a6, 0x15683, 0x168c9, 0x177ec, 0x18ec4, 0x191e1, 0x1afab, 0x1b08e, 0x1cc1a, 0x1d33f, 0x1ed75,
    0x1f250, 0x209d5, 0x216f0, 0x228ba, 0x2379f, 0x24b0b, 0x2542e, 0x26a64, 0x27541, 0x28c69,
];


// coordinates in the QR symbol where to write format and version bits. Do not include quiet region.
static VERSION_INFO_COORDS_BL: [(i16, i16); 18] = [
    (5, -9), (5, -10), (5, -11),
    (4, -9), (4, -10), (4, -11),
    (3, -9), (3, -10), (3, -11),
    (2, -9), (2, -10), (2, -11),
    (1, -9), (1, -10), (1, -11),
    (0, -9), (0, -10), (0, -11),
];

static VERSION_INFO_COORDS_TR: [(i16, i16); 18] = [
    (-9, 5), (-10, 5), (-11, 5),
    (-9, 4), (-10, 4), (-11, 4),
    (-9, 3), (-10, 3), (-11, 3),
    (-9, 2), (-10, 2), (-11, 2),
    (-9, 1), (-10, 1), (-11, 1),
    (-9, 0), (-10, 0), (-11, 0),
];

static FORMAT_INFO_COORDS_QR_MAIN: [(i16, i16); 15] = [
    (0, 8), (1, 8), (2, 8), (3, 8), (4, 8), (5, 8), (7, 8), (8, 8),
    (8, 7), (8, 5), (8, 4), (8, 3), (8, 2), (8, 1), (8, 0),
];

static FORMAT_INFO_COORDS_QR_SIDE: [(i16, i16); 15] = [
    (8, -1), (8, -2), (8, -3), (8, -4), (8, -5), (8, -6), (8, -7), (-8, 8),
    (-7, 8), (-6, 8), (-5, 8), (-4, 8), (-3, 8), (-2, 8), (-1, 8),
];

static FORMAT_INFO_COORDS_MICRO_QR: [(i16, i16); 15] = [
    (1, 8), (2, 8), (3, 8), (4, 8), (5, 8), (6, 8), (7, 8), (8, 8),
    (8, 7), (8, 6), (8, 5), (8, 4), (8, 3), (8, 2), (8, 1),
];

// helper function to write format or version bits to given coordinates in QR code
// bits are the bits actually to be written (big-endian order), num_bits is how many
// bits to write. Obviously this function supports writing only up to 32 bits at a time
fn insert_bits_at(symbol: &mut image::GrayImage, bits: u32, num_bits: u32, coords: &[(i16, i16)], size: Size) {
    let mut mask = 1 << (num_bits - 1);

    let (symbol_size, quiet_offset) = match size {
        Size::Micro(i) => (9+2*i as i16, 2),
        Size::Standard(i) => (17+4*i as i16, 4)
    };

    for &(xoff, yoff) in coords {
        let color = if (mask & bits) == 0 { BIT_WHITE } else { BIT_BLACK };
        let x = quiet_offset + if xoff < 0 { xoff + symbol_size } else { xoff };
        let y = quiet_offset + if yoff < 0 { yoff + symbol_size } else { yoff };
        symbol[(x as u32, y as u32)] = color;
        mask >>= 1;
    }
}


/// compute the 15bits long format bits "format info" specifier, which contains
/// information about the used mask and ECCLevel
fn compute_format_info_bits(size: Size, ecl: ECCLevel, mask_pattern: u8) -> u16 {
    match size {
        Size::Micro(i) => {
            let data_bits = match (i, ecl) {
                (1, ECCLevel::L) => 0b00000,
                (2, ECCLevel::L) => 0b00100,
                (2, ECCLevel::M) => 0b01000,
                (3, ECCLevel::L) => 0b01100,
                (3, ECCLevel::M) => 0b10000,
                (4, ECCLevel::L) => 0b10100,
                (4, ECCLevel::M) => 0b11000,
                (4, ECCLevel::Q) => 0b11100,
                _ => panic!("Invalid combination of size and ECC level")
            } as usize | (mask_pattern as usize);
            FORMAT_INFOS_MICRO_QR[data_bits]
        },
        Size::Standard(_) => {
            let data_bits = match ecl {
                ECCLevel::L => 0b01000,
                ECCLevel::M => 0b00000,
                ECCLevel::Q => 0b11000,
                ECCLevel::H => 0b10000
            } as usize | (mask_pattern as usize);
            FORMAT_INFOS_QR[data_bits]
        }
    }
}

/// Compute and write format bits into symbol
pub fn insert_format_info(symbol: &mut image::GrayImage, size: Size, ecl: ECCLevel, mask_pattern: u8) {
    let format_bits = compute_format_info_bits(size, ecl, mask_pattern);

    match size {
        Size::Micro(_) => {
            insert_bits_at(symbol, format_bits as u32, 15, &FORMAT_INFO_COORDS_MICRO_QR, size);
        },
        Size::Standard(i) => {
            insert_bits_at(symbol, format_bits as u32, 15, &FORMAT_INFO_COORDS_QR_MAIN, size);
            insert_bits_at(symbol, format_bits as u32, 15, &FORMAT_INFO_COORDS_QR_SIDE, size);
            symbol[(12, 13+4*i as u32)] = BIT_BLACK;
        }
    }
}

/// Compute and insert version info bits into symbol
/// Only does something for >= version 7 symbols.
pub fn insert_version_info(symbol: &mut image::GrayImage, size: Size) {
    if let Size::Standard(i) = size {
        if i >= 7 {
            let version_bits = VERSION_INFOS[(i-7) as usize];

            insert_bits_at(symbol, version_bits, 18, &VERSION_INFO_COORDS_BL, size);
            insert_bits_at(symbol, version_bits, 18, &VERSION_INFO_COORDS_TR, size);
        }
    }
}


//-------------------------------------------------------------------
// TESTS
//-------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_standard() {
        create_qr_canvas(Size::Standard(7)).save("./tmp_standard.png").unwrap();
    }

    #[test]
    fn test_micro() {
        create_qr_canvas(Size::Micro(3)).save("./tmp_micro.png").unwrap();
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
