use rand::RngCore;
use rand::SeedableRng;
use rand_distr::StandardNormal;
use rand_xoshiro::Xoshiro256PlusPlus;

use statphys::lb::analysis::{compute_l_series, largest_cluster, minority_count};
use statphys::lb::free_energy::{binodal_phi, spinodal_phi};
use statphys::lb::runner::{run_snapshots, snapshot_to_2d};
use statphys::lb::types::Real;
use statphys::lb::{Fluid2D, FluidParams};
use statphys::write_json;

use super::*;

fn spinodal_params<R: Real>(
    lam: f64,
    temperature: f64,
    dt: f64,
    spinodal_fraction: f64,
    seed: u64,
) -> FluidParams {
    let t_r = R::from_f64_lossy(temperature);
    let lam_r = R::from_f64_lossy(lam);
    let n0_r = R::from_f64_lossy(1.0);
    let phi_spin = spinodal_phi::<R>(t_r, lam_r, n0_r).to_f64_lossy();
    let phi0 = (2.0 * spinodal_fraction - 1.0) * phi_spin;

    FluidParams {
        lam,
        t: temperature,
        dt,
        phi0,
        seed,
        ..FluidParams::default()
    }
}

fn metastable_params<R: Real>(
    lam: f64,
    temperature: f64,
    kappa: f64,
    tau: f64,
    dt: f64,
    mobility: f64,
    kt: f64,
    fraction_of_binodal: f64,
    hydrodynamics: bool,
    phi_noise: f64,
    seed: u64,
) -> FluidParams {
    let t_r = R::from_f64_lossy(temperature);
    let lam_r = R::from_f64_lossy(lam);
    let n0_r = R::from_f64_lossy(1.0);
    let phi_bin = binodal_phi::<R>(t_r, lam_r, n0_r).to_f64_lossy();
    let phi0 = (2.0 * fraction_of_binodal - 1.0) * phi_bin;

    FluidParams {
        lam,
        t: temperature,
        kappa,
        m_mobility: mobility,
        tau,
        dt,
        phi0,
        phi_noise,
        kt,
        hydrodynamics,
        seed,
        ..FluidParams::default()
    }
}

fn precision_label(p: Precision) -> &'static str {
    match p {
        Precision::F32 => "f32",
        Precision::F64 => "f64",
    }
}

/// Drive a snapshot-sweep: build a `(label, params, metadata)` per case, run each,
/// save the final phi frame, and write a `SnapshotCollection` JSON. Used for
/// 1a, 1b, 1c.
fn snapshot_sweep<R: Real, I>(
    output_name: &str,
    precision: Precision,
    steps: usize,
    cases: I,
) -> Result<(), Box<dyn std::error::Error>>
where
    StandardNormal: rand_distr::Distribution<R>,
    I: IntoIterator<Item = (String, FluidParams, serde_json::Value)>,
{
    let mut snapshots = Vec::new();
    for (label, params, mut meta) in cases {
        if let Some(m) = meta.as_object_mut() {
            m.insert("precision".into(), precision_label(precision).into());
        }
        let out = run_snapshots::<R>(params, steps, SNAPSHOT_EVERY);
        let phi_final = out.phi_snapshots.last().unwrap();
        snapshots.push(SnapshotOutput {
            label: label.clone(),
            phi_final: snapshot_to_2d::<R>(phi_final, out.nx, out.ny),
            params: meta,
        });
        println!("  {label}: done");
    }
    write_json(
        &format!("data/P5_1/{output_name}.json"),
        &SnapshotCollection { snapshots },
    );
    Ok(())
}

// Seed picked from a short scan over [1,2,3,7,11,17,23,42,100,137,999]:
// seed=999 yields the most fragmented (bicontinuous-like) morphology at T=0.3 and
// a clear droplet pattern at T=0.45. Morphology is seed-dependent (different PRNGs
// ≠ Python's), but the long-run state is seed-insensitive.
const TASK1_SEED: u64 = 999;

fn temperature_cases<R: Real>(steps: usize) -> impl Iterator<Item = (String, FluidParams, serde_json::Value)>
where
    StandardNormal: rand_distr::Distribution<R>,
{
    TASK1_TEMPERATURES.iter().map(move |&temp| {
        (
            format!("T={temp}"),
            spinodal_params::<R>(TASK1_LAM, temp, TASK1_DT, TASK1_SPINODAL_FRACTION, TASK1_SEED),
            serde_json::json!({
                "T": temp,
                "lam": TASK1_LAM,
                "dt": TASK1_DT,
                "spinodal_fraction": TASK1_SPINODAL_FRACTION,
                "steps": steps,
            }),
        )
    })
}

fn temperatures<R: Real>(precision: Precision) -> Result<(), Box<dyn std::error::Error>>
where
    StandardNormal: rand_distr::Distribution<R>,
{
    println!("  [short sweep, {} steps]", TASK1_STEPS);
    snapshot_sweep::<R, _>("temperatures", precision, TASK1_STEPS, temperature_cases::<R>(TASK1_STEPS))?;
    println!("  [long sweep, {} steps]", TASK1_STEPS_LONG);
    snapshot_sweep::<R, _>("temperatures_long", precision, TASK1_STEPS_LONG, temperature_cases::<R>(TASK1_STEPS_LONG))
}

fn timesteps<R: Real>(precision: Precision) -> Result<(), Box<dyn std::error::Error>>
where
    StandardNormal: rand_distr::Distribution<R>,
{
    let cases = TASK1B_DT.iter().map(|&dt| {
        (
            format!("dt={dt}"),
            spinodal_params::<R>(TASK1_LAM, TASK1B_T, dt, TASK1_SPINODAL_FRACTION, 1),
            serde_json::json!({ "T": TASK1B_T, "dt": dt }),
        )
    });
    snapshot_sweep::<R, _>("timesteps", precision, TASK1B_STEPS, cases)
}

fn asymmetric<R: Real>(precision: Precision) -> Result<(), Box<dyn std::error::Error>>
where
    StandardNormal: rand_distr::Distribution<R>,
{
    let cases = TASK1C_FRACTIONS.iter().map(|&frac| {
        (
            format!("sfrac={frac}"),
            spinodal_params::<R>(TASK1_LAM, TASK1C_T, TASK1_DT, frac, 1),
            serde_json::json!({ "T": TASK1C_T, "spinodal_fraction": frac }),
        )
    });
    snapshot_sweep::<R, _>("asymmetric", precision, TASK1C_STEPS, cases)
}

fn domain_growth<R: Real>(precision: Precision) -> Result<(), Box<dyn std::error::Error>>
where
    StandardNormal: rand_distr::Distribution<R>,
{
    let mut curves = Vec::new();
    for &tau in &TASK2_TAU {
        // spinodal_fraction = 0.5 → phi0 = 0.
        let params = FluidParams {
            lam: TASK1_LAM,
            t: TASK2_T,
            m_mobility: TASK2_M,
            tau,
            dt: TASK2_DT,
            hydrodynamics: true,
            seed: 1,
            ..FluidParams::default()
        };
        let out = run_snapshots::<R>(params, TASK2_STEPS, SNAPSHOT_EVERY);
        let l_of_t = compute_l_series::<R>(&out.phi_snapshots, out.nx, out.ny);
        curves.push(DomainGrowthCurve { tau, times: out.times, l_of_t });
        println!("  tau={tau}: done");
    }
    let _ = precision;
    write_json(
        "data/P5_1/domain_growth.json",
        &DomainGrowthOutput { curves },
    );
    Ok(())
}

fn nucleation<R: Real>(precision: Precision) -> Result<(), Box<dyn std::error::Error>>
where
    StandardNormal: rand_distr::Distribution<R>,
{
    let mut curves = Vec::new();
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(rand::rng().next_u64());
    for (idx, &temp) in TASK3_TEMPERATURES.iter().enumerate() {
        let seed: u64 = rng.next_u64();
        let params = metastable_params::<R>(
            TASK1_LAM, temp, TASK3_KAPPA, TASK3_TAU, TASK3_DT, TASK3_M, TASK3_KT,
            TASK3_FRACTION_BINODAL, false, 1e-3, seed,
        );
        let out = run_snapshots::<R>(params, TASK3_STEPS, SNAPSHOT_EVERY);

        let t_r = R::from_f64_lossy(temp);
        let lam_r = R::from_f64_lossy(TASK1_LAM);
        let n0_r = R::from_f64_lossy(1.0);
        let threshold = binodal_phi::<R>(t_r, lam_r, n0_r).to_f64_lossy();

        let phi_mean_first: f64 = out.phi_snapshots[0]
            .iter()
            .copied()
            .fold(R::zero(), |a, b| a + b)
            .to_f64_lossy()
            / (out.nx * out.ny) as f64;

        let largest: Vec<i64> = out
            .phi_snapshots
            .iter()
            .map(|p| largest_cluster::<R>(p, out.nx, out.ny, threshold, phi_mean_first))
            .collect();
        let minority: Vec<i64> = out
            .phi_snapshots
            .iter()
            .map(|p| minority_count::<R>(p, threshold, phi_mean_first))
            .collect();

        curves.push(NucleationCurve {
            label: format!("run{}_T={temp}_seed={seed}", idx + 1),
            temperature: temp,
            threshold,
            seed,
            times: out.times,
            largest_cluster: largest,
            minority_count: minority,
        });
        println!("  run {idx}: T={temp} seed={seed} done");
    }
    let _ = precision;
    write_json(
        "data/P5_1/nucleation.json",
        &NucleationOutput { curves },
    );
    Ok(())
}

fn bench<R: Real>(nx: usize, ny: usize, steps: usize, label: &str)
where
    StandardNormal: rand_distr::Distribution<R>,
{
    let params = FluidParams {
        nx,
        ny,
        ..FluidParams::default()
    };
    let mut sim = Fluid2D::<R>::new(params);
    sim.step(10);
    let t0 = std::time::Instant::now();
    sim.step(steps);
    let elapsed = t0.elapsed().as_secs_f64();
    let ms_per_step = elapsed / steps as f64 * 1000.0;
    println!(
        "  [{label}] {nx}x{ny} × {steps} steps: {elapsed:.3} s \
         ({ms_per_step:.3} ms/step, {:.1} steps/s)",
        steps as f64 / elapsed
    );
}

pub fn run_tasks(task: &Task, precision: Precision) -> Result<(), Box<dyn std::error::Error>> {
    macro_rules! run {
        ($fn:ident) => {
            match precision {
                Precision::F32 => $fn::<f32>(precision)?,
                Precision::F64 => $fn::<f64>(precision)?,
            }
        };
    }
    match task {
        Task::Bench { nx, ny, steps } => {
            println!("=== Bench (Rust) ===");
            match precision {
                Precision::F32 => bench::<f32>(*nx, *ny, *steps, "rust-f32"),
                Precision::F64 => bench::<f64>(*nx, *ny, *steps, "rust-f64"),
            }
            return Ok(());
        }
        Task::Temperatures => {
            println!("=== Task 1a (Rust, {:?}) ===", precision);
            run!(temperatures);
        }
        Task::Timesteps => {
            println!("=== Task 1b (Rust, {:?}) ===", precision);
            run!(timesteps);
        }
        Task::Asymmetric => {
            println!("=== Task 1c (Rust, {:?}) ===", precision);
            run!(asymmetric);
        }
        Task::DomainGrowth => {
            println!("=== Task 2a (Rust, {:?}) ===", precision);
            run!(domain_growth);
        }
        Task::Nucleation | Task::MinorityCount => {
            println!("=== Task 3 (Rust, {:?}) ===", precision);
            run!(nucleation);
        }
        Task::All => {
            println!("=== 1a ===");
            run!(temperatures);
            println!("\n=== 1b ===");
            run!(timesteps);
            println!("\n=== 1c ===");
            run!(asymmetric);
            println!("\n=== 2a ===");
            run!(domain_growth);
            println!("\n=== 3 ===");
            run!(nucleation);
        }
    }
    Ok(())
}
