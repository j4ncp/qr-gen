use crate::config::{EncodingMode, Size, Encoding};

use std::convert::TryInto;

use bitstream_io::{BitWriter, BitWrite, BigEndian};

type QrBitWriter = BitWriter<Vec<u8>, BigEndian>;



fn write_mode_indicator(stream: &mut QrBitWriter, size: Size, ecm: EncodingMode) {
    if let Size::Micro(i) = size {
        if i == 1 {
            // no mode indicator for M1 tags
            return;
        } else if i == 2 {
            // one bit: 0 => Numeric, 1 => Alphanumeric
            stream.write(1, match ecm {
                EncodingMode::Numeric => 0,
                EncodingMode::Alphanumeric => 1,
                _ => panic!("Invalid encoding mode for chosen size!")
            }).unwrap();
        } else if i == 3 {
            // two bits
            stream.write(2, match ecm {
                EncodingMode::Numeric => 0b00,
                EncodingMode::Alphanumeric => 0b01,
                EncodingMode::Byte => 0b10,
                EncodingMode::Kanji => 0b11,
                _ => panic!("Invalid encoding mode for chosen size!")
            }).unwrap();
        } else if i == 4 {
            // three bits
            stream.write(3, match ecm {
                EncodingMode::Numeric => 0b000,
                EncodingMode::Alphanumeric => 0b001,
                EncodingMode::Byte => 0b010,
                EncodingMode::Kanji => 0b011,
                _ => panic!("Invalid encoding mode for chosen size!")
            }).unwrap();
        }
    } else if let Size::Standard(_) = size {
        stream.write(4, match ecm {
            EncodingMode::Numeric => 0b0001,
            EncodingMode::Alphanumeric => 0b0010,
            EncodingMode::Byte => 0b0100,
            EncodingMode::Kanji => 0b1000,
            EncodingMode::ECI => 0b0111,
            EncodingMode::StructuredAppend => 0b0011,
            EncodingMode::FNC1 => 0b0101    // TODO!
        }).unwrap();
    }
}

fn write_charcount_indicator(stream: &mut QrBitWriter, count: u32, size: Size, ec: Encoding) {
    let num_bits = match size {
        Size::Micro(1) => match ec {
            Encoding::Numeric => 3,
            _ => panic!("Invalid combination of encoding and size!")
        },
        Size::Micro(2) => match ec {
            Encoding::Numeric => 4,
            Encoding::Alphanumeric => 3,
            _ => panic!("Invalid combination of encoding and size!")
        },
        Size::Micro(3) => match ec {
            Encoding::Numeric => 5,
            Encoding::Alphanumeric => 4,
            Encoding::Bytes => 4,
            Encoding::Kanji => 3
        },
        Size::Micro(4) => match ec {
            Encoding::Numeric => 6,
            Encoding::Alphanumeric => 5,
            Encoding::Bytes => 5,
            Encoding::Kanji => 4
        },
        Size::Standard(i) => {
            if i >= 1 && i <= 9 {
                match ec {
                    Encoding::Numeric => 10,
                    Encoding::Alphanumeric => 9,
                    Encoding::Bytes => 8,
                    Encoding::Kanji => 8
                }
            } else if i >= 10 && i <= 26 {
                match ec {
                    Encoding::Numeric => 12,
                    Encoding::Alphanumeric => 11,
                    Encoding::Bytes => 16,
                    Encoding::Kanji => 10
                }
            } else /* i >= 27 && i <= 40 */ {
                match ec {
                    Encoding::Numeric => 14,
                    Encoding::Alphanumeric => 13,
                    Encoding::Bytes => 16,
                    Encoding::Kanji => 12
                }
            }
        },
        _ => panic!("Invalid index given for micro Qr code")
    };
    stream.write(num_bits, count).unwrap();
}


fn encode_numeric_data(stream: &mut QrBitWriter, input: &[u8]) {
    // iterate over input; group into
    // three digits and treat them as a decimal number between 0 and 999,
    // encode that number in 10 binary digits.
    let mut i = 0;         // 0-index of current digit in triplet
    let mut cur_code = 0;  // current value of triplet
    for &l in input {
        assert!(l >= 0x30 || l <= 0x39);    // ASCII codes for digits 0 to 9
        let digit = l - 0x30;
        cur_code = cur_code * 10 + digit;
        i += 1;
        if i == 3 {
            // got triplet. write the code to the bitstream and reset state
            // for next triplet
            stream.write(10, cur_code).unwrap();
            i = 0;
            cur_code = 0;
        }
    }
    // potentially encode last incomplete triplet
    if i == 1 {
        stream.write(4, cur_code).unwrap();
    } else if i == 2 {
        stream.write(7, cur_code).unwrap();
    }
}


fn map_alphanumeric(in_char: u8) -> u8 {
    match in_char {
        0x30..=0x39 => in_char - 0x30,  // a digit in [0-9] maps to that value
        0x41..=0x5A => in_char - 0x37,  // capital letters in [A-Z] map to the next 26 values
        0x20 => 36, // space
        0x24 => 37, // dollar $
        0x25 => 38, // percent %
        0x2A => 39, // asterisk *
        0x2B => 40, // plus +
        0x2D => 41, // minus -
        0x2E => 42, // period .
        0x2F => 43, // slash /
        0x3A => 44, // colon :
        _ => panic!("Invalid char for alphanumeric mode!")
    }
}

fn encode_alphanumeric_data(stream: &mut QrBitWriter, input: &[u8]) {
    // iterate over input; group into
    // two chars and multiply the first by 45, sum with second one.
    // encode that number in 11 binary digits.
    let mut i = 0;         // 0-index of current digit in triplet
    let mut cur_code = 0;  // current value of triplet
    for &l in input {
        cur_code = cur_code * 45 + map_alphanumeric(l);
        i += 1;
        if i == 2 {
            // got pair. write the code to the bitstream and reset state
            stream.write(11, cur_code).unwrap();
            i = 0;
            cur_code = 0;
        }
    }
    // potentially write remaining char as 6bit code
    if i == 1 {
        stream.write(6, cur_code).unwrap();
    }
}

fn encode_byte_data(stream: &mut QrBitWriter, input: &[u8]) {
    // assume byte data is already ISO8859-1 encoded,
    // so just write those as bits
    for &l in input {
        stream.write(8, l).unwrap();
    }
}

fn encode_kanji_data(stream: &mut QrBitWriter, input: &[u8]) {
    // we assume input is encoded in Shift JIS (see JIS X 0208)
    // using two bytes per character. Every character is compacted
    // into a 13bit codeword and written to the output.
    assert!(input.len() % 2 == 0);
    for p in input.chunks(2) {
        let pair: &[u8;2] = p.try_into().unwrap();
        let number: u16 = pair[0] as u16 * 0x100 + pair[1] as u16;
        if number >= 0x8140 && number <= 0x9FFC {
            let number = number - 0x8140;
            let code = (number >> 8) * 0xC0 + (number & 0xFF);
            stream.write(13, code).unwrap();
        } else if number >= 0xE040 && number <= 0xEBBF {
            let number = number - 0xC140;
            let code = (number >> 8) * 0xC0 + (number & 0xFF);
            stream.write(13, code).unwrap();
        }
    }
}


fn encode_data_segment(stream: &mut QrBitWriter, input: &[u8], ec: Encoding, size: Size) {
    // TODO
}

//-------------------------------------------------------------------
// TESTS
//-------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImageView, ImageResult};

    # [test]
    fn test_bla() {

    }
}