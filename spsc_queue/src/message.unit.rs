use super::*;

fn create_valid_message(value1: i16, value2: i16) -> Message {
    let [msb1, lsb1] = value1.to_be_bytes();
    let [msb2, lsb2] = value2.to_be_bytes();
    let bytes = [0x55, msb1, lsb1, msb2, lsb2];

    let sum = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));

    Message {
        bytes,
        checksum: !sum,
    }
}

#[test]
fn combine_msb_lsb_handles_positive_and_negative_values() {
    assert_eq!(Message::combine_msb_lsb(0x01, 0x00), 256);
    assert_eq!(Message::combine_msb_lsb(0xFF, 0xFF), -1);
    assert_eq!(Message::combine_msb_lsb(0x80, 0x00), i16::MIN);
    assert_eq!(Message::combine_msb_lsb(0x7F, 0xFF), i16::MAX);
}

#[test]
fn verifies_valid_checksum() {
    let valid = create_valid_message(256, -1);

    assert!(valid.verify());
}

#[test]
fn rejects_invalid_checksum() {
    let valid = create_valid_message(256, -1);

    let mut bad_checksum = valid;
    bad_checksum.checksum = bad_checksum.checksum.wrapping_add(1);

    assert!(!bad_checksum.verify());
}

#[test]
fn rejects_corrupted_payload_byte() {
    let mut mutated_message = create_valid_message(256, -1);

    // Flip a bit in a bytes[2]
    mutated_message.bytes[2] ^= 0x01;

    assert!(!mutated_message.verify());
}

#[test]
fn verifies_checksum_when_byte_sum_exceeds_255() {
    // Sum of bytes is 0x0451 (1105 > 255).
    // Wrapped to u8 -> 0x51 (81); bitwise NOT !0x51 = 0xAE (174).
    let message = create_valid_message(-1, -1);

    assert_eq!(message.bytes, [0x55, 0xFF, 0xFF, 0xFF, 0xFF]);
    assert_eq!(message.checksum, 174);
    assert!(message.verify());
}

#[test]
fn displays_message_in_expected_format() {
    let message = create_valid_message(256, -1);

    assert_eq!(message.to_string(), "85,1,0,256,255,255,-1,171\n")
}

#[test]
fn handles_zeroed_data() {
    let valid = create_valid_message(0, 0);

    assert!(valid.verify());
    assert_eq!(valid.to_string(), "85,0,0,0,0,0,0,170\n");
}
