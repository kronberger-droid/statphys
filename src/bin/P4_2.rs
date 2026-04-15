use std::fs;

use clap::{Parser, Subcommand};
use serde::Serialize;
use statphys::{Position3D, write_json};

#[derive(Parser)]
#[command(about = "Exercise 4.2: MD Lennard-Jones fluid analysis")]
struct Cli {
    #[command(subcommand)]
    task: Task,
}

#[derive(Subcommand)]
enum Task {
    /// 2a: Potential energy along trajectory
    Energy,
    /// 2b: Radial distribution function g(r)
    Rdf,
    /// Run all tasks
    All,
}

const STATE_POINTS: [(&str, f64, f64); 4] = [
    ("rho0.05-T0.5", 0.05, 0.5),
    ("rho0.05-T1.0", 0.05, 1.0),
    ("rho0.3-T0.5", 0.3, 0.5),
    ("rho0.3-T1.0", 0.3, 1.0),
];

const LJ_SIGMA: f64 = 1.0;
const LJ_EPSILON: f64 = 1.0;
const LJ_CUTOFF: f64 = 2.5;

struct Frame {
    timestep: usize,
    box_length: f64,
    positions: Vec<Position3D>,
}

/// Parse all frames from a LAMMPS trajectory file.
fn parse_trajectory(path: &str) -> Vec<Frame> {
    let content = fs::read_to_string(path).unwrap();
    let mut lines = content.lines();

    let mut frames = Vec::new();

    while let Some(_) = lines.next() {
        // skip: "ITEM: TIMESTEP" (consumed by while let)
        let timestep: usize = lines.next().unwrap().trim().parse().unwrap();

        lines.next(); // skip: "ITEM: NUMBER OF ATOMS"
        let n_atoms: usize = lines.next().unwrap().trim().parse().unwrap();

        lines.next(); // skip: "ITEM: BOX BOUNDS pp pp pp"
        let mut parts = lines.next().unwrap().split_whitespace(); // xlo xhi
        let xlo: f64 = parts.next().unwrap().parse().unwrap();
        let xhi: f64 = parts.next().unwrap().parse().unwrap();
        let box_length = xhi - xlo;
        lines.next(); // skip: ylo yhi
        lines.next(); // skip: zlo zhi

        lines.next(); // skip: "ITEM: ATOMS id type x y z"
        let mut positions = Vec::new();
        for _ in 0..n_atoms {
            let mut parts = lines.next().unwrap().split_whitespace();
            parts.next(); // skip: id
            parts.next(); // skip: type

            let x: f64 = parts.next().unwrap().parse().unwrap();
            let y: f64 = parts.next().unwrap().parse().unwrap();
            let z: f64 = parts.next().unwrap().parse().unwrap();

            positions.push(Position3D::new(x, y, z));
        }
        frames.push(Frame {
            timestep,
            box_length,
            positions,
        });
    }
    frames
}

/// Read the average energy from the average-energy.txt file.
fn read_average_energy(path: &str) -> f64 {
    let content = fs::read_to_string(path).unwrap();
    content.split_whitespace().last().unwrap().parse().unwrap()
}

/// Lennard-Jones pair potential with cutoff.
fn lj_energy(r: f64) -> f64 {
    let sr6 = (LJ_SIGMA / r).powi(6);
    let energy = 4. * LJ_EPSILON * (sr6.powi(2) - sr6);

    if r < LJ_CUTOFF { energy } else { 0. }
}

/// Total potential energy per particle for one frame.
fn frame_energy_per_particle(frame: &Frame) -> f64 {
    let n = frame.positions.len();
    let l = frame.box_length;
    let mut energy = 0.;

    for i in 0..n {
        for j in (i + 1)..n {
            let mut dx = frame.positions[i].x - frame.positions[j].x;
            let mut dy = frame.positions[i].y - frame.positions[j].y;
            let mut dz = frame.positions[i].z - frame.positions[j].z;

            dx -= l * (dx / l).round();
            dy -= l * (dy / l).round();
            dz -= l * (dz / l).round();

            let r = (dx.powi(2) + dy.powi(2) + dz.powi(2)).sqrt();

            energy += lj_energy(r);
        }
    }
    energy / n as f64
}

#[derive(Serialize)]
struct EnergyOutput {
    state_point: String,
    rho: f64,
    temperature: f64,
    average_energy: f64,
    timesteps: Vec<usize>,
    energy_per_particle: Vec<f64>,
}

fn task_energy() {
    for &(name, rho, temp) in &STATE_POINTS {
        let base = format!("exercises/exercise-4/MD/{name}");
        let frames = parse_trajectory(&format!("{base}/trajectory.lammpstrj"));
        let avg_energy =
            read_average_energy(&format!("{base}/average-energy.txt"));

        let timesteps: Vec<usize> = frames.iter().map(|f| f.timestep).collect();
        let energies: Vec<f64> =
            frames.iter().map(frame_energy_per_particle).collect();

        let output = EnergyOutput {
            state_point: name.to_string(),
            rho,
            temperature: temp,
            average_energy: avg_energy,
            timesteps,
            energy_per_particle: energies,
        };

        write_json(&format!("data/P4_2/energy_{name}.json"), &output);
    }
}

#[derive(Serialize)]
struct RdfOutput {
    state_point: String,
    rho: f64,
    temperature: f64,
    r: Vec<f64>,
    g_r: Vec<f64>,
}

fn task_rdf() {
    for &(name, rho, temp) in &STATE_POINTS {
        let base = format!("exercises/exercise-4/MD/{name}");
        let frames = parse_trajectory(&format!("{base}/trajectory.lammpstrj"));

        let n_bins = 200;
        let r_max = frames[0].box_length / 2.0; // max r = half box length
        let bin_width = r_max / n_bins as f64;

        let mut histogram = vec![0usize; n_bins];

        for frame in &frames {
            let n = frame.positions.len();
            let l = frame.box_length;
            for i in 0..n {
                for j in (i + 1)..n {
                    let mut dx = frame.positions[i].x - frame.positions[j].x;
                    let mut dy = frame.positions[i].y - frame.positions[j].y;
                    let mut dz = frame.positions[i].z - frame.positions[j].z;

                    dx -= l * (dx / l).round();
                    dy -= l * (dy / l).round();
                    dz -= l * (dz / l).round();

                    let r = (dx.powi(2) + dy.powi(2) + dz.powi(2)).sqrt();
                    if r < r_max {
                        let bin = (r / bin_width) as usize;
                        histogram[bin] += 1;
                    }
                }
            }
        }
        let n_frames = frames.len() as f64;
        let n = frames.first().unwrap().positions.len() as f64;

        let r = (0..n_bins)
            .map(|i| i as f64 * bin_width + bin_width / 2.)
            .collect();

        let g_r: Vec<f64> = (0..n_bins)
            .map(|i| {
                let r_inner = i as f64 * bin_width;
                let r_outer = r_inner + bin_width;
                let v_shell = 4. / 3.
                    * std::f64::consts::PI
                    * (r_outer.powi(3) - r_inner.powi(3));
                2.0 * histogram[i] as f64 / (n_frames * n * rho * v_shell)
            })
            .collect();

        let output = RdfOutput {
            state_point: name.to_string(),
            rho,
            temperature: temp,
            r,
            g_r,
        };

        write_json(&format!("data/P4_2/rdf_{name}.json"), &output);
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.task {
        Task::Energy => {
            println!("=== Task 2a: Potential energy ===");
            task_energy();
        }
        Task::Rdf => {
            println!("=== Task 2b: RDF ===");
            task_rdf();
        }
        Task::All => {
            println!("=== Task 2a: Potential energy ===");
            task_energy();
            println!("\n=== Task 2b: RDF ===");
            task_rdf();
        }
    }
}
