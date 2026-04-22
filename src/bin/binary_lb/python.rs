use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use statphys::lb::RunHistory;
use statphys::write_json;

use super::{MetastableArgs, SpinodalArgs, Task};

fn import_lb<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
    let sys = py.import("sys")?;
    sys.getattr("path")?
        .call_method1("insert", (0, "exercises/exercise-5"))?;
    py.import("binary_LB")
}

fn call_ctor<'py>(
    lb: &Bound<'py, PyModule>,
    ctor: &str,
    kwargs: &Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyAny>> {
    lb.call_method(ctor, (), Some(kwargs))
}

fn call_run_and_collect<'py>(
    lb: &Bound<'py, PyModule>,
    sim: &Bound<'py, PyAny>,
    steps: usize,
    snapshot_every: usize,
) -> PyResult<Bound<'py, PyAny>> {
    let kwargs = PyDict::new(lb.py());
    kwargs.set_item("steps", steps)?;
    kwargs.set_item("snapshot_every", snapshot_every)?;
    kwargs.set_item("include_velocity", false)?;
    lb.call_method("run_and_collect", (sim,), Some(&kwargs))
}

fn dump_history(history: &Bound<'_, PyAny>, out_path: &str) -> PyResult<()> {
    let times: Vec<f64> = history.get_item("times")?.extract()?;
    let steps: Vec<usize> = history.get_item("steps")?.extract()?;
    let phi_list: Bound<'_, PyList> = history.get_item("phi")?.downcast_into()?;
    let mut phi: Vec<Vec<Vec<f64>>> = Vec::with_capacity(phi_list.len());
    for frame in phi_list.iter() {
        phi.push(frame.call_method0("tolist")?.extract()?);
    }
    let phi_mean: Vec<f64> = history.get_item("phi_mean")?.extract()?;
    let params: serde_json::Value = {
        let p = history.get_item("params")?;
        let json_str: String = p.py().import("json")?.call_method1("dumps", (p,))?.extract()?;
        serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null)
    };
    let tc: f64 = history.get_item("Tc")?.extract()?;
    let phi_binodal: f64 = history.get_item("phi_binodal")?.extract()?;
    let phi_spinodal: f64 = history.get_item("phi_spinodal")?.extract()?;

    write_json(
        out_path,
        &RunHistory { times, steps, phi, phi_mean, params, tc, phi_binodal, phi_spinodal },
    );
    Ok(())
}

fn spinodal(py: Python<'_>, args: &SpinodalArgs) -> PyResult<()> {
    let lb = import_lb(py)?;
    let kwargs = PyDict::new(py);
    kwargs.set_item("Nx", args.nx)?;
    kwargs.set_item("Ny", args.ny)?;
    kwargs.set_item("n0", args.n0)?;
    kwargs.set_item("lam", args.lam)?;
    kwargs.set_item("T", args.temperature)?;
    kwargs.set_item("kappa", args.kappa)?;
    kwargs.set_item("M", args.mobility)?;
    kwargs.set_item("tau", args.tau)?;
    kwargs.set_item("dt", args.dt)?;
    kwargs.set_item("spinodal_fraction", args.spinodal_fraction)?;
    kwargs.set_item("phi_noise", args.phi_noise)?;
    kwargs.set_item("hydrodynamics", !args.no_hydrodynamics)?;
    kwargs.set_item("seed", args.seed)?;
    let sim = call_ctor(&lb, "make_spinodal_example", &kwargs)?;
    let history = call_run_and_collect(&lb, &sim, args.steps, args.snapshot_every)?;
    let out_path = args
        .output
        .clone()
        .unwrap_or_else(|| "data/binary_lb/spinodal_python.json".to_string());
    dump_history(&history, &out_path)
}

fn metastable(py: Python<'_>, args: &MetastableArgs) -> PyResult<()> {
    let lb = import_lb(py)?;
    let kwargs = PyDict::new(py);
    kwargs.set_item("Nx", args.nx)?;
    kwargs.set_item("Ny", args.ny)?;
    kwargs.set_item("n0", args.n0)?;
    kwargs.set_item("lam", args.lam)?;
    kwargs.set_item("T", args.temperature)?;
    kwargs.set_item("kappa", args.kappa)?;
    kwargs.set_item("M", args.mobility)?;
    kwargs.set_item("tau", args.tau)?;
    kwargs.set_item("dt", args.dt)?;
    kwargs.set_item("fraction_of_binodal", args.fraction_of_binodal)?;
    kwargs.set_item("phi_noise", args.phi_noise)?;
    kwargs.set_item("kT", args.kt)?;
    kwargs.set_item("hydrodynamics", !args.no_hydrodynamics)?;
    kwargs.set_item("seed", args.seed)?;
    let sim = call_ctor(&lb, "make_metastable_example", &kwargs)?;
    let history = call_run_and_collect(&lb, &sim, args.steps, args.snapshot_every)?;
    let out_path = args
        .output
        .clone()
        .unwrap_or_else(|| "data/binary_lb/metastable_python.json".to_string());
    dump_history(&history, &out_path)
}

pub fn run(task: &Task) -> PyResult<()> {
    Python::with_gil(|py| match task {
        Task::Spinodal(args) => {
            eprintln!("=== Spinodal (Python) ===");
            spinodal(py, args)
        }
        Task::Metastable(args) => {
            eprintln!("=== Metastable (Python) ===");
            metastable(py, args)
        }
        Task::Bench { .. } => Err(pyo3::exceptions::PyRuntimeError::new_err(
            "Bench is only supported by the Rust backend",
        )),
    })
}
