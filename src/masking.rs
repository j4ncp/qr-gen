
use crate::config::Size;
use crate::serialization::*;

use image;


/// Return the masking function for a given size. Pattern index is from 0..8 for standard
/// sizes and in 0..4 for micro symbols. Returns a function that returns for the given index
/// i,j (i row coord, j column coord, including the quiet region!)
/// whether it meets the masking condition.
fn get_masking_function(pattern_index: u8, size: Size) -> Box<dyn Fn(i32, i32) -> bool> {
    match size {
        Size::Micro(i) => {
            match pattern_index {
                0b00 => Box::new(|i, j| { (i-2) % 2 == 0 }),
                0b01 => Box::new(|i, j| { ((i-2) / 2 + (j-2) / 3) % 2 == 0 }),
                0b10 => Box::new(|i, j| { (((i-2)*(j-2)) % 2 + ((i-2)*(j-2)) % 3) % 2 == 0 }),
                0b11 => Box::new(|i, j| { (((i-2)+(j-2)) % 2 + ((i-2)*(j-2)) % 3) % 2 == 0 }),
                _ => panic!("Wrong pattern index given!")
            }
        },
        Size::Standard(i) => {
            match pattern_index {
                0b000 => Box::new(|i, j| { ((i-4) + (j-4)) % 2 == 0 }),
                0b001 => Box::new(|i, j| { (i-4) % 2 == 0 }),
                0b010 => Box::new(|i, j| { (j-4) % 3 == 0 }),
                0b011 => Box::new(|i, j| { ((i-4) + (j-4)) % 3 == 0 }),
                0b100 => Box::new(|i, j| { ((i-4) / 2 + (j-4) / 3) % 2 == 0 }),
                0b101 => Box::new(|i, j| { ((i-4)*(j-4)) % 2 + ((i-4)*(j-4)) % 3 == 0 }),
                0b110 => Box::new(|i, j| { (((i-4)*(j-4)) % 2 + ((i-4)*(j-4)) % 3) % 2 == 0 }),
                0b111 => Box::new(|i, j| { (((i-4)+(j-4)) % 2 + ((i-4)*(j-4)) % 3) % 2 == 0 }),
                _ => panic!("Wrong pattern index given!")
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn create_masked_canvas(size: Size, pattern_index: u8) -> image::GrayImage {
        // create canvas
        let mut canvas = create_qr_canvas(size);

        // retrieve pattern index
        let pattern = get_masking_function(pattern_index, size);

        // iterate over entire image and create mask in the encoding region
        for (x, y, mut pix) in canvas.enumerate_pixels_mut() {
            if *pix == MARKER_ENCODING_REGION {
                *pix = if pattern(y as i32, x as i32) { BIT_BLACK } else { BIT_WHITE };
            }
        }

        canvas
    }

    #[test]
    fn test_masks_micro() {
        for i in 0..4 {
            create_masked_canvas(Size::Micro(4), i as u8).save(format!("./mask_pattern_M1_{}.png", i));
        }
    }

    #[test]
    fn test_masks_standard() {
        for i in 0..8 {
            create_masked_canvas(Size::Standard(1), i as u8).save(format!("./mask_pattern_1_{}.png", i));
        }
    }
}
