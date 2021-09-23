

use super::*;

use image;


/// Return the masking function for a given size. Pattern index is from 0..8 for standard
/// sizes and in 0..4 for micro symbols. Returns a function that returns for the given index
/// i,j (i row coord, j column coord, including the quiet region!)
/// whether it meets the masking condition.
fn get_masking_function(pattern_index: u8, size: Size) -> Box<dyn Fn(i32, i32) -> bool> {
    match size {
        Size::Micro(_) => {
            match pattern_index {
                0b00 => Box::new(| i, _j| { (i-2) % 2 == 0 }),
                0b01 => Box::new(| i,  j| { ((i-2) / 2 + (j-2) / 3) % 2 == 0 }),
                0b10 => Box::new(| i,  j| { (((i-2)*(j-2)) % 2 + ((i-2)*(j-2)) % 3) % 2 == 0 }),
                0b11 => Box::new(| i,  j| { (((i-2)+(j-2)) % 2 + ((i-2)*(j-2)) % 3) % 2 == 0 }),
                _ => panic!("Wrong pattern index given!")
            }
        },
        Size::Standard(_) => {
            match pattern_index {
                0b000 => Box::new(| i,  j| { ((i-4) + (j-4)) % 2 == 0 }),
                0b001 => Box::new(| i, _j| { (i-4) % 2 == 0 }),
                0b010 => Box::new(|_i,  j| { (j-4) % 3 == 0 }),
                0b011 => Box::new(| i,  j| { ((i-4) + (j-4)) % 3 == 0 }),
                0b100 => Box::new(| i,  j| { ((i-4) / 2 + (j-4) / 3) % 2 == 0 }),
                0b101 => Box::new(| i,  j| { ((i-4)*(j-4)) % 2 + ((i-4)*(j-4)) % 3 == 0 }),
                0b110 => Box::new(| i,  j| { (((i-4)*(j-4)) % 2 + ((i-4)*(j-4)) % 3) % 2 == 0 }),
                0b111 => Box::new(| i,  j| { (((i-4)+(j-4)) % 2 + ((i-4)*(j-4)) % 3) % 2 == 0 }),
                _ => panic!("Wrong pattern index given!")
            }
        }
    }
}


/// apply mask to given symbol's encoding region. The second parameter is the canvas
/// without content, to mark the encoding region inside the symbol.
pub fn apply_mask(symbol: &mut image::GrayImage, pattern: u8, size: Size, marker: &image::GrayImage) {
    // get masking function
    let pattern_func = get_masking_function(pattern, size);

    // iterate over symbol
    for (x, y, pix) in symbol.enumerate_pixels_mut() {
        // check if we are in the encoding region. Ignore all other pixels
        if marker[(x, y)] == MARKER_ENCODING_REGION {
            // retrieve the mask bit. Flip the bit if the mask bit
            // is 1, leave it as is otherwise. This is equivalent with
            // a XOR between the mask and value bits.
            if pattern_func(y as i32, x as i32) {
                *pix = if *pix == BIT_BLACK { BIT_WHITE } else { BIT_BLACK };
            }
        }
    }
}

/// Compute penalty score for symbol with mask applied for standard size QR codes.
/// There is an extra function to do this for micro symbols, because it works differently for those.
const PENALTY_N1: u32 = 3;
const PENALTY_N2: u32 = 3;
const PENALTY_N3: u32 = 40;
const PENALTY_N4: u32 = 10;

fn compute_mask_penalty_score_standard(masked_symbol: &image::GrayImage) -> u32 {
    // NOte: all iterations exclude the quiet region, which accounts for the offset of 4.
    // FIRST feature: adjacent modules of same color or size in symbol.
    let mut score: u32 = 0;
    {
        // search all the rows for adjacent blocks of same-color modules.
        for y in 4..(masked_symbol.height()-4) {
            let mut last_color = BIT_WHITE;
            let mut current_run = 1;        // number of current adjacent modules found.
            for x in 4..(masked_symbol.width()-4) {
                if masked_symbol[(x, y)] == last_color {
                    // counts against current run
                    current_run += 1;
                } else {
                    // run resets. check for penalties
                    if current_run >= 5 {
                        score += (current_run - 5) + PENALTY_N1;
                    }
                    current_run = 1;
                    last_color = masked_symbol[(x, y)];
                }
            }
            // check for final penalty, if the last block is big enough
            if current_run >= 5 {
                score += (current_run - 5) + PENALTY_N1;
            }
        }

        // now the same for columns. This is almost the same, but note that the order of
        // iteration changed.
        for x in 4..(masked_symbol.width()-4) {
            let mut last_color = BIT_WHITE;
            let mut current_run = 1;        // number of current adjacent modules found.
            for y in 4..(masked_symbol.height()-4) {
                if masked_symbol[(x, y)] == last_color {
                    // counts against current run
                    current_run += 1;
                } else {
                    // run resets. check for penalties
                    if current_run >= 5 {
                        score += (current_run - 5) + PENALTY_N1;
                    }
                    current_run = 1;
                    last_color = masked_symbol[(x, y)];
                }
            }
            // check for final penalty, if the last block is big enough
            if current_run >= 5 {
                score += (current_run - 5) + PENALTY_N1;
            }
        }
    }

    // SECOND FEATURE: penalties for 2x2 module blocks of same color
    {
        for y in 4..(masked_symbol.height()-5) {
            for x in 4..(masked_symbol.width()-5) {
                if masked_symbol[(x, y)] == masked_symbol[(x+1, y)] &&
                   masked_symbol[(x, y)] == masked_symbol[(x, y+1)] &&
                   masked_symbol[(x, y)] == masked_symbol[(x+1, y+1)] {
                    // add penalty
                    score += PENALTY_N2;
                }
            }
        }
    }

    // THIRD FEATURE: 1011101 patterns with 4 zeros before or after it
    {
        const PATTERN: [image::Luma<u8>; 7] = [BIT_BLACK, BIT_WHITE, BIT_BLACK, BIT_BLACK, BIT_BLACK, BIT_WHITE, BIT_BLACK];

        for y in 4..(masked_symbol.height()-4) {
            for x in 4..(masked_symbol.width()-10) {
                // check if pattern exists in  (x:x+7, y)
                if (x..(x+7)).map(|x_cur| masked_symbol.get_pixel(x_cur, y)).ne(PATTERN.iter()) {
                    // is different, so go on
                    continue;
                }

                // check for four white spaces
                let is_black = |x_cur| 0 <= x_cur && x_cur < masked_symbol.width() && *masked_symbol.get_pixel(x_cur, y) == BIT_BLACK;
                if !((x - 4)..x).any(&is_black) || !((x+7)..(x+11)).any(&is_black) {
                    score += PENALTY_N3;
                }
            }
        }

        // subtract 9*N3 for the 9 occurrences of the pattern in the finders + quiet space
        score -= 9 * PENALTY_N3;

        // same for columns
        for x in 4..(masked_symbol.width()-4) {
            for y in 4..(masked_symbol.height()-10) {
                // check if pattern exists in  (x, y:y+7)
                if (y..(y+7)).map(|y_cur| masked_symbol.get_pixel(x, y_cur)).ne(PATTERN.iter()) {
                    // is different, so go on
                    continue;
                }

                // check for four white spaces
                let is_black = |y_cur| 0 <= y_cur && y_cur < masked_symbol.width() && *masked_symbol.get_pixel(x, y_cur) == BIT_BLACK;
                if !((y - 4)..y).any(&is_black) || !((y+7)..(y+11)).any(&is_black) {
                    score += PENALTY_N3;
                }
            }
        }

        // subtract 9*N3 for the 9 occurrences of the pattern in the finders + quiet space
        score -= 9 * PENALTY_N3;
    }

    // FOURTH FEATURE: dark/light ratio balance
    {
        // count dark modules
        let num_dark_modules = masked_symbol.pixels().filter(|&px| *px == BIT_BLACK).count();
        let ratio = num_dark_modules as f64 / ((masked_symbol.width()-8) * (masked_symbol.height()-8)) as f64;

        let ratio_diff = (0.5 - ratio).abs();
        let step = (ratio_diff * 20.0).floor() as u32; // *20 is actually / 0.05;

        // step is now the number of full-5%-steps by which ratio deviates from 50%.
        score += PENALTY_N4 * step;
    }

    score
}

/// compute the mask score for a masked micro QR symbol
fn compute_mask_score_micro(masked_symbol: &image::GrayImage) -> u32 {
    // count number of black modules in right and lower edges of symbol
    let sum1 = (3..(masked_symbol.height()-2))
        .map(|y_cur| masked_symbol.get_pixel(masked_symbol.width()-2, y_cur))
        .filter(|&px| *px == BIT_BLACK)
        .count() as u32;

    let sum2 = (3..(masked_symbol.width()-2))
        .map(|x_cur| masked_symbol.get_pixel(x_cur, masked_symbol.height()-2))
        .filter(|&px| *px == BIT_BLACK)
        .count() as u32;

    if sum1 <= sum2 {
        sum1 * 16 + sum2
    } else {
        sum2 * 16 + sum1
    }
}

/// Compute best mask and apply it.
/// Will evaluate all available masks for the given symbol, apply the best mask and return
/// the code of that mask and resulting masked symbol.
pub fn apply_best_mask(unmasked_symbol: &image::GrayImage, size: Size) -> (u8, image::GrayImage) {
    let canvas = create_qr_canvas(size);
    match size {
        Size::Micro(_) => {
            let (best_index, masked_symbol, _) = {
                (0..4)
                .map( | index| {
                    let mut masked_copy = unmasked_symbol.clone();
                    apply_mask( & mut masked_copy, index, size, & canvas);
                    let score = compute_mask_score_micro(&masked_copy);
                    (index, masked_copy, score)
                })
                .max_by_key( | data | data.2)  // mask with highest score is best
                .unwrap()
            };
            (best_index, masked_symbol)
        },
        Size::Standard(_) => {
            let (best_index, masked_symbol, _) = {
                (0..8)
                .map( | index| {
                    let mut masked_copy = unmasked_symbol.clone();
                    apply_mask( & mut masked_copy, index, size, & canvas);
                    let score = compute_mask_penalty_score_standard(&masked_copy);
                    (index, masked_copy, score)
                })
                .min_by_key( | data | data.2)  // mask with lowest score is best
                .unwrap()
            };
            (best_index, masked_symbol)
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
        for (x, y, pix) in canvas.enumerate_pixels_mut() {
            if *pix == MARKER_ENCODING_REGION {
                *pix = if pattern(y as i32, x as i32) { BIT_BLACK } else { BIT_WHITE };
            }
        }

        canvas
    }

    #[test]
    fn test_masks_micro() {
        for i in 0..4 {
            create_masked_canvas(Size::Micro(4), i as u8).save(format!("./mask_pattern_M1_{}.png", i)).unwrap();
        }
    }

    #[test]
    fn test_masks_standard() {
        for i in 0..8 {
            create_masked_canvas(Size::Standard(1), i as u8).save(format!("./mask_pattern_1_{}.png", i)).unwrap();
        }
    }
}
