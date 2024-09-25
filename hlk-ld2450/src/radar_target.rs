use crate::RadarError;

#[derive(Debug, Clone, PartialEq)]
pub struct RadarTarget {
    /// X coordinate of the target in mm
    pub x_coordinate: i16,
    /// Y coordinate of the target in mm
    pub y_coordinate: i16,
    /// Speed of the target in cm/s relative to the radar
    /// Negative values indicate relative movement towards the radar
    ///
    /// This is not the true speed, but the speed component in the direction away from the radar
    /// s_measured = s_true * cos(θ)
    /// where θ is the angle between the radar normal and the target velocity
    ///
    /// A target moving tangent to the radar will measure 0 speed
    pub speed: i16,
    /// The resolution of the distance measurement in mm (distance gate size)
    pub resolution: u16,
}

impl RadarTarget {
    /// Returns true if there is no target being tracked (all zeroes)
    pub fn is_untracked(&self) -> bool {
        self.x_coordinate == 0 && self.y_coordinate == 0 && self.speed == 0 && self.resolution == 0
    }
}

impl TryFrom<&[u8]> for RadarTarget {
    type Error = RadarError;
    fn try_from(data: &[u8]) -> Result<Self, RadarError> {
        if data.len() != 8 {
            return Err(RadarError::UnexpectedFrameSize);
        }

        Ok(Self {
            x_coordinate: i16_from_le_weird_sign([data[0], data[1]]),
            y_coordinate: i16_from_le_weird_sign([data[2], data[3]]),
            speed: i16_from_le_weird_sign([data[4], data[5]]),
            resolution: u16::from_le_bytes([data[6], data[7]]),
        })
    }
}

/// This decodes the weird signed 16-bit integer format used by the LD2450 into a regular i16
fn i16_from_le_weird_sign(mut data: [u8; 2]) -> i16 {
    // A 1 highest bit indicates a positive number
    if data[1] & 0b1000_0000 != 0 {
        data[1] &= 0b0111_1111;
        i16::from_le_bytes(data)
    } else {
        -i16::from_le_bytes(data)
    }
}

pub(crate) fn decode_radar_targets(
    data: &[u8; 24],
) -> Result<heapless::Vec<RadarTarget, 3>, RadarError> {
    let mut targets = heapless::Vec::new();
    for i in 0..3 {
        let range = i * 8..(i + 1) * 8;
        let target = RadarTarget::try_from(&data[range])?;
        if !target.is_untracked() {
            // Safety: The push will not overflow because of the for loop range
            unsafe { targets.push_unchecked(target) };
        }
    }
    Ok(targets)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_radar_target_from() {
        let data: [u8; 8] = [0x0E, 0x03, 0xB1, 0x86, 0x10, 0x00, 0x40, 0x01];
        let target = RadarTarget::try_from(&data[..]).unwrap();
        assert_eq!(target.x_coordinate, -782);
        assert_eq!(target.y_coordinate, 1713);
        assert_eq!(target.speed, -16);
        assert_eq!(target.resolution, 320);
    }

    // test decoding full frame:
    // 0xAA, 0xFF, 0x03, 0x00,
    // 0x0E, 0x03, 0xB1, 0x86, 0x10, 0x00, 0x40, 0x01,
    // 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // 0x55, 0xCC,
    #[test]
    fn test_full_frame() {
        let data: [u8; 24] = [
            0x0E, 0x03, 0xB1, 0x86, 0x10, 0x00, 0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let targets = decode_radar_targets(&data).unwrap();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].x_coordinate, -782);
    }
}
