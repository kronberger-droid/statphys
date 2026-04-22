use std::sync::Arc;

use num_complex::Complex;
use rustfft::Fft;

use crate::lb::types::Real;

/// Frequency grid matching `np.fft.fftfreq(n) * 2π`.
pub fn fftfreq<R: Real>(n: usize, two_pi: R) -> Vec<R> {
    let mut out = vec![R::zero(); n];
    let n_r = R::from_usize(n).unwrap();
    let half = (n + 1) / 2;
    for i in 0..half {
        out[i] = two_pi * R::from_usize(i).unwrap() / n_r;
    }
    for i in half..n {
        out[i] = two_pi * (R::from_usize(i).unwrap() - n_r) / n_r;
    }
    out
}

/// In-place 2D FFT via row + transposed-column passes.
/// `buf` is (ny, nx) row-major; `col_buf` must have length `ny`; `scratch` is
/// the shared rustfft in-place scratch (at least the larger of the two plans').
pub fn fft2_in_place<R: Real>(
    buf: &mut [Complex<R>],
    nx: usize,
    ny: usize,
    fft_x: &Arc<dyn Fft<R>>,
    fft_y: &Arc<dyn Fft<R>>,
    col_buf: &mut [Complex<R>],
    scratch: &mut [Complex<R>],
) {
    debug_assert_eq!(buf.len(), nx * ny);
    debug_assert_eq!(col_buf.len(), ny);

    for iy in 0..ny {
        let row = &mut buf[iy * nx..(iy + 1) * nx];
        fft_x.process_with_scratch(row, scratch);
    }
    for ix in 0..nx {
        for iy in 0..ny {
            col_buf[iy] = buf[iy * nx + ix];
        }
        fft_y.process_with_scratch(col_buf, scratch);
        for iy in 0..ny {
            buf[iy * nx + ix] = col_buf[iy];
        }
    }
}
