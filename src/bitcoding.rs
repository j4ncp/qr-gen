use crate::config::{Size, Encoding};

use crate::tables::lookup_capacity;

use std::convert::TryInto;
use std::cmp;

use bitstream_io::{BitWriter, BitRecorder, BitWrite, BigEndian};
use crate::ECCLevel;

pub type QrBitRecorder = BitRecorder<u32, BigEndian>;
pub type QrBitWriter<'a> = BitWriter<&'a mut Vec<u8>, BigEndian>;



fn write_mode_indicator(stream: &mut QrBitRecorder, size: Size, ec: Encoding) {
    match size {
        Size::Micro(1) => {},
        Size::Micro(2) => {
            // one bit: 0 => Numeric, 1 => Alphanumeric
            stream.write(1, match ec {
                Encoding::Numeric => 0,
                Encoding::Alphanumeric => 1,
                _ => panic!("Invalid encoding mode for chosen size!")
            }).unwrap();
        },
        Size::Micro(3) => {
            // two bits
            stream.write(2, match ec {
                Encoding::Numeric => 0b00,
                Encoding::Alphanumeric => 0b01,
                Encoding::Bytes => 0b10,
                Encoding::Kanji => 0b11
            }).unwrap();
        },
        Size::Micro(4) => {
            // three bits
            stream.write(3, match ec {
                Encoding::Numeric => 0b000,
                Encoding::Alphanumeric => 0b001,
                Encoding::Bytes => 0b010,
                Encoding::Kanji => 0b011
            }).unwrap();
        },
        Size::Standard(_) => {
            stream.write(4, match ec {
                Encoding::Numeric => 0b0001,
                Encoding::Alphanumeric => 0b0010,
                Encoding::Bytes => 0b0100,
                Encoding::Kanji => 0b1000
            }).unwrap();
        },
        _ => panic!("Invalid size given")
    }
}

/// write character count indicator to the bitstream. The interesting part is how many bits
/// are used for this, which is given by a helper member of Encoding
fn write_charcount_indicator(stream: &mut QrBitRecorder, count: u32, size: Size, ec: Encoding) {
    stream.write(ec.num_char_count_bits(size) as u32, count).unwrap();
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
pub fn write_eci_header(stream: &mut QrBitRecorder, assignment: u32) {
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

fn encode_numeric_data(stream: &mut QrBitRecorder, input: &[u8]) {
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

fn encode_alphanumeric_data(stream: &mut QrBitRecorder, input: &[u8]) {
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

fn encode_byte_data(stream: &mut QrBitRecorder, input: &[u8]) {
    // assume byte data is already ISO8859-1 encoded,
    // so just write those as bits
    for &l in input {
        stream.write(8, l).unwrap();
    }
}

fn encode_kanji_data(stream: &mut QrBitRecorder, input: &[u8]) {
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
pub fn encode_data_segment(stream: &mut QrBitRecorder, input: &[u8], ec: Encoding, size: Size) {
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


/// takes a recorded sequence of mode segments, maybe interspersed with
/// ECI headers and maybe containing more complex data sequences and finalizes it,
/// returning a sequence of codewords as a byte array. The finalization entails potentially
/// appending a terminator sequence, adding zero bits to byte-align the sequence and potentially
/// adding padding bytes to fill the chosen symbol's capacity exactly.
pub fn finalize_bitstream(stream: &mut QrBitRecorder, size: Size, ecl: ECCLevel) -> Vec<u8> {
    let bit_capacity = lookup_capacity(size, ecl).data_bits;

    // append terminator bits. At most as many zeroes as specified, and at least as many
    // of those as can fit within the symbol capacity.
    {
        let bit_rawdatasize = stream.written();
        assert!(bit_rawdatasize <= bit_capacity, "Too many data bits for chosen symbol size {:?}!", size);

        let terminator_bits = cmp::min(bit_capacity - bit_rawdatasize, size.terminator_length() as u32);
        stream.write(terminator_bits, 0 as u32).unwrap();
    }

    // pad with zeroes to next full byte
    {
        let written = stream.written();
        let alignment = written % 8;

        // if we are already byte-aligned there is nothing to do.
        if alignment > 0 {
            let padding = 8 - alignment;

            // special case: last word in M1 and M3 symbols is only 4 bits
            if (size == Size::Micro(1) || size == Size::Micro(3)) &&
               written + 4 > bit_capacity {
                // if we are already into those 4 last bits, just pad those with zeroes completely
                stream.write(bit_capacity - written, 0 as u32).unwrap();
            } else {
                // simply add zero padding
                stream.write(padding, 0 as u32).unwrap();
            }
        }
    }

    // pad alternately with the two specified codewords 0b11101100 and 0b00010001
    // until capacity is filled.
    {
        let bits_left = bit_capacity - stream.written();
        let bytes_left = bits_left / 8;

        // Note: the integer division by 8 is correct in all cases, because:
        //      - for standard sizes the capacity is a multiple of 8 and also the stream contains
        //        a multiple of 8 bits. So bits_left is also multiple of 8.
        //      - for M1 and M3 sizes, the capacity is a multiple of 8 plus 4, while bits_left is
        //        either zero, four, or a multiple of 8 plus four. So in the first two cases bytes_left
        //        is zero, and in the third will return the remaining multiplicity of 8, which is correct.

        // pad bytes_left with special codewords
        const PAD_CODEWORDS: [u32; 2] = [0b11101100, 0b00010001];
        for i in 0..bytes_left {
            let padding = PAD_CODEWORDS[i as usize % 2];
            stream.write(8, padding).unwrap();
        }
    }

    // now truly the only thing left could be to set the last four missing bits in
    // a M1 or M3 symbol to zero.
    {
        let bits_left = bit_capacity - stream.written();

        if (size == Size::Micro(1) || size == Size::Micro(3)) && bits_left > 0 {
            assert_eq!(bits_left, 4);
            stream.write(bits_left, 0 as u32).unwrap();
        } else {
            // otherwise no bits should be left, ever
            assert_eq!(bits_left, 0 as u32);
        }
    }

    assert_eq!(stream.written(), bit_capacity);

    // add four more zero bits in the case of M1 and M3 symbols, so we can return
    // as a vector of full bytes
    if size == Size::Micro(1) || size == Size::Micro(3) {
        stream.write(4, 0).unwrap();
    }

    // create a bit writer on a vector, play back all bits to it.
    let mut data_codewords: Vec<u8> = Vec::new();
    {
        let mut writer = QrBitWriter::new(&mut data_codewords);
        stream.playback(&mut writer).unwrap();
    }

    data_codewords
}


//-------------------------------------------------------------------
// TESTS
//-------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn to_bytes(rec: QrBitRecorder) -> (Vec<u8>, u32, u8) {
        let mut data: Vec<u8> = Vec::new();
        let (bits, value) = {
            let mut writer = QrBitWriter::new(&mut data);
            rec.playback(&mut writer).unwrap();
            writer.into_unwritten()
        };
        (data, bits, value)
    }

    #[test]
    fn test_numeric_example_1() {
        let mut recorder = QrBitRecorder::new();
        encode_data_segment(&mut recorder, b"01234567", Encoding::Numeric, Size::Standard(1));
        let (data, bits, value) = to_bytes(recorder);
        assert_eq!(data, [0b0001_0000, 0b0010_0000, 0b0000_1100, 0b0101_0110, 0b0110_0001]);
        assert_eq!(bits, 1);  // one bit left over
        assert_eq!(value, 1); // that bit is a 1
    }

    #[test]
    fn test_numeric_example_2() {
        let mut recorder = QrBitRecorder::new();
        encode_data_segment(&mut recorder, b"0123456789012345", Encoding::Numeric, Size::Micro(3));
        let (data, bits, value) = to_bytes(recorder);
        assert_eq!(data, [0b0010_0000, 0b0000_0110, 0b0010_1011, 0b0011_0101, 0b0011_0111,
                          0b0000_1010, 0b0111_0101]);
        assert_eq!(bits, 5);  // five bits left over
        assert_eq!(value, 5); // value of those is 00101, so 5
    }

    #[test]
    fn test_alphanumeric_example() {
        let mut recorder = QrBitRecorder::new();
        encode_data_segment(&mut recorder, b"AC-42", Encoding::Alphanumeric, Size::Standard(1));
        let (data, bits, value) = to_bytes(recorder);
        assert_eq!(data, [0b0010_0000, 0b0010_1001, 0b1100_1110, 0b1110_0111, 0b0010_0001]);
        assert_eq!(bits, 1);  // one bit left over
        assert_eq!(value, 0); // value of that bit is zero
    }

    #[test]
    fn test_kanji_example() {
        let mut recorder = QrBitRecorder::new();
        encode_data_segment(&mut recorder, &[0x93, 0x5F, 0xE4, 0xAA], Encoding::Kanji, Size::Standard(1));
        let (data, bits, value) = to_bytes(recorder);
        assert_eq!(data, [0b1000_0000, 0b0010_0110, 0b1100_1111, 0b1110_1010]);
        assert_eq!(bits, 6);  // six bits left over
        assert_eq!(value, 0b101010); // those bits are 0b101010
    }

    //TODO: tests for finalizing the bitstream
}