
use qr_gen::serialization::*;
use qr_gen::serialization::masking::*;
use qr_gen::bitcoding::*;
use qr_gen::config::*;
use qr_gen::reedsolomon::construct_codewords;


#[test]
fn test_micro_symbol() {
    // define symbol properties for this test
    let size = Size::Micro(3);
    let ecl = ECCLevel::M;

    // encode some data
    let (data_bytes, ecc_bytes) = {
        let mut encoder = QrBitRecorder::new();
        encode_data_segment(&mut encoder, b"1234567", Encoding::Numeric, size);
        let data_content = finalize_bitstream(&mut encoder, size, ecl);
        construct_codewords(&data_content, size, ecl)  // compute ecc bytes + interleave
    };

    // create a canvas
    let mut canvas = create_qr_canvas(size);
    insert_data_payload(&mut canvas, size, &data_bytes, &ecc_bytes);

    // determine best mask and apply it
    let (mask_code, mut masked_symbol) = apply_best_mask(&canvas, size);

    // apply format bits
    insert_format_info(&mut masked_symbol, size, ecl, mask_code);

    // save it
    masked_symbol.save("./micro3M_1234567.test.png").unwrap();
}

#[test]
fn test_standard_symbol_6H() {
    // define symbol properties for this test
    let size = Size::Standard(6);
    let ecl = ECCLevel::H;

    // encode some data
    let (data_bytes, ecc_bytes) = {
        let mut encoder = QrBitRecorder::new();
        encode_data_segment(&mut encoder, b"AC-47", Encoding::Alphanumeric, size);
        let data_content = finalize_bitstream(&mut encoder, size, ecl);
        construct_codewords(&data_content, size, ecl)  // compute ecc bytes + interleave
    };

    // create a canvas
    let mut canvas = create_qr_canvas(size);
    insert_data_payload(&mut canvas, size, &data_bytes, &ecc_bytes);

    // determine best mask and apply it
    let (mask_code, mut masked_symbol) = apply_best_mask(&canvas, size);

    // apply format bits
    insert_format_info(&mut masked_symbol, size, ecl, mask_code);

    // save it
    masked_symbol.save("./standard6H_AC-47.test.png").unwrap();
}

#[test]
fn test_standard_symbol_7Q() {
    // define symbol properties for this test
    let size = Size::Standard(7);
    let ecl = ECCLevel::Q;

    // encode some data
    let (data_bytes, ecc_bytes) = {
        let mut encoder = QrBitRecorder::new();
        encode_data_segment(&mut encoder, b"AC-47", Encoding::Alphanumeric, size);
        let data_content = finalize_bitstream(&mut encoder, size, ecl);
        construct_codewords(&data_content, size, ecl)  // compute ecc bytes + interleave
    };

    // create a canvas
    let mut canvas = create_qr_canvas(size);
    insert_data_payload(&mut canvas, size, &data_bytes, &ecc_bytes);

    // determine best mask and apply it
    let (mask_code, mut masked_symbol) = apply_best_mask(&canvas, size);

    // apply format bits
    insert_format_info(&mut masked_symbol, size, ecl, mask_code);

    // apply version info bits
    insert_version_info(&mut masked_symbol, size);

    // save it
    masked_symbol.save("./standard7Q_AC-47.test.png").unwrap();
}