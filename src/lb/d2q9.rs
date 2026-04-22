use crate::lb::types::Real;

/// D2Q9 velocity set, ordered to match `binary_LB.py::_d2q9()`:
/// [rest, +x, +y, -x, -y, +x+y, -x+y, -x-y, +x-y]
pub const CX: [i32; 9] = [0, 1, 0, -1, 0, 1, -1, -1, 1];
pub const CY: [i32; 9] = [0, 0, 1, 0, -1, 1, 1, -1, -1];

pub fn weights<R: Real>() -> [R; 9] {
    let a = R::from_f64_lossy(4.0 / 9.0);
    let b = R::from_f64_lossy(1.0 / 9.0);
    let c = R::from_f64_lossy(1.0 / 36.0);
    [a, b, b, b, b, c, c, c, c]
}

pub fn cs2<R: Real>() -> R {
    R::from_f64_lossy(1.0 / 3.0)
}
