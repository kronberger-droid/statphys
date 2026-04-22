//! Analysis helpers for exercise 5: domain size L(t), largest cluster, minority
//! cell count.

use std::sync::Arc;

use num_complex::Complex;
use rustfft::{Fft, FftDirection, FftPlanner};

use crate::lb::fft::{fft2_in_place, fftfreq};
use crate::lb::types::Real;

/// Reusable state for `compute_l`: the FFT plans, scratch, k-grid, complex buffer.
/// Build once per series (`compute_l_series`), not per snapshot.
pub struct DomainSizeCalc<R: Real> {
    nx: usize,
    ny: usize,
    fft_x: Arc<dyn Fft<R>>,
    fft_y: Arc<dyn Fft<R>>,
    buf: Vec<Complex<R>>,
    col_buf: Vec<Complex<R>>,
    scratch: Vec<Complex<R>>,
    k_mag: Vec<f64>,
}

impl<R: Real> DomainSizeCalc<R> {
    pub fn new(nx: usize, ny: usize) -> Self {
        let mut planner: FftPlanner<R> = FftPlanner::new();
        let fft_x = planner.plan_fft(nx, FftDirection::Forward);
        let fft_y = planner.plan_fft(ny, FftDirection::Forward);
        let scratch_len = fft_x
            .get_inplace_scratch_len()
            .max(fft_y.get_inplace_scratch_len());
        let two_pi = R::from_f64_lossy(2.0 * std::f64::consts::PI);
        let kx = fftfreq::<R>(nx, two_pi);
        let ky = fftfreq::<R>(ny, two_pi);
        let mut k_mag = vec![0.0_f64; nx * ny];
        for iy in 0..ny {
            for ix in 0..nx {
                let kxr = kx[ix];
                let kyr = ky[iy];
                k_mag[iy * nx + ix] = (kxr * kxr + kyr * kyr).sqrt().to_f64_lossy();
            }
        }
        Self {
            nx,
            ny,
            fft_x,
            fft_y,
            buf: vec![Complex::new(R::zero(), R::zero()); nx * ny],
            col_buf: vec![Complex::new(R::zero(), R::zero()); ny],
            scratch: vec![Complex::new(R::zero(), R::zero()); scratch_len],
            k_mag,
        }
    }

    /// Domain size L(t) = 2π / ⟨k⟩, with ⟨k⟩ the first moment of S(k) = |FFT(phi - mean)|².
    pub fn compute(&mut self, phi: &[R]) -> f64 {
        let n_cells = self.nx * self.ny;
        debug_assert_eq!(phi.len(), n_cells);

        let mean: R = phi.iter().copied().fold(R::zero(), |a, b| a + b)
            / R::from_usize(n_cells).unwrap();
        for (dst, &p) in self.buf.iter_mut().zip(phi.iter()) {
            *dst = Complex::new(p - mean, R::zero());
        }
        fft2_in_place(
            &mut self.buf,
            self.nx,
            self.ny,
            &self.fft_x,
            &self.fft_y,
            &mut self.col_buf,
            &mut self.scratch,
        );

        let mut num = 0.0_f64;
        let mut den = 0.0_f64;
        for idx in 0..n_cells {
            let kmag = self.k_mag[idx];
            if kmag <= 0.0 {
                continue;
            }
            let c = self.buf[idx];
            let s_k = (c.re * c.re + c.im * c.im).to_f64_lossy();
            num += kmag * s_k;
            den += s_k;
        }
        let k_mean = num / den;
        2.0 * std::f64::consts::PI / k_mean
    }
}

pub fn compute_l_series<R: Real>(
    phi_history: &[Vec<R>],
    nx: usize,
    ny: usize,
) -> Vec<f64> {
    let mut calc = DomainSizeCalc::<R>::new(nx, ny);
    phi_history.iter().map(|p| calc.compute(p)).collect()
}

/// Number of cells on the minority side of `threshold`. If `phi_mean < 0`, the
/// minority phase is positive; otherwise negative.
pub fn minority_count<R: Real>(phi: &[R], threshold: f64, phi_mean: f64) -> i64 {
    let t = R::from_f64_lossy(threshold);
    if phi_mean <= 0.0 {
        phi.iter().filter(|&&p| p > t).count() as i64
    } else {
        phi.iter().filter(|&&p| p < -t).count() as i64
    }
}

/// Largest connected cluster (4-connectivity, periodic wrap) on the minority side
/// of `threshold`. Matches `scipy.ndimage.label` with `structure=generate_binary_structure(2, 1)`.
pub fn largest_cluster<R: Real>(
    phi: &[R],
    nx: usize,
    ny: usize,
    threshold: f64,
    phi_mean: f64,
) -> i64 {
    let t = R::from_f64_lossy(threshold);
    let positive_minority = phi_mean <= 0.0;
    let n_cells = nx * ny;
    // label[idx]: 0 = unvisited, >0 = component id. Non-minority cells are never enqueued.
    let mut label = vec![0i32; n_cells];
    let mut stack: Vec<usize> = Vec::new();
    let mut max_size: i64 = 0;
    let mut next_label: i32 = 1;

    for start in 0..n_cells {
        if label[start] != 0 {
            continue;
        }
        let p = phi[start];
        let in_minority = if positive_minority { p > t } else { p < -t };
        if !in_minority {
            continue;
        }
        label[start] = next_label;
        stack.clear();
        stack.push(start);
        let mut size: i64 = 0;
        while let Some(idx) = stack.pop() {
            size += 1;
            let iy = idx / nx;
            let ix = idx % nx;
            let neighbors = [
                iy * nx + (ix + 1) % nx,
                iy * nx + (ix + nx - 1) % nx,
                ((iy + 1) % ny) * nx + ix,
                ((iy + ny - 1) % ny) * nx + ix,
            ];
            for nidx in neighbors {
                if label[nidx] != 0 {
                    continue;
                }
                let pn = phi[nidx];
                let nin = if positive_minority { pn > t } else { pn < -t };
                if nin {
                    label[nidx] = next_label;
                    stack.push(nidx);
                }
            }
        }
        next_label += 1;
        if size > max_size {
            max_size = size;
        }
    }
    max_size
}
