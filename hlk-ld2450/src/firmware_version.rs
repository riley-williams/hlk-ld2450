use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FirmwareVersion {
    /// It is unclear from the datasheet what this is used for, and when it would ever
    /// not be 0. It is assumed by this crate that 0 corresponds to the "V1" in the version string
    pub firmware_type: u16,
    pub major: u16,
    pub minor: u32,
}

impl From<&[u8; 8]> for FirmwareVersion {
    fn from(data: &[u8; 8]) -> Self {
        let firmware_type = u16::from_le_bytes([data[0], data[1]]);
        let major_version_number = u16::from_le_bytes([data[2], data[3]]);
        let minor_version_number = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);

        FirmwareVersion {
            firmware_type,
            major: major_version_number,
            minor: minor_version_number,
        }
    }
}

impl fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "V{}.{:02}.{}",
            self.firmware_type + 1,
            self.major,
            self.minor
        )
    }
}

#[cfg(test)]
mod tests {
    use fmt::Write;

    use super::*;

    #[test]
    fn test_ord_eq() {
        let v1 = FirmwareVersion {
            firmware_type: 0,
            major: 1,
            minor: 2,
        };
        let v2 = FirmwareVersion {
            firmware_type: 0,
            major: 1,
            minor: 2,
        };

        assert!(v1 == v2);
    }

    #[test]
    fn test_ord_lt() {
        let v1 = FirmwareVersion {
            firmware_type: 0,
            major: 1,
            minor: 2,
        };
        let v2 = FirmwareVersion {
            firmware_type: 0,
            major: 1,
            minor: 3,
        };

        assert!(v1 < v2);
    }

    #[test]
    fn test_ord_gt() {
        let v1 = FirmwareVersion {
            firmware_type: 0,
            major: 1,
            minor: 2,
        };
        let v2 = FirmwareVersion {
            firmware_type: 0,
            major: 0,
            minor: 2,
        };

        assert!(v1 > v2);
    }

    #[test]
    fn test_ord_transitive() {
        let v1 = FirmwareVersion {
            firmware_type: 0,
            major: 1,
            minor: 587,
        };
        let v2 = FirmwareVersion {
            firmware_type: 0,
            major: 1,
            minor: 999,
        };
        let v3 = FirmwareVersion {
            firmware_type: 0,
            major: 2,
            minor: 0,
        };

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_display_version() {
        let firmware_version = FirmwareVersion {
            firmware_type: 0,
            major: 2,
            minor: 22062416,
        };
        let mut s = heapless::String::<32>::new();
        write!(s, "{}", firmware_version).unwrap();

        assert_eq!(s, "V1.02.22062416");
    }
}
