use num_traits::{Float, FloatConst, FromPrimitive, NumAssign};
use rustfft::FftNum;

/// Floating-point scalar used by the LB simulation.
/// `f32` and `f64` both satisfy this; the simulation code is generic over it so
/// a single `--precision` flag at the CLI picks the monomorphization.
pub trait Real:
    FftNum + Float + FloatConst + FromPrimitive + NumAssign + Send + Sync + 'static
{
    fn from_f64_lossy(x: f64) -> Self {
        Self::from_f64(x).unwrap()
    }
    fn to_f64_lossy(self) -> f64 {
        // Used only for JSON export. f32 → f64 is exact; f64 → f64 is identity.
        num_traits::ToPrimitive::to_f64(&self).unwrap()
    }
}

impl Real for f32 {}
impl Real for f64 {}
