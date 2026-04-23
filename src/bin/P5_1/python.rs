use super::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use rand::RngCore;
use statphys::write_json;

// --- Python bridge ----------------------------------------------------------

struct Bridge<'py> {
    lb: Bound<'py, PyModule>,
    analysis: std::cell::OnceCell<Bound<'py, PyModule>>,
}

impl<'py> Bridge<'py> {
    /// Lazy-loaded `analysis` module. Not every task needs scipy/analysis, so
    /// don't pay the import cost (and scipy dependency) until the first caller asks.
    fn analysis(&self) -> PyResult<&Bound<'py, PyModule>> {
        if let Some(m) = self.analysis.get() {
            return Ok(m);
        }
        let m = self.lb.py().import("analysis")?;
        let _ = self.analysis.set(m);
        Ok(self.analysis.get().unwrap())
    }
}

fn setup<'py>(py: Python<'py>) -> PyResult<Bridge<'py>> {
    let sys = py.import("sys")?;
    sys.getattr("path")?
        .call_method1("insert", (0, "exercises/exercise-5"))?;
    Ok(Bridge {
        lb: py.import("binary_LB")?,
        analysis: std::cell::OnceCell::new(),
    })
}

fn call_make_spinodal<'py>(
    bridge: &Bridge<'py>,
    kwargs: &Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyAny>> {
    bridge
        .lb
        .call_method("make_spinodal_example", (), Some(kwargs))
}

fn call_make_metastable<'py>(
    bridge: &Bridge<'py>,
    kwargs: &Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyAny>> {
    bridge
        .lb
        .call_method("make_metastable_example", (), Some(kwargs))
}

fn run_and_collect<'py>(
    bridge: &Bridge<'py>,
    sim: &Bound<'py, PyAny>,
    steps: usize,
    snapshot_every: usize,
    include_velocity: bool,
) -> PyResult<Bound<'py, PyAny>> {
    let kwargs = PyDict::new(bridge.lb.py());
    kwargs.set_item("steps", steps)?;
    kwargs.set_item("snapshot_every", snapshot_every)?;
    kwargs.set_item("include_velocity", include_velocity)?;
    bridge
        .lb
        .call_method("run_and_collect", (sim,), Some(&kwargs))
}

/// Pull the final phi frame out of a history dict as `Vec<Vec<f64>>`.
fn extract_final_phi(history: &Bound<'_, PyAny>) -> PyResult<Vec<Vec<f64>>> {
    let phi_list: Bound<'_, PyList> =
        history.get_item("phi")?.downcast_into()?;
    let last = phi_list.get_item(phi_list.len() - 1)?;
    // numpy array -> nested list via tolist()
    last.call_method0("tolist")?.extract()
}

fn extract_times(history: &Bound<'_, PyAny>) -> PyResult<Vec<f64>> {
    history.get_item("times")?.extract()
}

// --- Task 1a: temperature sweep ---------------------------------------------

fn temperature_sweep(bridge: &Bridge<'_>, steps: usize, output: &str) -> PyResult<()> {
    let py = bridge.lb.py();
    let mut snapshots = Vec::new();

    for &t in &TASK1_TEMPERATURES {
        let kwargs = PyDict::new(py);
        kwargs.set_item("lam", TASK1_LAM)?;
        kwargs.set_item("T", t)?;
        kwargs.set_item("dt", TASK1_DT)?;
        kwargs.set_item("spinodal_fraction", TASK1_SPINODAL_FRACTION)?;
        kwargs.set_item("seed", 1)?;
        let sim = call_make_spinodal(bridge, &kwargs)?;
        let history = run_and_collect(bridge, &sim, steps, SNAPSHOT_EVERY, false)?;

        snapshots.push(SnapshotOutput {
            label: format!("T={t}"),
            phi_final: extract_final_phi(&history)?,
            params: serde_json::json!({
                "T": t,
                "lam": TASK1_LAM,
                "dt": TASK1_DT,
                "spinodal_fraction": TASK1_SPINODAL_FRACTION,
                "steps": steps,
            }),
        });
        println!("  T={t}: done");
    }

    write_json(
        &format!("data/P5_1/{output}.json"),
        &SnapshotCollection { snapshots },
    );
    Ok(())
}

fn temperatures(bridge: &Bridge<'_>) -> PyResult<()> {
    println!("  [short sweep, {} steps]", TASK1_STEPS);
    temperature_sweep(bridge, TASK1_STEPS, "temperatures")?;
    println!("  [long sweep, {} steps]", TASK1_STEPS_LONG);
    temperature_sweep(bridge, TASK1_STEPS_LONG, "temperatures_long")
}

// --- Task 1b: dt sweep ------------------------------------------------------

fn timesteps(bridge: &Bridge<'_>) -> PyResult<()> {
    let py = bridge.lb.py();
    let mut snapshots = Vec::new();

    for &dt in &TASK1B_DT {
        let kwargs = PyDict::new(py);
        kwargs.set_item("lam", TASK1_LAM)?;
        kwargs.set_item("T", TASK1B_T)?;
        kwargs.set_item("dt", dt)?;
        kwargs.set_item("spinodal_fraction", TASK1_SPINODAL_FRACTION)?;
        kwargs.set_item("seed", 1)?;
        let sim = call_make_spinodal(bridge, &kwargs)?;
        let history = run_and_collect(bridge, &sim, TASK1B_STEPS, SNAPSHOT_EVERY, false)?;

        snapshots.push(SnapshotOutput {
            label: format!("dt={dt}"),
            phi_final: extract_final_phi(&history)?,
            params: serde_json::json!({ "T": TASK1B_T, "dt": dt }),
        });
        println!("  dt={dt}: done");
    }

    write_json(
        "data/P5_1/timesteps.json",
        &SnapshotCollection { snapshots },
    );
    Ok(())
}

// --- Task 1c: asymmetric runs -----------------------------------------------

fn asymmetric(bridge: &Bridge<'_>) -> PyResult<()> {
    let py = bridge.lb.py();
    let mut snapshots = Vec::new();

    for &frac in &TASK1C_FRACTIONS {
        let kwargs = PyDict::new(py);
        kwargs.set_item("lam", TASK1_LAM)?;
        kwargs.set_item("T", TASK1C_T)?;
        kwargs.set_item("dt", TASK1_DT)?;
        kwargs.set_item("spinodal_fraction", frac)?;
        kwargs.set_item("seed", 1)?;
        let sim = call_make_spinodal(bridge, &kwargs)?;
        let history = run_and_collect(bridge, &sim, TASK1C_STEPS, SNAPSHOT_EVERY, false)?;

        snapshots.push(SnapshotOutput {
            label: format!("sfrac={frac}"),
            phi_final: extract_final_phi(&history)?,
            params: serde_json::json!({ "T": TASK1C_T, "spinodal_fraction": frac }),
        });
        println!("  sfrac={frac}: done");
    }

    write_json(
        "data/P5_1/asymmetric.json",
        &SnapshotCollection { snapshots },
    );
    Ok(())
}

// --- Task 2a: domain growth L(t) --------------------------------------------

fn domain_growth(bridge: &Bridge<'_>) -> PyResult<()> {
    let py = bridge.lb.py();
    let mut curves = Vec::new();

    for &tau in &TASK2_TAU {
        let kwargs = PyDict::new(py);
        kwargs.set_item("lam", TASK1_LAM)?;
        kwargs.set_item("T", TASK2_T)?;
        kwargs.set_item("M", TASK2_M)?;
        kwargs.set_item("tau", tau)?;
        kwargs.set_item("dt", TASK2_DT)?;
        kwargs.set_item("spinodal_fraction", 0.5)?;
        kwargs.set_item("hydrodynamics", true)?;
        kwargs.set_item("seed", 1)?;
        let sim = call_make_spinodal(bridge, &kwargs)?;
        let history = run_and_collect(bridge, &sim, TASK2_STEPS, SNAPSHOT_EVERY, true)?;

        let l_of_t: Vec<f64> = bridge.analysis()?.call_method1("compute_L_series", (&history,))?
            .extract()?;
        let times = extract_times(&history)?;

        curves.push(DomainGrowthCurve { tau, times, l_of_t });
        println!("  tau={tau}: done");
    }

    write_json(
        "data/P5_1/domain_growth.json",
        &DomainGrowthOutput { curves },
    );
    Ok(())
}

// --- Task 3: nucleation vs spinodal -----------------------------------------

fn nucleation(bridge: &Bridge<'_>) -> PyResult<()> {
    let py = bridge.lb.py();
    let mut curves = Vec::new();

    let mut rng = rand::rng();
    for (idx, &t) in TASK3_TEMPERATURES.iter().enumerate() {
        let seed: u64 = rng.next_u64();
        let kwargs = PyDict::new(py);
        kwargs.set_item("lam", TASK1_LAM)?;
        kwargs.set_item("T", t)?;
        kwargs.set_item("kappa", TASK3_KAPPA)?;
        kwargs.set_item("tau", TASK3_TAU)?;
        kwargs.set_item("dt", TASK3_DT)?;
        kwargs.set_item("M", TASK3_M)?;
        kwargs.set_item("kT", TASK3_KT)?;
        kwargs.set_item("fraction_of_binodal", TASK3_FRACTION_BINODAL)?;
        kwargs.set_item("hydrodynamics", false)?;
        kwargs.set_item("seed", seed)?;
        let sim = call_make_metastable(bridge, &kwargs)?;
        let history = run_and_collect(bridge, &sim, TASK3_STEPS, SNAPSHOT_EVERY, false)?;

        let largest: Vec<i64> = bridge.analysis()?.call_method1("largest_cluster_series", (&history,))?
            .extract()?;
        let minority: Vec<i64> = bridge.analysis()?.call_method1("minority_count_series", (&history,))?
            .extract()?;
        let threshold: f64 = history.get_item("phi_binodal")?.extract()?;
        let times = extract_times(&history)?;

        curves.push(NucleationCurve {
            label: format!("run{}_T={t}_seed={seed}", idx + 1),
            temperature: t,
            threshold,
            seed,
            times,
            largest_cluster: largest,
            minority_count: minority,
        });
        println!("  run {idx}: T={t} seed={seed} done");
    }

    write_json(
        "data/P5_1/nucleation.json",
        &NucleationOutput { curves },
    );
    Ok(())
}

// --- Dispatcher -------------------------------------------------------------

pub fn run_tasks(task: &Task) -> PyResult<()> {
    Python::with_gil(|py| {
        let bridge = setup(py)?;

        match task {
            Task::Temperatures => {
                println!("=== Task 1a: Temperature sweep ===");
                temperatures(&bridge)?;
            }
            Task::Timesteps => {
                println!("=== Task 1b: Timestep sweep ===");
                timesteps(&bridge)?;
            }
            Task::Asymmetric => {
                println!("=== Task 1c: Asymmetric fractions ===");
                asymmetric(&bridge)?;
            }
            Task::DomainGrowth => {
                println!("=== Task 2a: Domain growth L(t) ===");
                domain_growth(&bridge)?;
            }
            Task::Nucleation | Task::MinorityCount => {
                // 3a and 3b share the same runs; we always compute both.
                println!("=== Task 3: Nucleation vs spinodal ===");
                nucleation(&bridge)?;
            }
            Task::Bench { .. } => {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Bench is only supported by the Rust backend",
                ));
            }
            Task::All => {
                println!("=== Task 1a ===");
                temperatures(&bridge)?;
                println!("\n=== Task 1b ===");
                timesteps(&bridge)?;
                println!("\n=== Task 1c ===");
                asymmetric(&bridge)?;
                println!("\n=== Task 2a ===");
                domain_growth(&bridge)?;
                println!("\n=== Task 3 ===");
                nucleation(&bridge)?;
            }
        }

        Ok(())
    })
}
