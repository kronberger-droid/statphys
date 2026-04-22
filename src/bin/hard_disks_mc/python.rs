use pyo3::prelude::*;
use pyo3::types::PyDict;

use super::{Cli, SimulationResult};

pub fn run(cli: &Cli) -> PyResult<SimulationResult> {
    Python::with_gil(|py| {
        let sys = py.import("sys")?;
        sys.getattr("path")?
            .call_method1("insert", (0, "exercises/exercise-4/MC"))?;
        let hsmc = py.import("hard_disks_mc")?;

        let kwargs = PyDict::new(py);
        kwargs.set_item("n_particles", cli.n_particles)?;
        kwargs.set_item("t_end", cli.t_end)?;
        kwargs.set_item("temperature", cli.temperature)?;
        kwargs.set_item("box_length", cli.box_length)?;
        if let Some(p) = cli.pressure {
            kwargs.set_item("pressure", p)?;
        }
        kwargs.set_item("sigma", cli.sigma)?;
        kwargs.set_item("epsilon", cli.epsilon)?;
        kwargs.set_item("max_displacement", cli.max_displacement)?;
        kwargs.set_item("max_delta_log_area", cli.max_delta_log_area)?;
        kwargs.set_item("ensemble", cli.ensemble.to_string())?;
        kwargs.set_item("save_every", cli.save_every)?;
        kwargs.set_item("seed", cli.seed)?;
        kwargs.set_item("initialization", cli.initialization.to_string())?;

        let result = hsmc.call_method("run_simulation", (), Some(&kwargs))?;

        let positions: Vec<Vec<f64>> = result
            .getattr("positions")?
            .call_method0("tolist")?
            .extract()?;
        let trajectory: Vec<Vec<Vec<f64>>> = result
            .getattr("trajectory")?
            .call_method0("tolist")?
            .extract()?;
        let box_lengths: Vec<f64> = result.getattr("box_lengths")?.extract()?;
        let energies: Vec<f64> = result.getattr("energies")?.extract()?;
        let saved_sweeps: Vec<usize> = result.getattr("saved_sweeps")?.extract()?;
        let move_acceptance: f64 = result.getattr("move_acceptance")?.extract()?;
        let volume_acceptance: f64 = result.getattr("volume_acceptance")?.extract()?;
        let metadata: serde_json::Value = {
            // Python `MonteCarloResult.metadata` can contain NaN (e.g. pressure in NVT);
            // json.dumps emits `NaN` which serde_json rejects. Replace NaN → None first.
            let helper = r#"
def _sanitize(x):
    import math
    if isinstance(x, dict):
        return {k: _sanitize(v) for k, v in x.items()}
    if isinstance(x, float) and math.isnan(x):
        return None
    return x
"#;
            let globals = PyDict::new(py);
            py.run(&std::ffi::CString::new(helper).unwrap(), Some(&globals), None)?;
            let sanitize = globals.get_item("_sanitize")?.unwrap();
            let m = result.getattr("metadata")?;
            let cleaned = sanitize.call1((m,))?;
            let s: String = py.import("json")?.call_method1("dumps", (cleaned,))?.extract()?;
            serde_json::from_str(&s).unwrap_or(serde_json::Value::Null)
        };

        let to_pairs =
            |v: Vec<Vec<f64>>| v.into_iter().map(|p| [p[0], p[1]]).collect::<Vec<[f64; 2]>>();
        Ok(SimulationResult {
            positions: to_pairs(positions),
            trajectory: trajectory.into_iter().map(to_pairs).collect(),
            box_lengths,
            energies,
            saved_sweeps,
            move_acceptance,
            volume_acceptance,
            metadata,
        })
    })
}
