use geop_geometry::EQ_THRESHOLD;

pub const PROJECTION_THRESHOLD: f64 = EQ_THRESHOLD * 100.0;

pub mod space;
pub mod topology;
pub mod operations;
