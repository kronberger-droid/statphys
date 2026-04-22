use std::sync::Arc;

use num_complex::Complex;
use rand::{Rng, SeedableRng};
use rand_distr::StandardNormal;
use rand_xoshiro::Xoshiro256PlusPlus;
use rustfft::{Fft, FftDirection, FftPlanner};

use crate::lb::d2q9::{CX, CY, cs2, weights};
use crate::lb::fft::{fft2_in_place, fftfreq};
use crate::lb::free_energy::mu_bulk_point;
use crate::lb::types::Real;

#[derive(Clone, Debug)]
pub struct FluidParams {
    pub nx: usize,
    pub ny: usize,
    pub n0: f64,
    pub lam: f64,
    pub t: f64,
    pub kappa: f64,
    pub m_mobility: f64,
    pub tau: f64,
    pub dt: f64,
    pub phi0: f64,
    pub phi_noise: f64,
    pub kt: f64,
    pub seed: u64,
    pub hydrodynamics: bool,
    pub dealias_clip: f64,
}

impl Default for FluidParams {
    fn default() -> Self {
        Self {
            nx: 128,
            ny: 128,
            n0: 1.0,
            lam: 1.1,
            t: 0.50,
            kappa: 0.12,
            m_mobility: 0.08,
            tau: 0.9,
            dt: 0.5,
            phi0: 0.0,
            phi_noise: 1e-3,
            kt: 0.0,
            seed: 0,
            hydrodynamics: true,
            dealias_clip: 0.999999,
        }
    }
}

/// 2D binary-fluid LB simulation, generic over precision `R`.
///
/// Grid fields are flat `Vec<R>` of length `nx * ny`, row-major with `idx = iy * nx + ix`.
pub struct Fluid2D<R: Real> {
    nx: usize,
    ny: usize,
    n_cells: usize,

    n0: R,
    lam: R,
    t: R,
    kappa: R,
    m_mobility: R,
    tau: R,
    dt: R,
    kt: R,
    clip_frac: R,
    hydrodynamics: bool,
    phi_mean_target: R,

    phi: Vec<R>,
    rho: Vec<R>,
    ux: Vec<R>,
    uy: Vec<R>,
    fx: Vec<R>,
    fy: Vec<R>,
    f: [Vec<R>; 9],
    f_next: [Vec<R>; 9],

    k2: Vec<R>,
    k4: Vec<R>,

    fft_fwd_x: Arc<dyn Fft<R>>,
    fft_inv_x: Arc<dyn Fft<R>>,
    fft_fwd_y: Arc<dyn Fft<R>>,
    fft_inv_y: Arc<dyn Fft<R>>,
    fft_buf: Vec<Complex<R>>,
    fft_scratch: Vec<Complex<R>>,
    fft_col_buf: Vec<Complex<R>>,

    // Reusable scratch buffers (avoid per-step heap allocations).
    mu_scratch: Vec<R>,
    noise_jx: Vec<R>,
    noise_jy: Vec<R>,

    // Precomputed periodic-neighbor index tables (length nx / ny).
    ix_plus: Vec<usize>,
    ix_minus: Vec<usize>,
    iy_plus: Vec<usize>,
    iy_minus: Vec<usize>,

    // D2Q9 constants as the runtime precision type (avoid per-step rebuild).
    cx_r: [R; 9],
    cy_r: [R; 9],
    w_r: [R; 9],

    rng: Xoshiro256PlusPlus,

    time: R,
    step_count: usize,
}

impl<R: Real> Fluid2D<R>
where
    StandardNormal: rand_distr::Distribution<R>,
{
    pub fn new(params: FluidParams) -> Self {
        let nx = params.nx;
        let ny = params.ny;
        let n_cells = nx * ny;

        let n0 = R::from_f64_lossy(params.n0);
        let lam = R::from_f64_lossy(params.lam);
        let t = R::from_f64_lossy(params.t);
        let kappa = R::from_f64_lossy(params.kappa);
        let m_mobility = R::from_f64_lossy(params.m_mobility);
        let tau = R::from_f64_lossy(params.tau);
        let dt = R::from_f64_lossy(params.dt);
        let kt = R::from_f64_lossy(params.kt);
        let clip_frac = R::from_f64_lossy(params.dealias_clip);
        let phi_mean_target = R::from_f64_lossy(params.phi0);
        let phi_noise = R::from_f64_lossy(params.phi_noise);

        let mut rng = Xoshiro256PlusPlus::seed_from_u64(params.seed);

        // Initial phi: phi0 + phi_noise * standard_normal, then project mean → phi0
        let mut phi = vec![R::zero(); n_cells];
        for val in phi.iter_mut() {
            let n: R = rng.sample(StandardNormal);
            *val = phi_mean_target + phi_noise * n;
        }
        let mean: R = phi.iter().copied().fold(R::zero(), |a, b| a + b)
            / R::from_usize(n_cells).unwrap();
        let shift = mean - phi_mean_target;
        for val in phi.iter_mut() {
            *val -= shift;
        }

        let rho = vec![n0; n_cells];
        let ux = vec![R::zero(); n_cells];
        let uy = vec![R::zero(); n_cells];
        let fx = vec![R::zero(); n_cells];
        let fy = vec![R::zero(); n_cells];

        // Initial f: equilibrium at u=0, rho=n0 → f[i] = w[i] * n0 everywhere.
        let w = weights::<R>();
        let f: [Vec<R>; 9] = std::array::from_fn(|i| vec![w[i] * n0; n_cells]);
        let f_next: [Vec<R>; 9] = std::array::from_fn(|_| vec![R::zero(); n_cells]);

        let two_pi = R::from_f64_lossy(2.0 * std::f64::consts::PI);
        let kx_vals: Vec<R> = fftfreq(nx, two_pi);
        let ky_vals: Vec<R> = fftfreq(ny, two_pi);
        let mut k2 = vec![R::zero(); n_cells];
        let mut k4 = vec![R::zero(); n_cells];
        for iy in 0..ny {
            for ix in 0..nx {
                let kx = kx_vals[ix];
                let ky = ky_vals[iy];
                let k2v = kx * kx + ky * ky;
                let idx = iy * nx + ix;
                k2[idx] = k2v;
                k4[idx] = k2v * k2v;
            }
        }

        // FFT planners: separate 1D FFTs along x (length nx) and y (length ny).
        let mut planner: FftPlanner<R> = FftPlanner::new();
        let fft_fwd_x = planner.plan_fft(nx, FftDirection::Forward);
        let fft_inv_x = planner.plan_fft(nx, FftDirection::Inverse);
        let fft_fwd_y = planner.plan_fft(ny, FftDirection::Forward);
        let fft_inv_y = planner.plan_fft(ny, FftDirection::Inverse);
        let fft_buf = vec![Complex::new(R::zero(), R::zero()); n_cells];
        let scratch_len = fft_fwd_x
            .get_inplace_scratch_len()
            .max(fft_inv_x.get_inplace_scratch_len())
            .max(fft_fwd_y.get_inplace_scratch_len())
            .max(fft_inv_y.get_inplace_scratch_len());
        let fft_scratch = vec![Complex::new(R::zero(), R::zero()); scratch_len];
        let fft_col_buf = vec![Complex::new(R::zero(), R::zero()); ny];
        let mu_scratch = vec![R::zero(); n_cells];
        let noise_jx = vec![R::zero(); n_cells];
        let noise_jy = vec![R::zero(); n_cells];

        // Precompute periodic-neighbor tables.
        let ix_plus: Vec<usize> = (0..nx).map(|i| (i + 1) % nx).collect();
        let ix_minus: Vec<usize> = (0..nx).map(|i| (i + nx - 1) % nx).collect();
        let iy_plus: Vec<usize> = (0..ny).map(|i| (i + 1) % ny).collect();
        let iy_minus: Vec<usize> = (0..ny).map(|i| (i + ny - 1) % ny).collect();

        // D2Q9 constants in runtime type.
        let mut cx_r = [R::zero(); 9];
        let mut cy_r = [R::zero(); 9];
        for i in 0..9 {
            cx_r[i] = R::from_i32(CX[i]).unwrap();
            cy_r[i] = R::from_i32(CY[i]).unwrap();
        }
        let w_r = w;

        Self {
            nx,
            ny,
            n_cells,
            n0,
            lam,
            t,
            kappa,
            m_mobility,
            tau,
            dt,
            kt,
            clip_frac,
            hydrodynamics: params.hydrodynamics,
            phi_mean_target,
            phi,
            rho,
            ux,
            uy,
            fx,
            fy,
            f,
            f_next,
            k2,
            k4,
            fft_fwd_x,
            fft_inv_x,
            fft_fwd_y,
            fft_inv_y,
            fft_buf,
            fft_scratch,
            fft_col_buf,
            mu_scratch,
            noise_jx,
            noise_jy,
            ix_plus,
            ix_minus,
            iy_plus,
            iy_minus,
            cx_r,
            cy_r,
            w_r,
            rng,
            time: R::zero(),
            step_count: 0,
        }
    }

    /// Compute fx, fy = -phi * grad(mu), where mu = mu_bulk - kappa * laplacian(phi).
    fn pressure_force(&mut self) {
        let nx = self.nx;
        let ny = self.ny;
        let half = R::from_f64_lossy(0.5);
        let four = R::from_f64_lossy(4.0);

        for iy in 0..ny {
            let row = iy * nx;
            let row_p = self.iy_plus[iy] * nx;
            let row_m = self.iy_minus[iy] * nx;
            for ix in 0..nx {
                let idx = row + ix;
                let phi = self.phi[idx];
                let mb = mu_bulk_point(phi, self.n0, self.t, self.lam, self.clip_frac);
                let lap = self.phi[row + self.ix_plus[ix]]
                    + self.phi[row + self.ix_minus[ix]]
                    + self.phi[row_p + ix]
                    + self.phi[row_m + ix]
                    - four * phi;
                self.mu_scratch[idx] = mb - self.kappa * lap;
            }
        }
        for iy in 0..ny {
            let row = iy * nx;
            let row_p = self.iy_plus[iy] * nx;
            let row_m = self.iy_minus[iy] * nx;
            for ix in 0..nx {
                let idx = row + ix;
                let mux = half
                    * (self.mu_scratch[row + self.ix_plus[ix]]
                        - self.mu_scratch[row + self.ix_minus[ix]]);
                let muy = half * (self.mu_scratch[row_p + ix] - self.mu_scratch[row_m + ix]);
                self.fx[idx] = -self.phi[idx] * mux;
                self.fy[idx] = -self.phi[idx] * muy;
            }
        }
    }

    /// Fused LB step: pressure force, then one pass for moments + collision + push-stream,
    /// then a post-stream moment recomputation that phi_step consumes for its advection term.
    /// The post-stream pass is load-bearing: dropping it slows domain coarsening at deep quench
    /// (measured: T=0.3 saturates at |phi|≈0.86 instead of ≈0.95 after 20k steps).
    fn lb_step(&mut self) {
        if !self.hydrodynamics {
            for v in &mut self.ux {
                *v = R::zero();
            }
            for v in &mut self.uy {
                *v = R::zero();
            }
            for v in &mut self.rho {
                *v = self.n0;
            }
            return;
        }

        self.pressure_force();

        let nx = self.nx;
        let ny = self.ny;
        let half = R::from_f64_lossy(0.5);
        let one = R::one();
        let cs2: R = cs2();
        let inv_cs2 = one / cs2;
        let inv_cs4 = inv_cs2 * inv_cs2;
        let inv_tau = one / self.tau;
        let force_pref = one - half * inv_tau;
        let dt = self.dt;

        let cx_r = self.cx_r;
        let cy_r = self.cy_r;
        let w = self.w_r;

        for iy in 0..ny {
            let row = iy * nx;
            for ix in 0..nx {
                let idx = row + ix;

                let f0 = self.f[0][idx];
                let f1 = self.f[1][idx];
                let f2 = self.f[2][idx];
                let f3 = self.f[3][idx];
                let f4 = self.f[4][idx];
                let f5 = self.f[5][idx];
                let f6 = self.f[6][idx];
                let f7 = self.f[7][idx];
                let f8 = self.f[8][idx];
                let rho = f0 + f1 + f2 + f3 + f4 + f5 + f6 + f7 + f8;
                let jx = f1 - f3 + f5 - f6 - f7 + f8;
                let jy = f2 - f4 + f5 + f6 - f7 - f8;

                let fx = self.fx[idx];
                let fy = self.fy[idx];
                let inv_rho = one / rho;
                let uxv = (jx + half * fx * dt) * inv_rho;
                let uyv = (jy + half * fy * dt) * inv_rho;
                self.rho[idx] = rho;
                self.ux[idx] = uxv;
                self.uy[idx] = uyv;

                let usq = uxv * uxv + uyv * uyv;
                let u_dot_f = uxv * fx + uyv * fy;
                let f_local = [f0, f1, f2, f3, f4, f5, f6, f7, f8];

                let ixp = self.ix_plus[ix];
                let ixm = self.ix_minus[ix];
                let iyp_row = self.iy_plus[iy] * nx;
                let iym_row = self.iy_minus[iy] * nx;
                let dests = [
                    idx,
                    row + ixp,
                    iyp_row + ix,
                    row + ixm,
                    iym_row + ix,
                    iyp_row + ixp,
                    iyp_row + ixm,
                    iym_row + ixm,
                    iym_row + ixp,
                ];

                for i in 0..9 {
                    let cu = cx_r[i] * uxv + cy_r[i] * uyv;
                    let cdotf = cx_r[i] * fx + cy_r[i] * fy;
                    let feq =
                        w[i] * rho * (one + cu * inv_cs2 + half * cu * cu * inv_cs4
                            - half * usq * inv_cs2);
                    let fterm = force_pref * w[i]
                        * ((cdotf - u_dot_f) * inv_cs2 + cu * cdotf * inv_cs4);
                    let post = f_local[i] - (f_local[i] - feq) * inv_tau + dt * fterm;
                    self.f_next[i][dests[i]] = post;
                }
            }
        }

        std::mem::swap(&mut self.f, &mut self.f_next);

        for idx in 0..self.n_cells {
            let f0 = self.f[0][idx];
            let f1 = self.f[1][idx];
            let f2 = self.f[2][idx];
            let f3 = self.f[3][idx];
            let f4 = self.f[4][idx];
            let f5 = self.f[5][idx];
            let f6 = self.f[6][idx];
            let f7 = self.f[7][idx];
            let f8 = self.f[8][idx];
            let rho = f0 + f1 + f2 + f3 + f4 + f5 + f6 + f7 + f8;
            let jx = f1 - f3 + f5 - f6 - f7 + f8;
            let jy = f2 - f4 + f5 + f6 - f7 - f8;
            let fx = self.fx[idx];
            let fy = self.fy[idx];
            let inv_rho = one / rho;
            self.rho[idx] = rho;
            self.ux[idx] = (jx + half * fx * dt) * inv_rho;
            self.uy[idx] = (jy + half * fy * dt) * inv_rho;
        }
    }

    fn fft2_fwd(&mut self) {
        fft2_in_place(
            &mut self.fft_buf,
            self.nx,
            self.ny,
            &self.fft_fwd_x,
            &self.fft_fwd_y,
            &mut self.fft_col_buf,
            &mut self.fft_scratch,
        );
    }

    /// 2D inverse FFT in place on `fft_buf`, including the 1/(nx*ny) normalization.
    fn fft2_inv(&mut self) {
        fft2_in_place(
            &mut self.fft_buf,
            self.nx,
            self.ny,
            &self.fft_inv_x,
            &self.fft_inv_y,
            &mut self.fft_col_buf,
            &mut self.fft_scratch,
        );
        let norm = R::one() / R::from_usize(self.n_cells).unwrap();
        for v in &mut self.fft_buf {
            v.re *= norm;
            v.im *= norm;
        }
    }

    fn phi_step(&mut self) {
        let nx = self.nx;
        let ny = self.ny;
        let half = R::from_f64_lossy(0.5);
        let dt = self.dt;

        for idx in 0..self.n_cells {
            let phi = self.phi[idx];
            let mb = mu_bulk_point(phi, self.n0, self.t, self.lam, self.clip_frac);
            self.fft_buf[idx] = Complex::new(mb, R::zero());
        }
        self.fft2_fwd();
        for idx in 0..self.n_cells {
            let k2 = self.k2[idx];
            self.fft_buf[idx].re *= k2;
            self.fft_buf[idx].im *= k2;
        }
        self.fft2_inv();

        for idx in 0..self.n_cells {
            self.fx[idx] = self.phi[idx] * self.ux[idx];
            self.fy[idx] = self.phi[idx] * self.uy[idx];
        }

        let m = self.m_mobility;
        for iy in 0..ny {
            let row = iy * nx;
            let row_p = self.iy_plus[iy] * nx;
            let row_m = self.iy_minus[iy] * nx;
            for ix in 0..nx {
                let idx = row + ix;
                let ixp = self.ix_plus[ix];
                let ixm = self.ix_minus[ix];
                let adv = half * (self.fx[row + ixp] - self.fx[row + ixm])
                    + half * (self.fy[row_p + ix] - self.fy[row_m + ix]);
                let k2_ifft_mu = self.fft_buf[idx].re;
                let rhs = self.phi[idx] + dt * (-m * k2_ifft_mu - adv);
                self.fft_buf[idx] = Complex::new(rhs, R::zero());
            }
        }

        if self.kt > R::zero() {
            let two = R::from_f64_lossy(2.0);
            let sigma = (two * self.m_mobility * self.kt / self.dt).max(R::zero()).sqrt();
            for v in self.noise_jx.iter_mut() {
                let n: R = self.rng.sample(StandardNormal);
                *v = sigma * n;
            }
            for v in self.noise_jy.iter_mut() {
                let n: R = self.rng.sample(StandardNormal);
                *v = sigma * n;
            }
            for iy in 0..ny {
                let row = iy * nx;
                let row_p = self.iy_plus[iy] * nx;
                let row_m = self.iy_minus[iy] * nx;
                for ix in 0..nx {
                    let idx = row + ix;
                    let ixp = self.ix_plus[ix];
                    let ixm = self.ix_minus[ix];
                    let div = half * (self.noise_jx[row + ixp] - self.noise_jx[row + ixm])
                        + half * (self.noise_jy[row_p + ix] - self.noise_jy[row_m + ix]);
                    self.fft_buf[idx].re += dt * div;
                }
            }
        }

        self.fft2_fwd();
        let one = R::one();
        for idx in 0..self.n_cells {
            let denom = one + dt * m * self.kappa * self.k4[idx];
            // denom[0] would be 1 + 0 = 1 anyway since k4[0] = 0, so no fix-up needed.
            let d = if idx == 0 { one } else { denom };
            self.fft_buf[idx].re = self.fft_buf[idx].re / d;
            self.fft_buf[idx].im = self.fft_buf[idx].im / d;
        }
        self.fft2_inv();

        let mut sum = R::zero();
        for idx in 0..self.n_cells {
            let v = self.fft_buf[idx].re;
            self.phi[idx] = v;
            sum += v;
        }
        let mean = sum / R::from_usize(self.n_cells).unwrap();
        let shift = mean - self.phi_mean_target;
        for v in self.phi.iter_mut() {
            *v -= shift;
        }
    }

    pub fn step(&mut self, n_steps: usize) {
        for _ in 0..n_steps {
            self.lb_step();
            self.phi_step();
            self.time += self.dt;
            self.step_count += 1;
        }
    }

    pub fn phi_mean(&self) -> f64 {
        let sum: R = self.phi.iter().copied().fold(R::zero(), |a, b| a + b);
        let m = sum / R::from_usize(self.n_cells).unwrap();
        m.to_f64_lossy()
    }

    pub fn phi_as_f64(&self) -> Vec<f64> {
        self.phi.iter().map(|v| v.to_f64_lossy()).collect()
    }

    /// Phi as a row-major 2D `Vec<Vec<f64>>` suitable for JSON output.
    pub fn phi_as_2d(&self) -> Vec<Vec<f64>> {
        (0..self.ny)
            .map(|iy| {
                self.phi[iy * self.nx..(iy + 1) * self.nx]
                    .iter()
                    .map(|v| v.to_f64_lossy())
                    .collect()
            })
            .collect()
    }

    pub fn phi_vec(&self) -> &[R] {
        &self.phi
    }

    pub fn time_f64(&self) -> f64 {
        self.time.to_f64_lossy()
    }

    pub fn step_count(&self) -> usize {
        self.step_count
    }

    pub fn nx(&self) -> usize {
        self.nx
    }

    pub fn ny(&self) -> usize {
        self.ny
    }

}

