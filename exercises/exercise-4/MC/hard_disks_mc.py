"""
Educational Monte Carlo code for 2D particles (hard disks by default) in a
square box with periodic boundary conditions.

Designed for students in Statistical Mechanics.

Main features
-------------
- importable as a single module: just place ``hard_sphere_mc.py`` into the
  working directory.
- initialize N particles in 2D.
- run NVT or NPT Monte Carlo.
- one Monte Carlo sweep = N particle-move attempts plus, in NPT, 1 area move.
- store positions every ``save_every`` sweeps for later analysis.
- simple visualization helpers for snapshots and trajectories.
- pair potential isolated in its own function so students can replace the
  hard-disk interaction with e.g. Lennard-Jones.

Units
-----
Reduced units with k_B = 1.

Notes for teaching
------------------
For hard disks, the absolute temperature does not matter for displacement
moves because any overlap has infinite energy and any non-overlap has zero
energy. Nevertheless, ``temperature`` is kept in the code because it becomes
important immediately once the students switch to a soft potential like
Lennard-Jones.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Callable, Dict, List, Optional

import math
import numpy as np
import matplotlib.pyplot as plt
from matplotlib import animation
from matplotlib.patches import Circle, Rectangle

ArrayLike = np.ndarray


# -----------------------------------------------------------------------------
# Pair potentials
# -----------------------------------------------------------------------------

def hard_sphere_pair_energy(r: float, sigma: float = 1.0, epsilon: float = 1.0) -> float:
    """Hard-disk pair energy in 2D.

    Parameters
    ----------
    r : float
        Distance between two particles.
    sigma : float
        Particle diameter.
    epsilon : float
        Unused for hard disks, kept for interface compatibility.

    Returns
    -------
    float
        +inf for overlaps, 0 otherwise.

    Notes
    -----
    This function is intentionally written separately so that students can
    replace it with another potential, for example Lennard-Jones.
    """
    return np.inf if r < sigma else 0.0


def lennard_jones_pair_energy(r: float, sigma: float = 1.0, epsilon: float = 1.0) -> float:
    """Example Lennard-Jones pair potential.

    This function is provided as an example for later exercises.
    It is not used by default.
    """
    if r <= 0.0:
        return np.inf
    sr6 = (sigma / r) ** 6
    return 4.0 * epsilon * (sr6 * sr6 - sr6)


# -----------------------------------------------------------------------------
# Geometry helpers
# -----------------------------------------------------------------------------

def wrap_positions(positions: ArrayLike, box_length: float) -> ArrayLike:
    """Wrap positions back into the simulation box [0, L)."""
    return positions % box_length


def minimum_image(displacement: ArrayLike, box_length: float) -> ArrayLike:
    """Apply the minimum-image convention in a square box."""
    return displacement - box_length * np.round(displacement / box_length)


def distance(i_pos: ArrayLike, j_pos: ArrayLike, box_length: float) -> float:
    """Distance between two particles under periodic boundary conditions."""
    dr = minimum_image(i_pos - j_pos, box_length)
    return float(np.linalg.norm(dr))


# -----------------------------------------------------------------------------
# Energies
# -----------------------------------------------------------------------------

def particle_energy(
    index: int,
    positions: ArrayLike,
    box_length: float,
    pair_energy: Callable[[float, float, float], float],
    sigma: float,
    epsilon: float,
) -> float:
    """Energy contribution of one particle with all others."""
    e = 0.0
    pos_i = positions[index]
    for j in range(len(positions)):
        if j == index:
            continue
        r = distance(pos_i, positions[j], box_length)
        e_ij = pair_energy(r, sigma, epsilon)
        if np.isinf(e_ij):
            return np.inf
        e += e_ij
    return e


def total_energy(
    positions: ArrayLike,
    box_length: float,
    pair_energy: Callable[[float, float, float], float],
    sigma: float,
    epsilon: float,
) -> float:
    """Total pair energy of the system."""
    n = len(positions)
    e = 0.0
    for i in range(n - 1):
        for j in range(i + 1, n):
            r = distance(positions[i], positions[j], box_length)
            e_ij = pair_energy(r, sigma, epsilon)
            if np.isinf(e_ij):
                return np.inf
            e += e_ij
    return e


# -----------------------------------------------------------------------------
# Initialization
# -----------------------------------------------------------------------------

def initialize_square_lattice(n_particles: int, box_length: float) -> ArrayLike:
    """Place particles on a square lattice in 2D."""
    n_side = int(math.ceil(math.sqrt(n_particles)))
    spacing = box_length / n_side
    coords_1d = (np.arange(n_side) + 0.5) * spacing
    grid = np.array(np.meshgrid(coords_1d, coords_1d, indexing="ij"))
    positions = grid.reshape(2, -1).T[:n_particles].copy()
    return positions


def initialize_random_nonoverlapping(
    n_particles: int,
    box_length: float,
    sigma: float,
    rng: np.random.Generator,
    max_attempts: int = 100000,
) -> ArrayLike:
    """Random non-overlapping initialization by rejection sampling in 2D."""
    positions: List[ArrayLike] = []
    attempts = 0
    while len(positions) < n_particles and attempts < max_attempts:
        trial = rng.uniform(0.0, box_length, size=2)
        ok = True
        for pos in positions:
            if distance(trial, pos, box_length) < sigma:
                ok = False
                break
        if ok:
            positions.append(trial)
        attempts += 1
    if len(positions) < n_particles:
        raise RuntimeError(
            "Could not initialize a non-overlapping random configuration. "
            "Try a larger box or use initialization='square'."
        )
    return np.array(positions)


# -----------------------------------------------------------------------------
# Main simulation class
# -----------------------------------------------------------------------------

@dataclass
class MonteCarloResult:
    positions: ArrayLike
    trajectory: ArrayLike
    box_lengths: ArrayLike
    energies: ArrayLike
    saved_sweeps: ArrayLike
    move_acceptance: float
    volume_acceptance: float
    metadata: Dict[str, float]


class MonteCarloSystem:
    """Monte Carlo simulation for 2D particles in a square box."""

    def __init__(
        self,
        n_particles: int,
        temperature: float,
        box_length: float,
        pressure: Optional[float] = None,
        sigma: float = 1.0,
        epsilon: float = 1.0,
        max_displacement: float = 0.1,
        max_delta_log_volume: float = 0.02,
        max_delta_log_area: Optional[float] = None,
        ensemble: str = "NVT",
        pair_energy: Callable[[float, float, float], float] = hard_sphere_pair_energy,
        seed: Optional[int] = None,
        initialization: str = "square",
    ) -> None:
        self.n_particles = int(n_particles)
        self.temperature = float(temperature)
        if self.temperature <= 0.0:
            raise ValueError("temperature must be positive")
        self.beta = 1.0 / self.temperature
        self.box_length = float(box_length)
        self.pressure = None if pressure is None else float(pressure)
        self.sigma = float(sigma)
        self.epsilon = float(epsilon)
        self.max_displacement = float(max_displacement)
        if max_delta_log_area is not None:
            max_delta_log_volume = max_delta_log_area
        self.max_delta_log_volume = float(max_delta_log_volume)
        self.ensemble = ensemble.upper()
        self.pair_energy = pair_energy
        self.rng = np.random.default_rng(seed)

        if self.ensemble not in {"NVT", "NPT"}:
            raise ValueError("ensemble must be 'NVT' or 'NPT'.")
        if self.ensemble == "NPT" and self.pressure is None:
            raise ValueError("For NPT simulations, pressure must be given.")

        init = initialization.lower()
        if init in {"square", "lattice", "grid", "cubic"}:
            self.positions = initialize_square_lattice(self.n_particles, self.box_length)
        elif init == "random":
            self.positions = initialize_random_nonoverlapping(
                self.n_particles, self.box_length, self.sigma, self.rng
            )
        else:
            raise ValueError("initialization must be 'square' or 'random'.")

        e0 = total_energy(
            self.positions,
            self.box_length,
            self.pair_energy,
            self.sigma,
            self.epsilon,
        )
        if np.isinf(e0):
            raise ValueError("Initial configuration contains overlapping particles.")

    @property
    def volume(self) -> float:
        """Alias kept for compatibility; in 2D this is the box area."""
        return self.box_length ** 2

    @property
    def area(self) -> float:
        return self.box_length ** 2

    @property
    def density(self) -> float:
        return self.n_particles / self.area

    def attempt_particle_move(self) -> bool:
        """Attempt one single-particle Metropolis move."""
        i = self.rng.integers(0, self.n_particles)

        old_pos = self.positions[i].copy()
        old_e = particle_energy(
            i,
            self.positions,
            self.box_length,
            self.pair_energy,
            self.sigma,
            self.epsilon,
        )

        displacement_vec = self.rng.uniform(
            -self.max_displacement, self.max_displacement, size=2
        )
        self.positions[i] = wrap_positions(old_pos + displacement_vec, self.box_length)

        new_e = particle_energy(
            i,
            self.positions,
            self.box_length,
            self.pair_energy,
            self.sigma,
            self.epsilon,
        )

        delta_e = new_e - old_e

        if np.isinf(new_e):
            accept = False
        elif delta_e <= 0.0:
            accept = True
        else:
            accept = self.rng.random() < np.exp(-self.beta * delta_e)

        if not accept:
            self.positions[i] = old_pos
        return accept

    def attempt_volume_move(self) -> bool:
        """Attempt an isotropic NPT area move.

        In 2D, ``volume`` in the standard acceptance formula is replaced by area.
        We keep some variable names for familiarity/compatibility with earlier
        versions of the file.
        """
        if self.ensemble != "NPT":
            return False

        old_positions = self.positions.copy()
        old_box_length = self.box_length
        old_area = self.area
        old_energy = total_energy(
            self.positions,
            self.box_length,
            self.pair_energy,
            self.sigma,
            self.epsilon,
        )

        delta_log_a = self.rng.uniform(
            -self.max_delta_log_volume, self.max_delta_log_volume
        )
        new_area = old_area * np.exp(delta_log_a)
        new_box_length = math.sqrt(new_area)
        scale = new_box_length / old_box_length

        self.positions = wrap_positions(self.positions * scale, new_box_length)
        self.box_length = new_box_length

        new_energy = total_energy(
            self.positions,
            self.box_length,
            self.pair_energy,
            self.sigma,
            self.epsilon,
        )

        delta_e = new_energy - old_energy
        delta_a = new_area - old_area

        if np.isinf(new_energy):
            accept = False
        else:
            log_acceptance = (
                -self.beta * (delta_e + self.pressure * delta_a)
                + self.n_particles * np.log(new_area / old_area)
            )
            accept = (log_acceptance >= 0.0) or (
                np.log(self.rng.random()) < log_acceptance
            )

        if not accept:
            self.positions = old_positions
            self.box_length = old_box_length
        return accept

    def run(
        self,
        t_end: int,
        save_every: int = 1,
        include_initial: bool = True,
    ) -> MonteCarloResult:
        """Run the simulation for ``t_end`` Monte Carlo sweeps.

        One sweep always means:
        - N attempted particle moves
        - plus, in NPT, exactly 1 attempted area move

        States are saved every ``save_every`` sweeps. If ``include_initial=True``,
        the configuration at sweep 0 is also saved.
        """
        if t_end < 0:
            raise ValueError("t_end must be non-negative")
        if save_every <= 0:
            raise ValueError("save_every must be a positive integer")

        trajectory = []
        box_lengths = []
        energies = []
        saved_sweeps = []

        def save_state(sweep: int) -> None:
            trajectory.append(self.positions.copy())
            box_lengths.append(self.box_length)
            energies.append(
                total_energy(
                    self.positions,
                    self.box_length,
                    self.pair_energy,
                    self.sigma,
                    self.epsilon,
                )
            )
            saved_sweeps.append(int(sweep))

        if include_initial:
            save_state(0)

        n_move_attempts = 0
        n_move_accepts = 0
        n_volume_attempts = 0
        n_volume_accepts = 0

        for sweep in range(1, t_end + 1):
            for _ in range(self.n_particles):
                n_move_attempts += 1
                if self.attempt_particle_move():
                    n_move_accepts += 1

            if self.ensemble == "NPT":
                n_volume_attempts += 1
                if self.attempt_volume_move():
                    n_volume_accepts += 1

            if sweep % save_every == 0:
                save_state(sweep)

        return MonteCarloResult(
            positions=self.positions.copy(),
            trajectory=np.array(trajectory),
            box_lengths=np.array(box_lengths),
            energies=np.array(energies),
            saved_sweeps=np.array(saved_sweeps, dtype=int),
            move_acceptance=n_move_accepts / max(n_move_attempts, 1),
            volume_acceptance=n_volume_accepts / max(n_volume_attempts, 1),
            metadata={
                "n_particles": self.n_particles,
                "temperature": self.temperature,
                "pressure": np.nan if self.pressure is None else self.pressure,
                "sigma": self.sigma,
                "epsilon": self.epsilon,
                "final_box_length": self.box_length,
                "final_area": self.area,
                "final_density": self.density,
                "t_end": int(t_end),
                "save_every": int(save_every),
                "include_initial": bool(include_initial),
            },
        )


# -----------------------------------------------------------------------------
# Convenience wrapper
# -----------------------------------------------------------------------------

def run_simulation(
    n_particles: int = 64,
    t_end: int = 100,
    temperature: float = 1.0,
    box_length: float = 5.0,
    pressure: Optional[float] = None,
    sigma: float = 1.0,
    epsilon: float = 1.0,
    max_displacement: float = 0.1,
    max_delta_log_volume: float = 0.02,
    max_delta_log_area: Optional[float] = None,
    ensemble: str = "NVT",
    save_every: int = 10,
    include_initial: bool = True,
    seed: Optional[int] = None,
    initialization: str = "square",
    pair_energy: Callable[[float, float, float], float] = hard_sphere_pair_energy,
) -> MonteCarloResult:
    """Create a system and run it in one call."""
    system = MonteCarloSystem(
        n_particles=n_particles,
        temperature=temperature,
        box_length=box_length,
        pressure=pressure,
        sigma=sigma,
        epsilon=epsilon,
        max_displacement=max_displacement,
        max_delta_log_volume=max_delta_log_volume,
        max_delta_log_area=max_delta_log_area,
        ensemble=ensemble,
        pair_energy=pair_energy,
        seed=seed,
        initialization=initialization,
    )
    return system.run(t_end=t_end, save_every=save_every, include_initial=include_initial)


# -----------------------------------------------------------------------------
# Visualization helpers
# -----------------------------------------------------------------------------

def plot_snapshot(
    positions: ArrayLike,
    box_length: float,
    sigma: float = 1.0,
    title: str = "Particle snapshot",
    ax: Optional[plt.Axes] = None,
    show_disks: bool = True,
):
    """Plot a 2D snapshot of the particle positions."""
    positions = np.asarray(positions)
    if positions.ndim != 2 or positions.shape[1] != 2:
        raise ValueError("positions must have shape (N, 2)")

    created_fig = False
    if ax is None:
        fig, ax = plt.subplots(figsize=(6, 6))
        created_fig = True
    else:
        fig = ax.figure

    ax.clear()
    ax.set_title(title)
    ax.set_xlim(0.0, box_length)
    ax.set_ylim(0.0, box_length)
    ax.set_aspect("equal", adjustable="box")
    ax.add_patch(Rectangle((0, 0), box_length, box_length, fill=False, lw=1.5))

    if show_disks:
        radius = sigma / 2.0
        for x, y in positions:
            ax.add_patch(Circle((x, y), radius=radius, fill=False))
    else:
        ax.scatter(positions[:, 0], positions[:, 1], s=25)

    if created_fig:
        plt.tight_layout()
    return fig, ax


def animate_trajectory(
    trajectory: ArrayLike,
    box_lengths: ArrayLike,
    sigma: float = 1.0,
    interval: int = 200,
    show_disks: bool = True,
):
    """Create a matplotlib animation from a saved trajectory.

    By default, the trajectory uses the same disk-style visualization as
    ``plot_snapshot`` so the static and animated views look identical.
    """
    trajectory = np.asarray(trajectory)
    box_lengths = np.asarray(box_lengths)

    if trajectory.ndim != 3 or trajectory.shape[-1] != 2:
        raise ValueError("trajectory must have shape (n_frames, N, 2)")
    if len(trajectory) != len(box_lengths):
        raise ValueError("trajectory and box_lengths must have the same length")

    fig, ax = plt.subplots(figsize=(6, 6))

    def update(frame: int):
        plot_snapshot(
            trajectory[frame],
            box_lengths[frame],
            sigma=sigma,
            title=f"Frame {frame}",
            ax=ax,
            show_disks=show_disks,
        )
        return []

    ani = animation.FuncAnimation(fig, update, frames=len(trajectory), interval=interval)
    plt.close(fig)
    return ani
