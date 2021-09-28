/// Contains enums and structs that will also be exported as the public
/// API of this crate.
use itertools::Itertools;
use std::cmp::{Ordering, PartialOrd};

//-------------------------------------------------------------------------------------------------

#[derive(Clone,Copy,Hash,Eq,PartialEq,Debug)]
pub enum Encoding {
    Numeric,            // only digits allowed [0-9]
    Alphanumeric,       // digits, capital letters and nine other chars [0-9A-Z$%*+-./: ]
    Bytes,              // ISO8859-1 encoded or otherwise (7.3.5)
    Kanji               // Kanji characters
}

impl Encoding {
    /// Return the number of bits used in the given symbol version/size to encode the number of
    /// characters for the following encoded content.
    pub fn num_char_count_bits(self, size: Size) -> usize {
        match size {
            Size::Micro(i) => {
                let a = i as usize;
                match self {
                    Encoding::Numeric => 2 + a,
                    Encoding::Alphanumeric | Encoding::Bytes => 1 + a,
                    Encoding::Kanji => a
                }
            },
            Size::Standard(1..=9) => match self {
                Encoding::Numeric => 10,
                Encoding::Alphanumeric => 9,
                Encoding::Bytes | Encoding::Kanji => 8,
            },
            Size::Standard(10..=26) => match self {
                Encoding::Numeric => 12,
                Encoding::Alphanumeric => 11,
                Encoding::Bytes => 16,
                Encoding::Kanji => 10,
            }
            Size::Standard(_) => match self {
                Encoding::Numeric => 14,
                Encoding::Alphanumeric => 13,
                Encoding::Bytes => 16,
                Encoding::Kanji => 12,
            }
        }
    }

    /// Compute the number of bits needed to encode a sequence with the given length
    /// of characters. Note that in Kanji encodings this is half the number of bytes,
    /// while in the other encodings it is equivalent with the number of bytes.
    pub fn num_encoded_bits(self, num_chars: usize) -> usize {
        match self {
            Encoding::Numeric => (10 * num_chars + 2) / 3,
            Encoding::Alphanumeric => (11 * num_chars + 1) / 2,
            Encoding::Bytes => num_chars * 8,
            Encoding::Kanji => num_chars * 13,
        }
    }

    /// Compute the lowest common encoding of two encodings in the sense of the partial
    /// ordering defined below.
    pub fn upper_bound(self, other: Self) -> Self {
        match self.partial_cmp(&other) {
            Some(Ordering::Less) | Some(Ordering::Equal) => other,
            Some(Ordering::Greater) => self,
            None => Encoding::Bytes  // bytes is an upper bound for every encoding
        }
    }
}

impl PartialOrd for Encoding {
    /// Defines a partial ordering for encodings, it is `a <= b` if `b` contains a superset of
    /// all characters supported by `a`.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (*self, *other) {
            (Encoding::Numeric, Encoding::Alphanumeric) |
            (Encoding::Numeric, Encoding::Bytes) |
            (Encoding::Alphanumeric, Encoding::Bytes) |
            (Encoding::Kanji, Encoding::Bytes) => Some(Ordering::Less),
            (Encoding::Alphanumeric, Encoding::Numeric) |
            (Encoding::Bytes, Encoding::Numeric) |
            (Encoding::Bytes, Encoding::Alphanumeric) |
            (Encoding::Bytes, Encoding::Kanji) => Some(Ordering::Greater),
            (a, b) if a == b => Some(Ordering::Equal),
            _ => None
        }
    }
}

//-------------------------------------------------------------------------------------------------

#[derive(Clone,Copy,Hash,Eq,PartialEq,Debug)]
pub enum Size {
    Micro(u8),         // versions M1 through M4
    Standard(u8)       // versions 1 through 40
}

impl Size {
    /// Convert a simple string description into a fitting enum
    /// value by parsing it. micro symbols are described as "M1"
    /// through "M4", the standard ones just by their size index, e.g. "6".
    pub fn from_str(decl: &str) -> Size {
        if decl.starts_with("M") {
            match &decl[1..] {
                "1" => Size::Micro(1),
                "2" => Size::Micro(2),
                "3" => Size::Micro(3),
                "4" => Size::Micro(4),
                _ => panic!("Unrecognized symbol configuration string!")
            }
        }
        else if let Ok(i) = decl.parse::<u8>() {
            if i >= 1 && i <= 40 {
                Size::Standard(i)
            }
            else {
                panic!("Unrecognized symbol configuration string!")
            }
        }
        else {
            panic!("Unrecognized symbol configuration string!")
        }
    }

    /// Simply return only version number, without the micro or standard
    pub fn version(self) -> u8 {
        match self {
            Size::Micro(i) => i,
            Size::Standard(i) => i
        }
    }

    /// Return quiet region size (counted only once)
    pub fn quiet_region_size(self) -> u32 {
        match self {
            Size::Micro(_) => 2,
            Size::Standard(_) => 4
        }
    }

    /// Return the width & height of the given size symbol, not counting the quiet region
    pub fn dimensions(self) -> u32 {
        match self {
            Size::Micro(i) => (i as u32) * 2 + 9,
            Size::Standard(i) => (i as u32) * 4 + 17
        }
    }

    /// Return the number of mode indicator bits
    pub fn num_mode_indicator_bits(self) -> usize {
        match self {
            Size::Micro(i) => (i as usize) - 1,
            Size::Standard(_) => 4
        }
    }

    /// Return terminator length (terminator is always only zero bits)
    pub fn terminator_length(self) -> usize {
        match self {
            Size::Micro(i) => 1 + 2 * i as usize,
            Size::Standard(_) => 4
        }
    }

    /// turn micro vs standard into a boolean
    pub fn is_micro(self) -> bool {
        match self {
            Size::Micro(_) => true,
            Size::Standard(_) => false
        }
    }
}

//-------------------------------------------------------------------------------------------------

#[derive(Clone,Copy,Hash,Eq,PartialEq,Debug)]
pub enum ECCLevel {
    L,      // allows recovery of  7% of the data
    M,      // allows recovery of 15% of the data
    Q,      // allows recovery of 25% of the data
    H       // allows recovery of 30% of the data
}

impl ECCLevel {
    /// Convert a simple string denoting the ECC level into
    /// the corresponding enum value
    pub fn from_str(desc: &str) -> ECCLevel {
        match desc {
            "L" => ECCLevel::L,
            "M" => ECCLevel::M,
            "Q" => ECCLevel::Q,
            "H" => ECCLevel::H,
            _ => panic!("Unrecognized symbol configuration string!")
        }
    }
}


#[derive(Clone,Copy,Hash, Eq, PartialEq,Debug)]
pub struct SymbolConfig(Size, ECCLevel);

impl SymbolConfig {
    /// Constructor
    pub const fn new(s: Size, e: ECCLevel) -> SymbolConfig {
        SymbolConfig(s, e)
    }

    /// Convenience function that creates a SymbolConfig from
    /// a string in the form commonly used in the standard,
    /// such as 1-H, M3-L, 6-M, etc.
    pub fn from_str(decl: &str) -> SymbolConfig {
        let (s, e) = decl.split("-").next_tuple().unwrap();
        SymbolConfig::new(Size::from_str(s), ECCLevel::from_str(e))
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_code_parsing() {
        assert_eq!(SymbolConfig::from_str("M2-M"), SymbolConfig::new(Size::Micro(2), ECCLevel::M));
        assert_eq!(SymbolConfig::from_str("M3-H"), SymbolConfig::new(Size::Micro(3), ECCLevel::H));
        assert_eq!(SymbolConfig::from_str("2-L"), SymbolConfig::new(Size::Standard(2), ECCLevel::L));
        assert_eq!(SymbolConfig::from_str("20-Q"), SymbolConfig::new(Size::Standard(20), ECCLevel::Q));
        assert_eq!(SymbolConfig::from_str("38-M"), SymbolConfig::new(Size::Standard(38), ECCLevel::M));
    }
}
