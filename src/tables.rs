/// Contains lookup tables and other computation functions that provide reference
/// data needed for encoding or decing a QR code, such as the capacity of each
/// code configuration in different encodings, etc.

use crate::config::{Encoding, ECCLevel, Size, SymbolConfig};

use std::collections::HashMap;
use std::ops::Index;

#[derive(Clone,Copy,Hash,Eq,PartialEq,Debug)]
pub struct SymbolCapacity {
    pub codewords: u32,         // total number of codewords in this symbol type (depends only on size)
    pub data_codewords: u32,    // of those codewords, how many carry data (as opposed to ecc)
    pub data_bits: u32,         // how many data bits?

    chars_numeric: u32,         // data capacity measured in digits when using numeric encoding
    chars_alphanum: u32,        // data capacity measured in alphanum chars when usign alphanum encoding
    chars_bytes: u32,           // data capacity measured in bytes when using bytes encoding
    chars_kanji: u32,           // data capacity measured in kanji chars when using kanji encoding
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
    pub const fn new(words_total: u32,
                     words: u32,
                     bits: u32,
                     charsnum: u32,
                     charsalphanum: u32,
                     charsbytes: u32,
                     charskanji: u32) -> SymbolCapacity {
        SymbolCapacity {
            codewords: words_total,
            data_codewords: words,
            data_bits: bits,
            chars_numeric: charsnum,
            chars_alphanum: charsalphanum,
            chars_bytes: charsbytes,
            chars_kanji: charskanji
        }
    }

    pub fn ecc_words(&self) -> u32 {
        &self.codewords - &self.data_codewords
    }
}



macro_rules! define_capacity_table {
    {$(
        $size:expr,
        $ecc:expr,
        $total_words:expr,
        $words:expr,
        $bits:expr,
        $chars_n:expr,
        $chars_a:expr,
        $chars_b:expr,
        $chars_k:expr;
    )*} => {
        lazy_static! {
            pub static ref SYMBOL_CAPACITY_TABLE: HashMap<SymbolConfig, SymbolCapacity> = [
                $(
                    (SymbolConfig::new($size, $ecc), SymbolCapacity::new($total_words, $words, $bits, $chars_n, $chars_a, $chars_b, $chars_k)),
                )*
            ].iter().copied().collect();
        }
    }
}


define_capacity_table!(
    Size::Micro(1), ECCLevel::L, 5, 3, 20, 5, 0, 0, 0;

    Size::Micro(2), ECCLevel::L, 10, 5, 40, 10, 6, 0, 0;
    Size::Micro(2), ECCLevel::M, 10, 4, 32,  8, 5, 0, 0;

    Size::Micro(3), ECCLevel::L, 17, 11, 84, 23, 14, 9, 6;
    Size::Micro(3), ECCLevel::M, 17,  9, 68, 18, 11, 7, 4;

    Size::Micro(4), ECCLevel::L, 24, 16, 128, 35, 21, 15, 9;
    Size::Micro(4), ECCLevel::M, 24, 14, 112, 30, 18, 13, 8;
    Size::Micro(4), ECCLevel::Q, 24, 10,  80, 21, 13,  9, 4;

    Size::Standard(1), ECCLevel::L, 26, 19, 152, 41, 25, 17, 10;
    Size::Standard(1), ECCLevel::M, 26, 16, 128, 34, 20, 14,  8;
    Size::Standard(1), ECCLevel::Q, 26, 13, 104, 27, 16, 11,  7;
    Size::Standard(1), ECCLevel::H, 26,  9,  72, 17, 10,  7,  4;

    // TODO: extend with all symbol configurations
);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table() {
        assert_eq!(lookup_capacity(Size::Micro(3), ECCLevel::M).data_codewords, 9);
        assert_eq!(lookup_capacity(Size::Micro(3), ECCLevel::M)[Encoding::Bytes], 7);
        assert_eq!(lookup_capacity(Size::Standard(1), ECCLevel::Q).ecc_words(), 13);
    }
}