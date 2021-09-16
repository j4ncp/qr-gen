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

    /// compute and return the number of ecc codewords per block for this symbol
    pub fn ecc_words_per_block(&self) -> u32 { &self.block_def1.codewords - &self.block_def1.data_codewords }
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

    Size::Standard(15), ECCLevel::L, 4184, 1250, 758, 520, 320;  5, (109, 87,); 1, (110, 88,);
    Size::Standard(15), ECCLevel::M, 3320,  991, 600, 412, 254;  5, ( 65, 41,); 5, ( 66, 42,);
    Size::Standard(15), ECCLevel::Q, 2360,  703, 426, 292, 180;  5, ( 54, 24,); 7, ( 55, 25,);
    Size::Standard(15), ECCLevel::H, 1784,  530, 321, 220, 136; 11, ( 36, 12,); 7, ( 37, 13,);

    Size::Standard(16), ECCLevel::L, 4712, 1408, 854, 586, 361;  5, (122, 98,);  1, (123, 99,);
    Size::Standard(16), ECCLevel::M, 3624, 1082, 656, 450, 277;  7, ( 73, 45,);  3, ( 74, 46,);
    Size::Standard(16), ECCLevel::Q, 2600,  775, 470, 322, 198; 15, ( 43, 19,);  2, ( 44, 20,);
    Size::Standard(16), ECCLevel::H, 2024,  602, 365, 250, 154;  3, ( 45, 15,); 13, ( 46, 16,);

    Size::Standard(17), ECCLevel::L, 5176, 1548, 938, 644, 397;  1, (135, 107,);  5, (136, 108,);
    Size::Standard(17), ECCLevel::M, 4056, 1212, 734, 504, 310; 10, ( 74,  46,);  1, ( 75,  47,);
    Size::Standard(17), ECCLevel::Q, 2936,  876, 531, 364, 224;  1, ( 50,  22,); 15, ( 51,  23,);
    Size::Standard(17), ECCLevel::H, 2264,  674, 408, 280, 173;  2, ( 42,  14,); 17, ( 43,  15,);

    Size::Standard(18), ECCLevel::L, 5768, 1725, 1046, 718, 442;  5, (150, 120,);  1, (151, 121,);
    Size::Standard(18), ECCLevel::M, 4504, 1346,  816, 560, 345;  9, ( 69,  43,);  4, ( 70,  44,);
    Size::Standard(18), ECCLevel::Q, 3176,  948,  574, 394, 243; 17, ( 50,  22,);  1, ( 51,  23,);
    Size::Standard(18), ECCLevel::H, 2504,  746,  452, 310, 191;  2, ( 42,  14,); 19, ( 43,  15,);

    Size::Standard(19), ECCLevel::L, 6360, 1903, 1153, 792, 488;  3, (141, 113,);  4, (142, 114,);
    Size::Standard(19), ECCLevel::M, 5016, 1500,  909, 624, 384;  3, ( 70,  44,); 11, ( 71,  45,);
    Size::Standard(19), ECCLevel::Q, 3560, 1063,  644, 442, 272; 17, ( 47,  21,);  4, ( 48,  22,);
    Size::Standard(19), ECCLevel::H, 2728,  813,  493, 338, 208;  9, ( 39,  13,); 16, ( 40,  14,);

    Size::Standard(20), ECCLevel::L, 6888, 2061, 1249, 858, 528;  3, (135, 107,);  5, (136, 108,);
    Size::Standard(20), ECCLevel::M, 5352, 1600,  970, 666, 410;  3, ( 67,  41,); 13, ( 68,  42,);
    Size::Standard(20), ECCLevel::Q, 3880, 1159,  702, 482, 297; 15, ( 54,  24,);  5, ( 55,  25,);
    Size::Standard(20), ECCLevel::H, 3080,  919,  557, 382, 235; 15, ( 43,  15,); 10, ( 44,  16,);

    Size::Standard(21), ECCLevel::L, 7456, 2232, 1352, 929, 572;  4, (144, 116,);  4, (145, 117,);
    Size::Standard(21), ECCLevel::M, 5712, 1708, 1035, 711, 438; 17, ( 68,  42,);  0, (  0,   0,);
    Size::Standard(21), ECCLevel::Q, 4096, 1224,  742, 509, 314; 17, ( 50,  22,);  6, ( 51,  23,);
    Size::Standard(21), ECCLevel::H, 3248,  969,  587, 403, 248; 19, ( 46,  16,);  6, ( 47,  17,);

    Size::Standard(22), ECCLevel::L, 8048, 2409, 1460, 1003, 618;  2, (139, 111,);  7, (140, 112,);
    Size::Standard(22), ECCLevel::M, 6256, 1872, 1134,  779, 480; 17, ( 74,  46,);  0, (  0,   0,);
    Size::Standard(22), ECCLevel::Q, 4544, 1358,  823,  565, 348;  7, ( 54,  24,); 16, ( 55,  25,);
    Size::Standard(22), ECCLevel::H, 3536, 1056,  640,  439, 270; 34, ( 37,  13,);  0, (  0,   0,);

    Size::Standard(23), ECCLevel::L, 8752, 2620, 1588, 1091, 672;  4, (151, 121,);  5, (152, 122,);
    Size::Standard(23), ECCLevel::M, 6880, 2059, 1248,  857, 528;  4, ( 75,  47,); 14, ( 76,  48,);
    Size::Standard(23), ECCLevel::Q, 4912, 1468,  890,  611, 376; 11, ( 54,  24,); 14, ( 55,  25,);
    Size::Standard(23), ECCLevel::H, 3712, 1108,  672,  461, 284; 16, ( 45,  15,); 14, ( 46,  16,);

    Size::Standard(24), ECCLevel::L, 9392, 2812, 1704, 1171, 721;  6, (147, 117,);  4, (148, 118,);
    Size::Standard(24), ECCLevel::M, 7312, 2188, 1326,  911, 561;  6, ( 73,  45,); 14, ( 74,  46,);
    Size::Standard(24), ECCLevel::Q, 5312, 1588,  963,  661, 407; 11, ( 54,  24,); 16, ( 55,  25,);
    Size::Standard(24), ECCLevel::H, 4112, 1228,  744,  511, 315; 30, ( 46,  16,);  2, ( 47,  17,);

    Size::Standard(25), ECCLevel::L, 10208, 3057, 1853, 1273, 784;  8, (132, 106,);  4, (133, 107,);
    Size::Standard(25), ECCLevel::M,  8000, 2395, 1451,  997, 614;  8, ( 75,  47,); 13, ( 76,  48,);
    Size::Standard(25), ECCLevel::Q,  5744, 1718, 1041,  715, 440;  7, ( 54,  24,); 22, ( 55,  25,);
    Size::Standard(25), ECCLevel::H,  4304, 1286,  779,  535, 330; 22, ( 45,  15,); 13, ( 46,  16,);

    Size::Standard(26), ECCLevel::L, 10960, 3283, 1990, 1367, 842; 10, (142, 114,);  2, (143, 115,);
    Size::Standard(26), ECCLevel::M,  8496, 2544, 1542, 1059, 652; 19, ( 74,  46,);  4, ( 75,  47,);
    Size::Standard(26), ECCLevel::Q,  6032, 1804, 1094,  751, 462; 28, ( 50,  22,);  6, ( 51,  23,);
    Size::Standard(26), ECCLevel::H,  4768, 1425,  864,  593, 365; 33, ( 46,  16,);  4, ( 47,  17,);

    Size::Standard(27), ECCLevel::L, 11744, 3517, 2132, 1465, 902;  8, (152, 122,);  4, (153, 123,);
    Size::Standard(27), ECCLevel::M,  9024, 2701, 1637, 1125, 692; 22, ( 73,  45,);  3, ( 74,  46,);
    Size::Standard(27), ECCLevel::Q,  6464, 1933, 1172,  805, 496;  8, ( 53,  23,); 26, ( 54,  24,);
    Size::Standard(27), ECCLevel::H,  5024, 1501,  910,  625, 385; 12, ( 45,  15,); 28, ( 46,  16,);

    Size::Standard(28), ECCLevel::L, 12248, 3669, 2223, 1528, 940;  3, (147, 117,); 10, (148, 118,);
    Size::Standard(28), ECCLevel::M,  9544, 2857, 1732, 1190, 732;  3, ( 73,  45,); 23, ( 74,  46,);
    Size::Standard(28), ECCLevel::Q,  6968, 2085, 1263,  868, 534;  4, ( 54,  24,); 31, ( 55,  25,);
    Size::Standard(28), ECCLevel::H,  5288, 1581,  958,  658, 405; 11, ( 45,  15,); 31, ( 46,  16,);

    Size::Standard(29), ECCLevel::L, 13048, 3909, 2369, 1628, 1002;  7, (146, 116,);  7, (147, 117,);
    Size::Standard(29), ECCLevel::M, 10136, 3035, 1839, 1264,  778; 21, ( 73,  45,);  7, ( 74,  46,);
    Size::Standard(29), ECCLevel::Q,  7288, 2181, 1322,  908,  559;  1, ( 53,  23,); 37, ( 54,  24,);
    Size::Standard(29), ECCLevel::H,  5608, 1677, 1016,  698,  430; 19, ( 45,  15,); 26, ( 46,  16,);

    Size::Standard(30), ECCLevel::L, 13880, 4158, 2520, 1732, 1066;  5, (145, 115,); 10, (146, 116,);
    Size::Standard(30), ECCLevel::M, 10984, 3289, 1994, 1370,  843; 19, ( 75,  47,); 10, ( 76,  48,);
    Size::Standard(30), ECCLevel::Q,  7880, 2358, 1429,  982,  604; 15, ( 54,  24,); 25, ( 55,  25,);
    Size::Standard(30), ECCLevel::H,  5960, 1782, 1080,  742,  457; 23, ( 45,  15,); 25, ( 46,  16,);

    Size::Standard(31), ECCLevel::L, 14744, 4417, 2677, 1840, 1132; 13, (145, 115,);  3, (146, 116,);
    Size::Standard(31), ECCLevel::M, 11640, 3486, 2113, 1452,  894;  2, ( 74,  46,); 29, ( 75,  47,);
    Size::Standard(31), ECCLevel::Q,  8264, 2473, 1499, 1030,  634; 42, ( 54,  24,);  1, ( 55,  25,);
    Size::Standard(31), ECCLevel::H,  6344, 1897, 1150,  790,  486; 23, ( 45,  15,); 28, ( 46,  16,);

    Size::Standard(32), ECCLevel::L, 15640, 4686, 2840, 1952, 1201; 17, (145, 115,);  0, (  0,   0,);
    Size::Standard(32), ECCLevel::M, 12328, 3693, 2238, 1538,  947; 10, ( 74,  46,); 23, ( 75,  47,);
    Size::Standard(32), ECCLevel::Q,  8920, 2670, 1618, 1112,  684; 10, ( 54,  24,); 35, ( 55,  25,);
    Size::Standard(32), ECCLevel::H,  6760, 2022, 1226,  842,  518; 19, ( 45,  15,); 35, ( 46,  16,);

    Size::Standard(33), ECCLevel::L, 16568, 4965, 3009, 2068, 1273; 17, (145, 115,);  1, (146, 116,);
    Size::Standard(33), ECCLevel::M, 13048, 3909, 2369, 1628, 1002; 14, ( 74,  46,); 21, ( 75,  47,);
    Size::Standard(33), ECCLevel::Q,  9368, 2805, 1700, 1168,  719; 29, ( 54,  24,); 19, ( 55,  25,);
    Size::Standard(33), ECCLevel::H,  7208, 2157, 1307,  898,  553; 11, ( 45,  15,); 46, ( 46,  16,);

    Size::Standard(34), ECCLevel::L, 17528, 5253, 3183, 2188, 1347; 13, (145, 115,);  6, (146, 116,);
    Size::Standard(34), ECCLevel::M, 13800, 4134, 2506, 1722, 1060; 14, ( 74,  46,); 23, ( 75,  47,);
    Size::Standard(34), ECCLevel::Q,  9848, 2949, 1787, 1228,  756; 44, ( 54,  24,);  7, ( 55,  25,);
    Size::Standard(34), ECCLevel::H,  7688, 2301, 1394,  958,  590; 59, ( 46,  16,);  1, ( 47,  17,);

    Size::Standard(35), ECCLevel::L, 18448, 5529, 3351, 2303, 1417; 12, (151, 121,);  7, (152, 122,);
    Size::Standard(35), ECCLevel::M, 14496, 4343, 2632, 1809, 1113; 12, ( 75,  47,); 26, ( 76,  48,);
    Size::Standard(35), ECCLevel::Q, 10288, 3081, 1867, 1283,  790; 39, ( 54,  24,); 14, ( 55,  25,);
    Size::Standard(35), ECCLevel::H,  7888, 2361, 1431,  983,  605; 22, ( 45,  15,); 41, ( 46,  16,);

    Size::Standard(36), ECCLevel::L, 19472, 5836, 3537, 2431, 1496;  6, (151, 121,); 14, (152, 122,);
    Size::Standard(36), ECCLevel::M, 15312, 4588, 2780, 1911, 1176;  6, ( 75,  47,); 34, ( 76,  48,);
    Size::Standard(36), ECCLevel::Q, 10832, 3244, 1966, 1351,  832; 46, ( 54,  24,); 10, ( 55,  25,);
    Size::Standard(36), ECCLevel::H,  8432, 2524, 1530, 1051,  647;  2, ( 45,  15,); 64, ( 46,  16,);

    Size::Standard(37), ECCLevel::L, 20528, 6153, 3729, 2563, 1577; 17, (152, 122,);  4, (153, 123,);
    Size::Standard(37), ECCLevel::M, 15936, 4775, 2894, 1989, 1224; 29, ( 74,  46,); 14, ( 75,  47,);
    Size::Standard(37), ECCLevel::Q, 11408, 3417, 2071, 1423,  876; 49, ( 54,  24,); 10, ( 55,  25,);
    Size::Standard(37), ECCLevel::H,  8768, 2625, 1591, 1093,  673; 24, ( 45,  15,); 46, ( 46,  16,);

    Size::Standard(38), ECCLevel::L, 21616, 6479, 3927, 2699, 1661;  4, (152, 122,); 18, (153, 123,);
    Size::Standard(38), ECCLevel::M, 16816, 5039, 3054, 2099, 1292; 13, ( 74,  46,); 32, ( 75,  47,);
    Size::Standard(38), ECCLevel::Q, 12016, 3599, 2181, 1499,  923; 48, ( 54,  24,); 14, ( 55,  25,);
    Size::Standard(38), ECCLevel::H,  9136, 2735, 1658, 1139,  701; 42, ( 45,  15,); 32, ( 46,  16,);

    Size::Standard(39), ECCLevel::L, 22496, 6743, 4087, 2809, 1729; 20, (147, 117,);  4, (148, 118,);
    Size::Standard(39), ECCLevel::M, 17728, 5313, 3220, 2213, 1362; 40, ( 75,  47,);  7, ( 76,  48,);
    Size::Standard(39), ECCLevel::Q, 12656, 3791, 2298, 1579,  972; 43, ( 54,  24,); 22, ( 55,  25,);
    Size::Standard(39), ECCLevel::H,  9776, 2927, 1774, 1219,  750; 10, ( 45,  15,); 67, ( 46,  16,);

    Size::Standard(40), ECCLevel::L, 23648, 7089, 4296, 2953, 1817; 19, (148, 118,);  6, (149, 119,);
    Size::Standard(40), ECCLevel::M, 18672, 5596, 3391, 2331, 1435; 18, ( 75,  47,); 31, ( 76,  48,);
    Size::Standard(40), ECCLevel::Q, 13328, 3993, 2420, 1663, 1024; 34, ( 54,  24,); 34, ( 55,  25,);
    Size::Standard(40), ECCLevel::H, 10208, 3057, 1852, 1273,  784; 20, ( 45,  15,); 61, ( 46,  16,);
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
        for i in 1..31 {
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
        for i in 1..31 {
            for l in [ECCLevel::L, ECCLevel::M, ECCLevel::Q, ECCLevel::H] {
                let cap = lookup_capacity(Size::Standard(i), l);

                assert_eq!(cap.data_codewords() * 8, cap.data_bits,
                           "Error in num data bits of symbol {}, level {:?}", i, l);
            }
        }
    }
}