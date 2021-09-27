
use qr_gen::*;


#[test]
fn test_micro_symbol() {
    // create symbol
    let masked_symbol = create_qr_code(b"1234567", Size::Micro(3), ECCLevel::M, Some(Encoding::Numeric));

    // save it
    masked_symbol.save("./micro3M_1234567.test.png").unwrap();
}

#[test]
fn test_standard_symbol_6H() {
    let masked_symbol = create_qr_code(b"AC-47", Size::Standard(6), ECCLevel::H, Some(Encoding::Alphanumeric));

    // save it
    masked_symbol.save("./standard6H_AC-47.test.png").unwrap();
}

#[test]
fn test_standard_symbol_7Q() {
    let masked_symbol = create_qr_code(b"AC-47", Size::Standard(7), ECCLevel::Q, Some(Encoding::Alphanumeric));

    // save it
    masked_symbol.save("./standard7Q_AC-47.test.png").unwrap();
}