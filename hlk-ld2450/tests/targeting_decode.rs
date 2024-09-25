mod common;

use common::MockSerial;
use hlk_ld2450::{RadarError, LD2450};

#[tokio::test]
async fn test_ld2450_next_radar_targets() {
    let data: [u8; 30] = [
        0xAA, 0xFF, 0x03, 0x00, 0x0E, 0x03, 0xB1, 0x86, 0x10, 0x00, 0x40, 0x01, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55, 0xCC,
    ];
    let serial = MockSerial::<30>::new(&data);

    let mut radar = LD2450::new_recycled_config(serial);
    let targets = radar.next_radar_targets().await.unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].x_coordinate, -782);
    assert_eq!(targets[0].y_coordinate, 1713);
    assert_eq!(targets[0].speed, -16);
    assert_eq!(targets[0].resolution, 320);
}

#[tokio::test]
async fn test_ld2450_next_radar_targets_out_of_sync() {
    let data: [u8; 35] = [
        0x00, 0x00, 0xFF, 0xAA, 0xFF, 0x03, 0x00, 0x0E, 0x03, 0xB1, 0x86, 0x10, 0x00, 0x40, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x55, 0xCC, 0xFF, 0xAA,
    ];
    let serial = MockSerial::<33>::new(&data);

    let mut radar = LD2450::new_recycled_config(serial);
    let targets = radar.next_radar_targets().await.unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].x_coordinate, -782);
    assert_eq!(targets[0].y_coordinate, 1713);
    assert_eq!(targets[0].speed, -16);
    assert_eq!(targets[0].resolution, 320);
}

#[tokio::test]
async fn test_ld2450_next_radar_targets_sneaky_out_of_sync() {
    // partial header match
    let data: [u8; 35] = [
        0xAA, 0xFF, 0xFF, 0xAA, 0xFF, 0x03, 0x00, 0x0E, 0x03, 0xB1, 0x86, 0x10, 0x00, 0x40, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x55, 0xCC, 0xFF, 0xAA,
    ];
    let serial = MockSerial::<33>::new(&data);

    let mut radar = LD2450::new_recycled_config(serial);
    let targets = radar.next_radar_targets().await.unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].x_coordinate, -782);
    assert_eq!(targets[0].y_coordinate, 1713);
    assert_eq!(targets[0].speed, -16);
    assert_eq!(targets[0].resolution, 320);
}

#[tokio::test]
async fn test_ld2450_next_radar_targets_invalid_length() {
    // partial header match
    let data: [u8; 35] = [
        0xAA, 0xFF, 0xFF, 0xAA, 0xFF, 0x03, 0x00, 0x0E, 0x03, 0xB1, 0x86, 0x10, 0x00, 0x40, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x55, 0xCC, 0xFF,
    ];
    let serial = MockSerial::<33>::new(&data);

    let mut radar = LD2450::new_recycled_config(serial);
    let targets = radar.next_radar_targets().await;

    assert_eq!(targets, Err(RadarError::UnexpectedFrameSize));
}
