use crate::lb::types::Real;

/// Bulk chemical potential at a single point, generic over precision.
/// Mirrors `binary_LB.py::chemical_potential_bulk`.
#[inline]
pub fn mu_bulk_point<R: Real>(phi: R, n0: R, t: R, lam: R, clip_frac: R) -> R {
    let half = R::from_f64_lossy(0.5);
    let bound = clip_frac * n0;
    let phi_c = if phi > bound {
        bound
    } else if phi < -bound {
        -bound
    } else {
        phi
    };
    let log_ratio = ((n0 + phi_c) / (n0 - phi_c)).ln();
    -half * lam * phi / n0 + half * t * log_ratio
}

/// Exact spinodal ∆n(T) at fixed n0, symmetric Orlandini bulk.
/// Matches `binary_LB.py::exact_spinodal_phi`.
pub fn spinodal_phi<R: Real>(t: R, lam: R, n0: R) -> R {
    let one = R::one();
    let two = R::from_f64_lossy(2.0);
    let val = (one - two * t / lam).max(R::zero());
    n0 * val.sqrt()
}

/// Exact binodal ∆n(T) at fixed n0 via bisection.
/// Matches `binary_LB.py::exact_binodal_phi`.
pub fn binodal_phi<R: Real>(t: R, lam: R, n0: R) -> R {
    let two = R::from_f64_lossy(2.0);
    let tc = lam / two;
    if t <= R::zero() {
        return n0;
    }
    if t >= tc {
        return R::zero();
    }
    let alpha = lam / (two * t);
    let mut lo = R::from_f64_lossy(1e-14);
    let mut hi = R::one() - R::from_f64_lossy(1e-12);
    let tol = R::from_f64_lossy(1e-12);
    for _ in 0..200 {
        let mid = (lo + hi) * R::from_f64_lossy(0.5);
        let g = mid.atanh() - alpha * mid;
        if g.abs() < tol || (hi - lo) < tol {
            return n0 * mid;
        }
        if g > R::zero() {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    n0 * (lo + hi) * R::from_f64_lossy(0.5)
}
