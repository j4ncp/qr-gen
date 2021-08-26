/// Contains enums and structs that will also be exported as the public
/// API of this crate.
use itertools::Itertools;

#[derive(Clone,Copy,Hash,Eq,PartialEq,Debug)]
pub enum Encoding {
    Numeric,            // only digits allowed [0-9]
    Alphanumeric,       // digits, capital letters and nine other chars [0-9A-Z$%*+-./: ]
    Bytes,              // ISO8859-1 encoded or otherwise (7.3.5)
    Kanji               // Kanji characters
}

#[derive(Clone,Copy,Hash,Eq,PartialEq,Debug)]
pub enum Size {
    Micro(u8),         // versions M1 through M3
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
}

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
