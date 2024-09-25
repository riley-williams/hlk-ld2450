#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    pub tracking: TargetTrackingMode,
    pub bluetooth_enabled: bool,
    pub filtering_mode: FilteringMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetTrackingMode {
    Single,
    #[default]
    Multiple,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FilteringMode {
    /// No filtering
    #[default]
    None,
    /// Filter out targets inside the region.
    Inside(heapless::Vec<FilteredRegion, 3>),
    /// Filter out targets outside the region.
    Outside(heapless::Vec<FilteredRegion, 3>),
}

/// A radar region, defined by two diagonal vertices in mm, with the sensor at the origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilteredRegion {
    x_start: i16,
    y_start: i16,
    x_end: i16,
    y_end: i16,
}
