use crate::config::{EncodingMode, Size, Encoding};

use std::convert::TryInto;

use bitstream_io::{BitWriter, BitWrite, BigEndian};

type QrBitWriter<'a> = BitWriter<&'a mut Vec<u8>, BigEndian>;



fn write_mode_indicator(stream: &mut QrBitWriter, size: Size, ec: Encoding) {
    if let Size::Micro(i) = size {
        if i == 1 {
            // no mode indicator for M1 tags
            return;
        } else if i == 2 {
            // one bit: 0 => Numeric, 1 => Alphanumeric
            stream.write(1, match ec {
                Encoding::Numeric => 0,
                Encoding::Alphanumeric => 1,
                _ => panic!("Invalid encoding mode for chosen size!")
            }).unwrap();
        } else if i == 3 {
            // two bits
            stream.write(2, match ec {
                Encoding::Numeric => 0b00,
                Encoding::Alphanumeric => 0b01,
                Encoding::Bytes => 0b10,
                Encoding::Kanji => 0b11
            }).unwrap();
        } else if i == 4 {
            // three bits
            stream.write(3, match ec {
                Encoding::Numeric => 0b000,
                Encoding::Alphanumeric => 0b001,
                Encoding::Bytes => 0b010,
                Encoding::Kanji => 0b011
            }).unwrap();
        }
    } else if let Size::Standard(_) = size {
        stream.write(4, match ec {
            Encoding::Numeric => 0b0001,
            Encoding::Alphanumeric => 0b0010,
            Encoding::Bytes => 0b0100,
            Encoding::Kanji => 0b1000
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

/// Write a terminator bit sequence to the stream. This is done if
/// the message+terminator does not reach the capacity of the specified
/// QR symbol size.
pub fn write_terminator(stream: &mut QrBitWriter, size: Size) {
    // write a specified number of zeroes, depending on the code model
    stream.write(match size {
        Size::Micro(1) => 3,
        Size::Micro(2) => 5,
        Size::Micro(3) => 7,
        Size::Micro(4) => 9,
        Size::Standard(_) => 4,
        _ => panic!("Invalid size given")
    }, 0).unwrap();
}

/// Write an ECI header to the bitstream, which changes the interpretation
/// of the following encoded message, until another ECI header is encountered.
///
/// assignment is a decimal 6-digit number between 000000 and 999999 specifying
/// the encoding (as defined by the AIM ECI specification).
///
/// The ECI header can be omitted completely; in that case, the default
/// interpretation is Shift JIS X 0208 for "kanji" mode and ISO/IEC 8859-1
/// for the other three modes.
pub fn write_eci_header(stream: &mut QrBitWriter, assignment: u32) {
    // write ECI mode indicator
    stream.write(4, 0b0111).unwrap();
    // depending on value of assignment, encode it as either 1, 2 or 3
    // bytes
    if assignment < 128 {
        // encode as 0bbbbbbb
        stream.write(1, 0).unwrap();
        stream.write(7, assignment).unwrap();
    } else if assignment >= 128 && assignment < 16384 {
        // encode as 10bbbbbb bbbbbbbb
        stream.write(2, 0b10).unwrap();
        stream.write(14, assignment).unwrap();
    } else /* assigment >= 16384 && assigment < 1000000 */ {
        // encode as 110bbbbb bbbbbbbb bbbbbbbb
        stream.write(3, 0b110).unwrap();
        stream.write(21, assignment).unwrap();
    }
}

fn encode_numeric_data(stream: &mut QrBitWriter, input: &[u8]) {
    // iterate over input; group into
    // three digits and treat them as a decimal number between 0 and 999,
    // encode that number in 10 binary digits.
    let mut i = 0;         // 0-index of current digit in triplet
    let mut cur_code: u32 = 0;  // current value of triplet
    for &l in input {
        assert!(l >= 0x30 || l <= 0x39);    // ASCII codes for digits 0 to 9
        let digit = l - 0x30;
        cur_code = cur_code * 10 + digit as u32;
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
    let mut cur_code: u32 = 0;  // current value of triplet
    for &l in input {
        cur_code = cur_code * 45 + map_alphanumeric(l) as u32;
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

/// Write a given sequence of ISO/IEC 8859-1 or Shift JIS X 0208 encoded bytes
/// to a bitstream (default ECI). For simple QR codes this is the entire coded message.
///
/// Segments can be chained to encode parts of the message in different modes.
///
/// To use non-default ECIs, write the ECI header to the stream first, then call this
/// function to write data in any of the four supported encoding modes. The ECI changes the
/// interpretation of the encoded data. In most cases you will want to use the "bytes" encoding
/// there. See
pub fn encode_data_segment(stream: &mut QrBitWriter, input: &[u8], ec: Encoding, size: Size) {
    write_mode_indicator(stream, size, ec);
    match ec {
        Encoding::Numeric => {
            write_charcount_indicator(stream, input.len() as u32, size, ec);
            encode_numeric_data(stream, input);
        },
        Encoding::Alphanumeric => {
            write_charcount_indicator(stream, input.len() as u32, size, ec);
            encode_alphanumeric_data(stream, input);
        },
        Encoding::Bytes => {
            write_charcount_indicator(stream, input.len() as u32, size, ec);
            encode_byte_data(stream, input);
        },
        Encoding::Kanji => {
            write_charcount_indicator(stream, input.len() as u32 / 2, size, ec);
            encode_kanji_data(stream, input);
        }
    }
}

// TODO: structured append (see Chapter 8, page 67)

// TODO: FCN1 format (see Chapter 7.4.8, page 38)


//-------------------------------------------------------------------
// TESTS
//-------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_example_1() {
        let mut data: Vec<u8> = Vec::new();
        let (bits, value) = {
            let mut stream = QrBitWriter::new(&mut data);
            encode_data_segment(&mut stream, b"01234567", Encoding::Numeric, Size::Standard(1));
            stream.into_unwritten()
        };
        assert_eq!(data, [0b0001_0000, 0b0010_0000, 0b0000_1100, 0b0101_0110, 0b0110_0001]);
        assert_eq!(bits, 1);  // one bit left over
        assert_eq!(value, 1); // that bit is a 1
    }

    #[test]
    fn test_numeric_example_2() {
        let mut data: Vec<u8> = Vec::new();
        let (bits, value) = {
            let mut stream = QrBitWriter::new(&mut data);
            encode_data_segment(&mut stream, b"0123456789012345", Encoding::Numeric, Size::Micro(3));
            stream.into_unwritten()
        };
        assert_eq!(data, [0b0010_0000, 0b0000_0110, 0b0010_1011, 0b0011_0101, 0b0011_0111,
                          0b0000_1010, 0b0111_0101]);
        assert_eq!(bits, 5);  // five bits left over
        assert_eq!(value, 5); // value of those is 00101, so 5
    }

    #[test]
    fn test_alphanumeric_example() {
        let mut data: Vec<u8> = Vec::new();
        let (bits, value) = {
            let mut stream = QrBitWriter::new(&mut data);
            encode_data_segment(&mut stream, b"AC-42", Encoding::Alphanumeric, Size::Standard(1));
            stream.into_unwritten()
        };
        assert_eq!(data, [0b0010_0000, 0b0010_1001, 0b1100_1110, 0b1110_0111, 0b0010_0001]);
        assert_eq!(bits, 1);  // one bit left over
        assert_eq!(value, 0); // value of that bit is zero
    }

    #[test]
    fn test_kanji_example() {
        let mut data: Vec<u8> = Vec::new();
        let (bits, value) = {
            let mut stream = QrBitWriter::new(&mut data);
            encode_data_segment(&mut stream, &[0x93, 0x5F, 0xE4, 0xAA], Encoding::Kanji, Size::Standard(1));
            stream.into_unwritten()
        };
        assert_eq!(data, [0b1000_0000, 0b0010_0110, 0b1100_1111, 0b1110_1010]);
        assert_eq!(bits, 6);  // six bits left over
        assert_eq!(value, 0b101010); // those bits are 0b101010
    }
}