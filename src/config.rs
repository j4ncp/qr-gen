pub enum Encoding {
    Numeric,            // only digits allowed [0-9]
    Alphanumeric,       // digits, capital letters and nine other chars [0-9A-Z$%*+-./: ]
    Bytes,              // ISO8859-1 encoded or otherwise (7.3.5)
    Kanji               // Kanji characters
}

pub enum Size {
    Micro(u8),         // versions M1 through M3
    Standard(u8)       // versions 1 through 40
}

pub enum ECCLevel {
    L,      // allows recovery of  7% of the data
    M,      // allows recovery of 15% of the data
    Q,      // allows recovery of 25% of the data
    H       // allows recovery of 30% of the data
}