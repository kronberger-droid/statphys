use super::*;
use pyo3::prelude::*;
use pyo3::types::PyDict;

fn import_hsmc<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
    let sys = py.import("sys")?;
    sys.getattr("path")?
        .call_method1("insert", (0, "exercises/exercise-4/MC"))?;
    py.import("hard_disks_mc")
}

fn timing(hsmc: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = hsmc.py();
    let ensembles = ["NVT", "NPT"];

    let mut points = Vec::new();

    for &ensemble in &ensembles {
        for &n in &TIMING_PARTICLES {
            let box_length = (n as f64 / TIMING_DENSITY).sqrt();
            let mut times = Vec::new();

            for run in 0..TIMING_N_RUNS {
                let kwargs = PyDict::new(py);
                kwargs.set_item("n_particles", n)?;
                kwargs.set_item("t_end", TIMING_T_END)?;
                kwargs.set_item("box_length", box_length)?;
                kwargs.set_item("ensemble", ensemble)?;
                kwargs.set_item("save_every", TIMING_T_END)?;
                kwargs.set_item("seed", run)?;
                if matches!(ensemble, "NPT") {
                    kwargs.set_item("pressure", TIMING_PRESSURE)?;
                };

                let start = std::time::Instant::now();
                hsmc.call_method("run_simulation", (), Some(&kwargs))?;
                let elapsed = start.elapsed().as_secs_f64();

                times.push(elapsed);
            }

            points.push(TimingPoint {
                n_particles: n,
                ensemble: ensemble.to_string(),
                times_s: times,
            });

            println!("  {ensemble} N={n}: done ({} runs)", TIMING_N_RUNS);
        }
    }

    let output = TimingOutput {
        t_end: TIMING_T_END,
        n_runs: TIMING_N_RUNS,
        density: TIMING_DENSITY,
        points,
    };

    write_json("exercises/exercise-4/data/P4_1/timing_python.json", &output);
    Ok(())
}

fn acceptance(hsmc: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = hsmc.py();

    let mut points = Vec::new();

    for &density in &ACCEPTANCE_DENSITIES {
        let box_length = (ACCEPTANCE_N_PARTICLES as f64 / density).sqrt();

        for &max_disp in &ACCEPTANCE_DISPLACEMENTS {
            let kwargs = PyDict::new(py);
            kwargs.set_item("n_particles", ACCEPTANCE_N_PARTICLES)?;
            kwargs.set_item("t_end", ACCEPTANCE_T_END)?;
            kwargs.set_item("box_length", box_length)?;
            kwargs.set_item("sigma", ACCEPTANCE_SIGMA)?;
            kwargs.set_item("max_displacement", max_disp)?;
            kwargs.set_item("ensemble", "NVT")?;
            kwargs.set_item("save_every", ACCEPTANCE_T_END)?;
            kwargs.set_item("seed", ACCEPTANCE_SEED)?;
            let result =
                hsmc.call_method("run_simulation", (), Some(&kwargs))?;

            let acceptance: f64 =
                result.getattr("move_acceptance")?.extract()?;

            points.push(AcceptancePoint {
                density,
                max_displacement: max_disp,
                acceptance_rate: acceptance,
            });
        }

        println!("  density={density}: done");
    }

    let output = AcceptanceOutput {
        n_particles: ACCEPTANCE_N_PARTICLES,
        sigma: ACCEPTANCE_SIGMA,
        t_end: ACCEPTANCE_T_END,
        points,
    };

    write_json("exercises/exercise-4/data/P4_1/acceptance_python.json", &output);
    Ok(())
}

fn packing(hsmc: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = hsmc.py();

    let v_particle =
        std::f64::consts::PI * (PACKING_SIGMA / 2.0) * (PACKING_SIGMA / 2.0);

    let mut curves = Vec::new();

    for &pressure in &PACKING_PRESSURES {
        let kwargs = PyDict::new(py);
        kwargs.set_item("n_particles", PACKING_N_PARTICLES)?;
        kwargs.set_item("t_end", PACKING_T_END)?;
        kwargs.set_item("box_length", PACKING_INITIAL_BOX_LENGTH)?;
        kwargs.set_item("sigma", PACKING_SIGMA)?;
        kwargs.set_item("ensemble", "NPT")?;
        kwargs.set_item("pressure", pressure)?;
        kwargs.set_item("save_every", 1)?;
        kwargs.set_item("seed", PACKING_SEED)?;

        let result =
            hsmc.call_method("run_simulation", (), Some(&kwargs))?;

        let box_lengths: Vec<f64> =
            result.getattr("box_lengths")?.extract()?;
        let saved_sweeps: Vec<usize> =
            result.getattr("saved_sweeps")?.extract()?;

        let packing_fractions: Vec<f64> = box_lengths
            .iter()
            .map(|&l| PACKING_N_PARTICLES as f64 * v_particle / (l * l))
            .collect();

        curves.push(PackingCurve {
            pressure,
            sweeps: saved_sweeps,
            packing_fractions,
        });

        println!("  P={pressure}: done");
    }

    let output = PackingOutput {
        n_particles: PACKING_N_PARTICLES,
        sigma: PACKING_SIGMA,
        initial_box_length: PACKING_INITIAL_BOX_LENGTH,
        curves,
    };

    write_json("exercises/exercise-4/data/P4_1/packing_python.json", &output);
    Ok(())
}

fn henderson(hsmc: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = hsmc.py();

    let v_particle =
        std::f64::consts::PI * (PACKING_SIGMA / 2.0) * (PACKING_SIGMA / 2.0);

    let mut points = Vec::new();

    for &pressure in &HENDERSON_PRESSURES {
        let kwargs = PyDict::new(py);
        kwargs.set_item("n_particles", PACKING_N_PARTICLES)?;
        kwargs.set_item("t_end", HENDERSON_T_END)?;
        kwargs.set_item("box_length", PACKING_INITIAL_BOX_LENGTH)?;
        kwargs.set_item("sigma", PACKING_SIGMA)?;
        kwargs.set_item("ensemble", "NPT")?;
        kwargs.set_item("pressure", pressure)?;
        kwargs.set_item("save_every", 1)?;
        kwargs.set_item("seed", PACKING_SEED)?;

        let result =
            hsmc.call_method("run_simulation", (), Some(&kwargs))?;

        let box_lengths: Vec<f64> =
            result.getattr("box_lengths")?.extract()?;

        let packing_fractions: Vec<f64> = box_lengths
            .iter()
            .map(|&l| PACKING_N_PARTICLES as f64 * v_particle / (l * l))
            .collect();

        let equilibrated = &packing_fractions[HENDERSON_EQUILIBRATION..];
        let n = equilibrated.len() as f64;
        let mean_phi = equilibrated.iter().sum::<f64>() / n;
        let std_phi = (equilibrated
            .iter()
            .map(|phi| (phi - mean_phi).powi(2))
            .sum::<f64>()
            / n)
            .sqrt();

        points.push(HendersonPoint {
            pressure,
            packing_fraction: mean_phi,
            packing_std: std_phi,
        });

        println!("  P={pressure}: phi={mean_phi:.4} +/- {std_phi:.4}");
    }

    let output = HendersonOutput {
        n_particles: PACKING_N_PARTICLES,
        sigma: PACKING_SIGMA,
        equilibration_sweeps: HENDERSON_EQUILIBRATION,
        points,
    };

    write_json("exercises/exercise-4/data/P4_1/henderson_python.json", &output);
    Ok(())
}

pub fn run_tasks(task: &Task) -> PyResult<()> {
    Python::with_gil(|py| {
        let hsmc = import_hsmc(py)?;

        match task {
            Task::Timing => {
                println!("=== Task 1.1a: Timing (Python) ===");
                timing(&hsmc)?;
            }
            Task::Acceptance => {
                println!("=== Task 1.1b: Acceptance (Python) ===");
                acceptance(&hsmc)?;
            }
            Task::Packing => {
                println!("=== Task 1.2a: Packing fraction (Python) ===");
                packing(&hsmc)?;
            }
            Task::Henderson => {
                println!("=== Task 1.2b: Henderson EOS (Python) ===");
                henderson(&hsmc)?;
            }
            Task::All => {
                println!("=== Task 1.1a: Timing (Python) ===");
                timing(&hsmc)?;
                println!("\n=== Task 1.1b: Acceptance (Python) ===");
                acceptance(&hsmc)?;
                println!("\n=== Task 1.2a: Packing fraction (Python) ===");
                packing(&hsmc)?;
            }
        }

        Ok(())
    })
}
