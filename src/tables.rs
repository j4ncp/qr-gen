/// Contains lookup tables and other computation functions that provide reference
/// data needed for encoding or decing a QR code, such as the capacity of each
/// code configuration in different encodings, etc.

use crate::config::{Encoding, ECCLevel, Size, SymbolConfig};

use std::collections::HashMap;
use std::ops::Index;

///////////////////////////////////////////////////////////////////////////////////////////////////

/// Definition of a block of data + ECC bytes
#[derive(Clone,Copy,Hash,Eq,PartialEq,Debug)]
pub struct BlockDef {
    pub num_blocks: u32,        // the number of occurrences of this block type
    pub codewords: u32,         // total number of codewords in this block
    pub data_codewords: u32,    // of those codewords, how many carry data (as opposed to ecc)
}

impl BlockDef {
    pub const fn new(num_blocks: u32,
                     words_total: u32,
                     words: u32) -> BlockDef {
        BlockDef {
            num_blocks: num_blocks,
            codewords: words_total,
            data_codewords: words
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone,Copy,Hash,Eq,PartialEq,Debug)]
pub struct SymbolCapacity {
    pub data_bits: u32,         // how many data bits?

    chars_numeric: u32,         // data capacity measured in digits when using numeric encoding
    chars_alphanum: u32,        // data capacity measured in alphanum chars when usign alphanum encoding
    chars_bytes: u32,           // data capacity measured in bytes when using bytes encoding
    chars_kanji: u32,           // data capacity measured in kanji chars when using kanji encoding

    pub block_def1: BlockDef,    // block definition for distributing the data over multiple blocks
    pub block_def2: BlockDef     // secondary block definition, null for some sizes
}

impl Index<Encoding> for SymbolCapacity {
    type Output = u32;

    fn index(&self, ty: Encoding) -> &Self::Output {
        match ty {
            Encoding::Numeric => &self.chars_numeric,
            Encoding::Alphanumeric => &self.chars_alphanum,
            Encoding::Bytes => &self.chars_bytes,
            Encoding::Kanji => &self.chars_kanji
        }
    }
}

impl SymbolCapacity {
    /// constructor for entries with two block types
    pub const fn new(bits: u32,
                     charsnum: u32,
                     charsalphanum: u32,
                     charsbytes: u32,
                     charskanji: u32,
                     num_blocks1: u32,
                     block_size1: u32,
                     block_data_words1: u32,
                     num_blocks2: u32,
                     block_size2: u32,
                     block_data_words2: u32,) -> SymbolCapacity {
        SymbolCapacity {
            data_bits: bits,
            chars_numeric: charsnum,
            chars_alphanum: charsalphanum,
            chars_bytes: charsbytes,
            chars_kanji: charskanji,
            block_def1: BlockDef::new(num_blocks1, block_size1, block_data_words1),
            block_def2: BlockDef::new(num_blocks2, block_size2, block_data_words2),
        }
    }

    /// compute and return the total number of codewords for this symbol (capacity)
    pub fn codewords(&self) -> u32 {
        &self.block_def1.num_blocks * &self.block_def1.codewords +
        &self.block_def2.num_blocks * &self.block_def2.codewords
    }

    /// compute and return the number of data codewords for this symbol (capacity)
    pub fn data_codewords(&self) -> u32 {
        &self.block_def1.num_blocks * &self.block_def1.data_codewords +
        &self.block_def2.num_blocks * &self.block_def2.data_codewords
    }

    /// compute and return the number of ecc codewords for this symbol
    pub fn ecc_words(&self) -> u32 {
        &self.codewords() - &self.data_codewords()
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////



macro_rules! define_capacity_table {
    {$(
        $size:expr,
        $ecc:expr,
        $bits:expr,
        $chars_n:expr,
        $chars_a:expr,
        $chars_b:expr,
        $chars_k:expr;
        $num_blocks1:expr, ($block_size1:expr, $data_size1:expr,);
        $num_blocks2:expr, ($block_size2:expr, $data_size2:expr,);
    )*} => {
        lazy_static! {
            pub static ref SYMBOL_CAPACITY_TABLE: HashMap<SymbolConfig, SymbolCapacity> = [
                $(
                    (SymbolConfig::new($size, $ecc), SymbolCapacity::new($bits, $chars_n, $chars_a, $chars_b, $chars_k, $num_blocks1, $block_size1, $data_size1, $num_blocks2, $block_size2, $data_size2)),
                )*
            ].iter().copied().collect();
        }
    }
}

// creates a copy of a combination of table 7 in ISO/IEC 18004:2015 section 7.4.10
// and table 9 in section 7.5.1
define_capacity_table!(
    Size::Micro(1), ECCLevel::L, 20, 5, 0, 0, 0;  1, (5, 3,); 0, (0, 0,);

    Size::Micro(2), ECCLevel::L, 40, 10, 6, 0, 0;  1, (10, 5,); 0, (0, 0,);
    Size::Micro(2), ECCLevel::M, 32,  8, 5, 0, 0;  1, (10, 4,); 0, (0, 0,);

    Size::Micro(3), ECCLevel::L, 84, 23, 14, 9, 6;  1, (17, 11,); 0, (0, 0,);
    Size::Micro(3), ECCLevel::M, 68, 18, 11, 7, 4;  1, (17,  9,); 0, (0, 0,);

    Size::Micro(4), ECCLevel::L, 128, 35, 21, 15, 9;  1, (24, 16,); 0, (0, 0,);
    Size::Micro(4), ECCLevel::M, 112, 30, 18, 13, 8;  1, (24, 14,); 0, (0, 0,);
    Size::Micro(4), ECCLevel::Q,  80, 21, 13,  9, 4;  1, (24, 10,); 0, (0, 0,);

    Size::Standard(1), ECCLevel::L, 152, 41, 25, 17, 10;  1, (26, 19,); 0, (0, 0,);
    Size::Standard(1), ECCLevel::M, 128, 34, 20, 14,  8;  1, (26, 16,); 0, (0, 0,);
    Size::Standard(1), ECCLevel::Q, 104, 27, 16, 11,  7;  1, (26, 13,); 0, (0, 0,);
    Size::Standard(1), ECCLevel::H,  72, 17, 10,  7,  4;  1, (26,  9,); 0, (0, 0,);

    Size::Standard(2), ECCLevel::L, 272, 77, 47, 32, 20;  1, (44, 34,); 0, (0, 0,);
    Size::Standard(2), ECCLevel::M, 224, 63, 38, 26, 16;  1, (44, 28,); 0, (0, 0,);
    Size::Standard(2), ECCLevel::Q, 176, 48, 29, 20, 12;  1, (44, 22,); 0, (0, 0,);
    Size::Standard(2), ECCLevel::H, 128, 34, 20, 14,  8;  1, (44, 16,); 0, (0, 0,);

    Size::Standard(3), ECCLevel::L, 440, 127, 77, 53, 32;  1, (70, 55,); 0, (0, 0,);
    Size::Standard(3), ECCLevel::M, 352, 101, 61, 42, 26;  1, (70, 44,); 0, (0, 0,);
    Size::Standard(3), ECCLevel::Q, 272,  77, 47, 32, 20;  2, (35, 17,); 0, (0, 0,);
    Size::Standard(3), ECCLevel::H, 208,  58, 35, 24, 15;  2, (35, 13,); 0, (0, 0,);

    Size::Standard(4), ECCLevel::L, 640, 187, 114, 78, 48;  1, (100, 80,); 0, (0, 0,);
    Size::Standard(4), ECCLevel::M, 512, 149,  90, 62, 38;  2, ( 50, 32,); 0, (0, 0,);
    Size::Standard(4), ECCLevel::Q, 384, 111,  67, 46, 28;  2, ( 50, 24,); 0, (0, 0,);
    Size::Standard(4), ECCLevel::H, 288,  82,  50, 34, 21;  4, ( 25,  9,); 0, (0, 0,);

    Size::Standard(5), ECCLevel::L, 864, 255, 154, 106, 65;  1, (134, 108,); 0, ( 0,  0,);
    Size::Standard(5), ECCLevel::M, 688, 202, 122,  84, 52;  2, ( 67,  43,); 0, ( 0,  0,);
    Size::Standard(5), ECCLevel::Q, 496, 144,  87,  60, 37;  2, ( 33,  15,); 2, (34, 16,);
    Size::Standard(5), ECCLevel::H, 368, 106,  64,  44, 27;  2, ( 33,  11,); 2, (34, 12,);

    Size::Standard(6), ECCLevel::L, 1088, 322, 195, 134, 82;  2, (86, 68,); 0, (0, 0,);
    Size::Standard(6), ECCLevel::M,  864, 255, 154, 106, 65;  4, (43, 27,); 0, (0, 0,);
    Size::Standard(6), ECCLevel::Q,  608, 178, 108,  74, 45;  4, (43, 19,); 0, (0, 0,);
    Size::Standard(6), ECCLevel::H,  480, 139,  84,  58, 36;  4, (43, 15,); 0, (0, 0,);

    Size::Standard(7), ECCLevel::L, 1248, 370, 224, 154, 95;  2, (98, 78,); 0, ( 0,  0,);
    Size::Standard(7), ECCLevel::M,  992, 293, 178, 122, 75;  4, (49, 31,); 0, ( 0,  0,);
    Size::Standard(7), ECCLevel::Q,  704, 207, 125,  86, 53;  2, (32, 14,); 4, (33, 15,);
    Size::Standard(7), ECCLevel::H,  528, 154,  93,  64, 39;  4, (39, 13,); 1, (40, 14,);

    Size::Standard(8), ECCLevel::L, 1552, 461, 279, 192, 118;  2, (121, 97,); 0, ( 0,  0,);
    Size::Standard(8), ECCLevel::M, 1232, 365, 221, 152,  93;  2, ( 60, 38,); 2, (61, 39,);
    Size::Standard(8), ECCLevel::Q,  880, 259, 157, 108,  66;  4, ( 40, 18,); 2, (41, 19,);
    Size::Standard(8), ECCLevel::H,  688, 202, 122,  84,  52;  4, ( 40, 14,); 2, (41, 15,);

    Size::Standard(9), ECCLevel::L, 1856, 552, 335, 230, 141;  2, (146, 116,); 0, ( 0,  0,);
    Size::Standard(9), ECCLevel::M, 1456, 432, 262, 180, 111;  3, ( 58,  36,); 2, (59, 37,);
    Size::Standard(9), ECCLevel::Q, 1056, 312, 189, 130,  80;  4, ( 36,  16,); 4, (37, 17,);
    Size::Standard(9), ECCLevel::H,  800, 235, 143,  98,  60;  4, ( 36,  12,); 4, (37, 13,);

    Size::Standard(10), ECCLevel::L, 2192, 652, 395, 271, 167;  2, (86, 68,); 2, (87, 69,);
    Size::Standard(10), ECCLevel::M, 1728, 513, 311, 213, 131;  4, (69, 43,); 1, (70, 44,);
    Size::Standard(10), ECCLevel::Q, 1232, 364, 221, 151,  93;  6, (43, 19,); 2, (44, 20,);
    Size::Standard(10), ECCLevel::H,  976, 288, 174, 119,  74;  6, (43, 15,); 2, (44, 16,);

    Size::Standard(11), ECCLevel::L, 2592, 772, 468, 321, 198;  4, (101, 81,); 0, ( 0,  0,);
    Size::Standard(11), ECCLevel::M, 2032, 604, 366, 251, 155;  1, ( 80, 50,); 4, (81, 51,);
    Size::Standard(11), ECCLevel::Q, 1440, 427, 259, 177, 109;  4, ( 50, 22,); 4, (51, 23,);
    Size::Standard(11), ECCLevel::H, 1120, 331, 200, 137,  85;  3, ( 36, 12,); 8, (37, 13,);

    Size::Standard(12), ECCLevel::L, 2960, 883, 535, 367, 226;  2, (116, 92,); 2, (117, 93,);
    Size::Standard(12), ECCLevel::M, 2320, 691, 419, 287, 177;  6, ( 58, 36,); 2, ( 59, 37,);
    Size::Standard(12), ECCLevel::Q, 1648, 489, 296, 203, 125;  4, ( 46, 20,); 6, ( 47, 21,);
    Size::Standard(12), ECCLevel::H, 1264, 374, 227, 155,  96;  7, ( 42, 14,); 4, ( 43, 15,);

    Size::Standard(13), ECCLevel::L, 3424, 1022, 619, 425, 262;  4, (133, 107,); 0, ( 0,  0,);
    Size::Standard(13), ECCLevel::M, 2672,  796, 483, 331, 204;  8, ( 59,  37,); 1, (60, 38,);
    Size::Standard(13), ECCLevel::Q, 1952,  580, 352, 241, 149;  8, ( 44,  20,); 4, (45, 21,);
    Size::Standard(13), ECCLevel::H, 1440,  427, 259, 177, 109; 12, ( 33,  11,); 4, (34, 12,);

    Size::Standard(14), ECCLevel::L, 3688, 1101, 667, 458, 282;  3, (145, 115,); 1, (146, 116,);
    Size::Standard(14), ECCLevel::M, 2920,  871, 528, 362, 223;  4, ( 64,  40,); 5, ( 65,  41,);
    Size::Standard(14), ECCLevel::Q, 2088,  621, 376, 258, 159; 11, ( 36,  16,); 5, ( 37,  17,);
    Size::Standard(14), ECCLevel::H, 1576,  468, 283, 194, 120; 11, ( 36,  12,); 5, ( 37,  13,);

    // TODO: extend with all symbol configurations
);

///////////////////////////////////////////////////////////////////////////////////////////////////

/// Convenience function that just indexes into the static table
pub fn lookup_capacity(s: Size, ecc: ECCLevel) -> SymbolCapacity {
    SYMBOL_CAPACITY_TABLE[&SymbolConfig::new(s, ecc)]
}

/// Returns the number of misdecode protection codewords p
pub fn get_p_for_symbol(s: Size, ecc: ECCLevel) -> u8 {
    // by definition in the standard ISO/IEC 18004:2015
    match s {
        Size::Micro(1) => 2,
        Size::Micro(2) => if ecc == ECCLevel::L { 3 } else { 2 },
        Size::Micro(3) => 2,
        Size::Micro(4) => if ecc == ECCLevel::L { 2 } else { 0 },
        Size::Standard(1) => match ecc {
            ECCLevel::L => 3,
            ECCLevel::M => 2,
            _ => 1
        },
        Size::Standard(2) => if ecc == ECCLevel::L {2} else {0},
        Size::Standard(3) => if ecc == ECCLevel::L {1} else {0},
        _ => 0
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table1() {
        assert_eq!(lookup_capacity(Size::Micro(3), ECCLevel::M).data_codewords(), 9);
        assert_eq!(lookup_capacity(Size::Micro(3), ECCLevel::M)[Encoding::Bytes], 7);
        assert_eq!(lookup_capacity(Size::Standard(1), ECCLevel::Q).ecc_words(), 13);
    }

    #[test]
    fn test_table2() {
        // check for all entries of a single size that all LMQH levels return the same number of
        // total codewords. If this fails, there is a transcription error in the table.
        // Do not check micro symbols as they are easy to check by hand.
        for i in 1..15 {
            let nl = lookup_capacity(Size::Standard(i), ECCLevel::L).codewords();
            let nm = lookup_capacity(Size::Standard(i), ECCLevel::M).codewords();
            let nq = lookup_capacity(Size::Standard(i), ECCLevel::Q).codewords();
            let nh = lookup_capacity(Size::Standard(i), ECCLevel::H).codewords();
            assert_eq!(nl, nm, "Error in total codewords number for symbol {}", i);
            assert_eq!(nl, nq, "Error in total codewords number for symbol {}", i);
            assert_eq!(nl, nh, "Error in total codewords number for symbol {}", i);
        }
    }

    #[test]
    fn test_table3() {
        // check that all entries with two block sizes have the following property:
        // the second block size (resp. data size) is exactly one unit larger than
        // the first block size (resp. data size).
        for (sym, cap) in SYMBOL_CAPACITY_TABLE.iter() {
            if cap.block_def2.num_blocks != 0 {
                assert_eq!(cap.block_def1.codewords + 1, cap.block_def2.codewords,
                           "Error in codewords numbers for symbol {:?}", sym);
                assert_eq!(cap.block_def1.data_codewords + 1, cap.block_def2.data_codewords,
                           "Error in codewords numbers for symbol {:?}", sym);
            }
        }
    }

    #[test]
    fn test_table4() {
        // check the number of data bits is exactly 8 times the number of data words
        // for all standard-size symbols
        for i in 1..15 {
            for l in [ECCLevel::L, ECCLevel::M, ECCLevel::Q, ECCLevel::H] {
                let cap = lookup_capacity(Size::Standard(i), l);

                assert_eq!(cap.data_codewords() * 8, cap.data_bits,
                           "Error in num data bits of symbol {}, level {:?}", i, l);
            }
        }
    }
}