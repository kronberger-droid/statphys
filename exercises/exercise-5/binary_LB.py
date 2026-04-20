
"""
orlandini_binary_lb.py

Student-friendly hybrid free-energy lattice Boltzmann model for a symmetric
binary fluid inspired by Swift, Orlandini, Osborn, and Yeomans (PRE 54, 5041, 1996).

Design choices
--------------
- Order parameter: phi = Δn = n1 - n2
- Total density n is kept approximately constant (default n0 = 1.0), which is
  the regime where the 1996 binary-fluid model is especially clean.
- Hydrodynamics: D2Q9 lattice Boltzmann solver with Guo forcing.
- Thermodynamics: exact local chemical potential from the Orlandini free energy
  (their Eq. 38 at constant n), plus a square-gradient term.
- Order-parameter update: semi-implicit Fourier-space update for the stiff
  κ∇⁴phi part, which is much more stable than a fully explicit discretization.
- Fluctuations: conserved noise added as the divergence of a random flux in the
  Cahn–Hilliard sector. This is the key ingredient for metastable nucleation.
  Momentum fluctuations are left deterministic by default to keep the model fast
  and pedagogically focused.

The code is intentionally compact and transparent, not a production CFD package.
"""

from __future__ import annotations

from dataclasses import dataclass, asdict
from typing import Dict, Optional, Tuple

import numpy as np
import matplotlib.pyplot as plt
from matplotlib import animation
try:
    from IPython.display import HTML
except ModuleNotFoundError:  # running headless from Rust driver
    HTML = None


Array = np.ndarray


def _d2q9():
    c = np.array(
        [
            [0, 0],
            [1, 0],
            [0, 1],
            [-1, 0],
            [0, -1],
            [1, 1],
            [-1, 1],
            [-1, -1],
            [1, -1],
        ],
        dtype=int,
    )
    w = np.array([4 / 9] + [1 / 9] * 4 + [1 / 36] * 4, dtype=float)
    cs2 = 1.0 / 3.0
    return c, w, cs2


_C, _W, _CS2 = _d2q9()


def laplacian(field: Array) -> Array:
    """Periodic 2D five-point Laplacian with Δx = 1."""
    return (
        np.roll(field, 1, axis=0)
        + np.roll(field, -1, axis=0)
        + np.roll(field, 1, axis=1)
        + np.roll(field, -1, axis=1)
        - 4.0 * field
    )


def gradient(field: Array) -> Tuple[Array, Array]:
    """Periodic centered gradient with Δx = 1."""
    gx = 0.5 * (np.roll(field, -1, axis=1) - np.roll(field, 1, axis=1))
    gy = 0.5 * (np.roll(field, -1, axis=0) - np.roll(field, 1, axis=0))
    return gx, gy


def divergence(vx: Array, vy: Array) -> Array:
    """Periodic centered divergence with Δx = 1."""
    dx = 0.5 * (np.roll(vx, -1, axis=1) - np.roll(vx, 1, axis=1))
    dy = 0.5 * (np.roll(vy, -1, axis=0) - np.roll(vy, 1, axis=0))
    return dx + dy


def _safe_log_ratio(phi: Array, n0: float, clip_fraction: float = 0.999999) -> Array:
    """
    Stable evaluation of log((n0+phi)/(n0-phi)).

    The binary-mixture free energy requires |phi| < n0. We clip very slightly
    inside that interval for numerical safety.
    """
    bound = clip_fraction * n0
    phi_clip = np.clip(phi, -bound, bound)
    return np.log((n0 + phi_clip) / (n0 - phi_clip))


def exact_spinodal_phi(T: Array | float, lam: float, n0: float = 1.0) -> Array | float:
    """
    Exact spinodal of the symmetric Orlandini bulk free energy at fixed n.
    Tc = lam / 2.
    """
    T_arr = np.asarray(T, dtype=float)
    val = np.maximum(0.0, 1.0 - 2.0 * T_arr / lam)
    out = n0 * np.sqrt(val)
    return out if isinstance(T, np.ndarray) else float(out)


def exact_binodal_phi(T: Array | float, lam: float, n0: float = 1.0,
                      tol: float = 1e-12, max_iter: int = 200) -> Array | float:
    """
    Exact coexistence curve of the symmetric Orlandini bulk free energy at fixed n.

    For m = phi / n0, coexistence satisfies:
        artanh(m) = (lam / (2 T)) * m
    besides the trivial m=0 root.
    We solve for the nonzero root on [0, 1).
    """
    T_arr = np.asarray(T, dtype=float)
    Tc = lam / 2.0
    result = np.zeros_like(T_arr, dtype=float)

    flat = result.ravel()
    Tflat = T_arr.ravel()
    for i, Ti in enumerate(Tflat):
        if Ti <= 0:
            flat[i] = n0
            continue
        if Ti >= Tc:
            flat[i] = 0.0
            continue

        alpha = lam / (2.0 * Ti)

        def g(m: float) -> float:
            return np.arctanh(m) - alpha * m

        lo = 1e-14
        hi = 1.0 - 1e-12
        for _ in range(max_iter):
            mid = 0.5 * (lo + hi)
            gmid = g(mid)
            if abs(gmid) < tol or (hi - lo) < tol:
                flat[i] = n0 * mid
                break
            if gmid > 0:
                hi = mid
            else:
                lo = mid
        else:
            flat[i] = n0 * 0.5 * (lo + hi)

    return result if isinstance(T, np.ndarray) else float(result)


@dataclass
class SimulationParameters:
    Nx: int = 128
    Ny: int = 128
    n0: float = 1.0
    lam: float = 1.1
    T: float = 0.50
    kappa: float = 0.12
    M: float = 0.08
    tau: float = 0.9
    dt: float = 0.5
    phi0: float = 0.0
    phi_noise: float = 1e-3
    kT: float = 0.0
    seed: Optional[int] = None
    hydrodynamics: bool = True
    dealias_clip: float = 0.999999
    mobility_prefactor: float = 1.0


class OrlandiniBinaryFluid2D:
    """
    Hybrid free-energy LB binary fluid.

    Notes
    -----
    The deterministic thermodynamics follow the binary-fluid free energy of the
    1996 Orlandini/Swift/Yeomans paper at fixed total density n0:
        c(phi) = lam/4 * n0 * (1 - phi^2 / n0^2)
                 + T/2 * (n0+phi) ln((n0+phi)/2)
                 + T/2 * (n0-phi) ln((n0-phi)/2)
                 + const
    which yields the chemical potential difference
        mu = -lam/2 * phi / n0 + T/2 * ln((n0+phi)/(n0-phi)) - kappa ∇² phi.
    """

    def __init__(self, **kwargs):
        params = SimulationParameters(**kwargs)
        self.params = params
        self.Nx = params.Nx
        self.Ny = params.Ny
        self.n0 = float(params.n0)
        self.lam = float(params.lam)
        self.T = float(params.T)
        self.kappa = float(params.kappa)
        self.M = float(params.M) * float(params.mobility_prefactor)
        self.tau = float(params.tau)
        self.dt = float(params.dt)
        self.kT = float(params.kT)
        self.hydrodynamics = bool(params.hydrodynamics)
        self.clip_fraction = float(params.dealias_clip)

        self.rng = np.random.default_rng(params.seed)

        self.phi_mean_target = float(params.phi0)
        self.phi = (
            self.phi_mean_target
            + params.phi_noise * self.rng.standard_normal((self.Ny, self.Nx))
        )
        self.phi = self._project_mean(self.phi)
        self.u = np.zeros((self.Ny, self.Nx, 2), dtype=float)
        self.rho = np.full((self.Ny, self.Nx), self.n0, dtype=float)

        self.c = _C
        self.w = _W
        self.cs2 = _CS2

        self.f = self.equilibrium(self.rho, self.u)

        kx = 2.0 * np.pi * np.fft.fftfreq(self.Nx)
        ky = 2.0 * np.pi * np.fft.fftfreq(self.Ny)
        self.kx, self.ky = np.meshgrid(kx, ky)
        self.k2 = self.kx**2 + self.ky**2
        self.k4 = self.k2**2

        self.time = 0.0
        self.step_count = 0

    @property
    def Tc(self) -> float:
        return 0.5 * self.lam

    @property
    def reduced_temperature(self) -> float:
        return self.T / self.Tc

    def bulk_free_energy_density(self, phi: Array) -> Array:
        phi_clip = np.clip(phi, -self.clip_fraction * self.n0, self.clip_fraction * self.n0)
        return (
            0.25 * self.lam * self.n0 * (1.0 - (phi_clip * phi_clip) / (self.n0 * self.n0))
            + 0.5 * self.T * (self.n0 + phi_clip) * np.log((self.n0 + phi_clip) / 2.0)
            + 0.5 * self.T * (self.n0 - phi_clip) * np.log((self.n0 - phi_clip) / 2.0)
        )

    def chemical_potential_bulk(self, phi: Array) -> Array:
        return -0.5 * self.lam * phi / self.n0 + 0.5 * self.T * _safe_log_ratio(
            phi, self.n0, self.clip_fraction
        )

    def chemical_potential(self, phi: Optional[Array] = None) -> Array:
        if phi is None:
            phi = self.phi
        return self.chemical_potential_bulk(phi) - self.kappa * laplacian(phi)

    def pressure_force(self) -> Tuple[Array, Array]:
        mu = self.chemical_potential(self.phi)
        mux, muy = gradient(mu)
        fx = -self.phi * mux
        fy = -self.phi * muy
        return fx, fy

    def equilibrium(self, rho: Array, u: Array) -> Array:
        ux = u[..., 0]
        uy = u[..., 1]
        usq = ux * ux + uy * uy
        feq = np.empty((9, self.Ny, self.Nx), dtype=float)
        for i, (cx, cy) in enumerate(self.c):
            cu = cx * ux + cy * uy
            feq[i] = self.w[i] * rho * (
                1.0 + cu / self.cs2 + 0.5 * (cu * cu) / (self.cs2 * self.cs2) - 0.5 * usq / self.cs2
            )
        return feq

    def guo_force_term(self, u: Array, fx: Array, fy: Array) -> Array:
        ux = u[..., 0]
        uy = u[..., 1]
        F = np.empty((9, self.Ny, self.Nx), dtype=float)
        pref = (1.0 - 0.5 / self.tau)
        for i, (cx, cy) in enumerate(self.c):
            ci_dot_u = cx * ux + cy * uy
            ci_dot_f = cx * fx + cy * fy
            u_dot_f = ux * fx + uy * fy
            term = ((ci_dot_f - u_dot_f) / self.cs2
                    + (ci_dot_u * ci_dot_f) / (self.cs2 * self.cs2))
            F[i] = pref * self.w[i] * term
        return F

    def lb_step(self) -> None:
        if not self.hydrodynamics:
            self.u.fill(0.0)
            self.rho.fill(self.n0)
            return

        fx, fy = self.pressure_force()
        self.rho = np.sum(self.f, axis=0)
        jx = np.tensordot(self.c[:, 0], self.f, axes=(0, 0))
        jy = np.tensordot(self.c[:, 1], self.f, axes=(0, 0))
        ux = (jx + 0.5 * fx * self.dt) / self.rho
        uy = (jy + 0.5 * fy * self.dt) / self.rho
        self.u[..., 0] = ux
        self.u[..., 1] = uy

        feq = self.equilibrium(self.rho, self.u)
        force_term = self.guo_force_term(self.u, fx, fy)

        self.f += -(self.f - feq) / self.tau + self.dt * force_term

        for i, (cx, cy) in enumerate(self.c):
            self.f[i] = np.roll(self.f[i], shift=cy, axis=0)
            self.f[i] = np.roll(self.f[i], shift=cx, axis=1)

        self.rho = np.sum(self.f, axis=0)
        jx = np.tensordot(self.c[:, 0], self.f, axes=(0, 0))
        jy = np.tensordot(self.c[:, 1], self.f, axes=(0, 0))
        ux = (jx + 0.5 * fx * self.dt) / self.rho
        uy = (jy + 0.5 * fy * self.dt) / self.rho
        self.u[..., 0] = ux
        self.u[..., 1] = uy

    def advective_divergence(self, phi: Array, u: Array) -> Array:
        return divergence(phi * u[..., 0], phi * u[..., 1])

    def conservative_noise_divergence(self) -> Array:
        if self.kT <= 0.0:
            return np.zeros_like(self.phi)
        sigma = np.sqrt(max(0.0, 2.0 * self.M * self.kT / self.dt))
        jx = sigma * self.rng.standard_normal(self.phi.shape)
        jy = sigma * self.rng.standard_normal(self.phi.shape)
        return divergence(jx, jy)

    def _project_mean(self, field: Array) -> Array:
        return field - field.mean() + self.phi_mean_target

    def k2_ifft(self, field: Array) -> Array:
        return np.real(np.fft.ifft2(self.k2 * np.fft.fft2(field)))

    def phi_step(self) -> None:
        phi = self.phi
        mu_bulk = self.chemical_potential_bulk(phi)
        adv = self.advective_divergence(phi, self.u)
        noise_div = self.conservative_noise_divergence()

        rhs = phi + self.dt * (-self.M * self.k2_ifft(mu_bulk) - adv + noise_div)

        rhs_k = np.fft.fft2(rhs)
        denom = 1.0 + self.dt * self.M * self.kappa * self.k4
        denom[0, 0] = 1.0
        phi_new = np.real(np.fft.ifft2(rhs_k / denom))
        self.phi = self._project_mean(phi_new)

    def step(self, n_steps: int = 1) -> None:
        for _ in range(n_steps):
            self.lb_step()
            self.phi_step()
            self.time += self.dt
            self.step_count += 1

    def get_state(self) -> Dict[str, Array | float | int]:
        mu = self.chemical_potential(self.phi)
        return {
            "phi": self.phi.copy(),
            "rho": self.rho.copy(),
            "ux": self.u[..., 0].copy(),
            "uy": self.u[..., 1].copy(),
            "u_mag": np.sqrt(self.u[..., 0] ** 2 + self.u[..., 1] ** 2),
            "mu": mu.copy(),
            "time": float(self.time),
            "step": int(self.step_count),
            "phi_mean": float(self.phi.mean()),
        }

    def diagnostics(self) -> Dict[str, float]:
        mu = self.chemical_potential(self.phi)
        gradx, grady = gradient(self.phi)
        free_energy = (
            np.sum(self.bulk_free_energy_density(self.phi))
            + 0.5 * self.kappa * np.sum(gradx**2 + grady**2)
        )
        return {
            "time": float(self.time),
            "step": int(self.step_count),
            "phi_mean": float(self.phi.mean()),
            "phi_min": float(self.phi.min()),
            "phi_max": float(self.phi.max()),
            "mu_rms": float(np.sqrt(np.mean(mu * mu))),
            "u_rms": float(np.sqrt(np.mean(self.u[..., 0] ** 2 + self.u[..., 1] ** 2))),
            "free_energy": float(free_energy),
        }


def make_spinodal_example(
    Nx: int = 128,
    Ny: int = 128,
    n0: float = 1.0,
    lam: float = 1.1,
    T: float = 0.50,
    kappa: float = 0.12,
    M: float = 0.08,
    tau: float = 0.9,
    dt: float = 0.5,
    spinodal_fraction: float = 0.40,
    phi_noise: float = 2e-3,
    hydrodynamics: bool = True,
    seed: Optional[int] = None,
    **kwargs,
) -> OrlandiniBinaryFluid2D:
    phi_spin = exact_spinodal_phi(T, lam, n0)
    phi0 = (2 * spinodal_fraction - 1) * phi_spin

    params = dict(
        Nx=Nx, Ny=Ny, n0=n0, lam=lam, T=T, kappa=kappa, M=M, tau=tau, dt=dt,
        phi0=phi0, phi_noise=phi_noise, kT=0.0, hydrodynamics=hydrodynamics, seed=seed
    )
    params.update(kwargs)
    return OrlandiniBinaryFluid2D(**params)


def make_metastable_example(
    Nx: int = 128,
    Ny: int = 128,
    n0: float = 1.0,
    lam: float = 1.1,
    T: float = 0.50,
    kappa: float = 0.12,
    M: float = 0.08,
    tau: float = 0.9,
    dt: float = 0.5,
    fraction_of_binodal: float = 0.82,
    phi_noise: float = 1e-3,
    kT: float = 0.0025,
    hydrodynamics: bool = True,
    seed: Optional[int] = None,
    **kwargs,
) -> OrlandiniBinaryFluid2D:
    phi_bin = exact_binodal_phi(T, lam, n0)
    phi0 = (2 * fraction_of_binodal - 1) * phi_bin
    params = dict(
        Nx=Nx, Ny=Ny, n0=n0, lam=lam, T=T, kappa=kappa, M=M, tau=tau, dt=dt,
        phi0=phi0, phi_noise=phi_noise, kT=kT, hydrodynamics=hydrodynamics, seed=seed
    )
    params.update(kwargs)
    return OrlandiniBinaryFluid2D(**params)


def run_and_collect(
    sim: OrlandiniBinaryFluid2D,
    steps: int,
    snapshot_every: int = 20,
    include_mu: bool = False,
    include_velocity: bool = True,
) -> Dict[str, object]:
    history: Dict[str, object] = {
        "times": [],
        "steps": [],
        "phi": [],
        "phi_mean": [],
        "params": asdict(sim.params),
        "Tc": sim.Tc,
        "phi_binodal": exact_binodal_phi(sim.T, sim.lam, sim.n0),
        "phi_spinodal": exact_spinodal_phi(sim.T, sim.lam, sim.n0),
    }
    if include_mu:
        history["mu"] = []
    if include_velocity:
        history["ux"] = []
        history["uy"] = []
        history["u_mag"] = []

    state = sim.get_state()
    history["times"].append(state["time"])
    history["steps"].append(state["step"])
    history["phi"].append(state["phi"])
    history["phi_mean"].append(state["phi_mean"])
    if include_mu:
        history["mu"].append(state["mu"])
    if include_velocity:
        history["ux"].append(state["ux"])
        history["uy"].append(state["uy"])
        history["u_mag"].append(state["u_mag"])

    nblocks = steps // snapshot_every
    remainder = steps % snapshot_every
    for _ in range(nblocks):
        sim.step(snapshot_every)
        state = sim.get_state()
        history["times"].append(state["time"])
        history["steps"].append(state["step"])
        history["phi"].append(state["phi"])
        history["phi_mean"].append(state["phi_mean"])
        if include_mu:
            history["mu"].append(state["mu"])
        if include_velocity:
            history["ux"].append(state["ux"])
            history["uy"].append(state["uy"])
            history["u_mag"].append(state["u_mag"])
    if remainder:
        sim.step(remainder)
        state = sim.get_state()
        history["times"].append(state["time"])
        history["steps"].append(state["step"])
        history["phi"].append(state["phi"])
        history["phi_mean"].append(state["phi_mean"])
        if include_mu:
            history["mu"].append(state["mu"])
        if include_velocity:
            history["ux"].append(state["ux"])
            history["uy"].append(state["uy"])
            history["u_mag"].append(state["u_mag"])

    return history


def _history_field(history: Dict[str, object], which: str, frame: int) -> Array:
    seq = history[which]
    if not isinstance(seq, list):
        raise ValueError(f"history['{which}'] is not a frame sequence")
    return seq[frame]


def plot_snapshot(
    history: Dict[str, object],
    which: str = "phi",
    frame: int = -1,
    ax=None,
    title: Optional[str] = None,
    vmin: Optional[float] = None,
    vmax: Optional[float] = None,
    autoscale: bool = False,
    center_on_mean: bool = False,
    cmap: str = "coolwarm",
):
    field = np.array(_history_field(history, which, frame), copy=True)
    if center_on_mean:
        field = field - np.mean(field)

    if ax is None:
        fig, ax = plt.subplots(figsize=(5.8, 4.6))
    else:
        fig = ax.figure

    if autoscale and (vmin is None and vmax is None):
        vmin = float(np.min(field))
        vmax = float(np.max(field))
    elif vmin is None and vmax is None and which == "phi":
        phi_bin = float(history.get("phi_binodal", max(abs(field.min()), abs(field.max()))))
        if center_on_mean:
            spread = max(abs(field.min()), abs(field.max()), 1e-12)
            vmin, vmax = -spread, spread
        else:
            vmin, vmax = -phi_bin, phi_bin

    im = ax.imshow(field, origin="lower", cmap=cmap, vmin=vmin, vmax=vmax)
    t = history["times"][frame]
    if title is None:
        title = f"{which} at t = {t:.0f}"
    ax.set_title(title)
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    cbar = fig.colorbar(im, ax=ax)
    cbar.set_label(which)
    return fig, ax, im


def plot_velocity(
    history: Dict[str, object],
    frame: int = -1,
    stride: int = 6,
    ax=None,
    vmin: Optional[float] = None,
    vmax: Optional[float] = None,
):
    if "u_mag" not in history or "ux" not in history or "uy" not in history:
        raise ValueError("history does not contain velocity fields")
    u_mag = np.asarray(history["u_mag"][frame])
    ux = np.asarray(history["ux"][frame])
    uy = np.asarray(history["uy"][frame])

    if ax is None:
        fig, ax = plt.subplots(figsize=(5.8, 4.6))
    else:
        fig = ax.figure

    im = ax.imshow(u_mag, origin="lower", cmap="viridis", vmin=vmin, vmax=vmax)
    yy, xx = np.mgrid[0:u_mag.shape[0]:stride, 0:u_mag.shape[1]:stride]
    ax.quiver(xx, yy, ux[::stride, ::stride], uy[::stride, ::stride], color="white", scale=1.5)
    ax.set_title(f"|u| at t = {history['times'][frame]:.0f}")
    ax.set_xlabel("x")
    ax.set_ylabel("y")
    cbar = fig.colorbar(im, ax=ax)
    cbar.set_label("|u|")
    return fig, ax, im


def animate_history(
    history: Dict[str, object],
    which: str = "phi",
    interval: int = 80,
    repeat: bool = True,
    vmin: Optional[float] = None,
    vmax: Optional[float] = None,
    autoscale: bool = False,
    center_on_mean: bool = False,
    cmap: str = "coolwarm",
):
    frames = history[which]
    if not isinstance(frames, list):
        raise ValueError(f"history['{which}'] is not a list of frames")
    nframes = len(frames)

    if not autoscale and vmin is None and vmax is None:
        if which == "phi":
            phi_bin = float(history.get("phi_binodal", 1.0))
            if center_on_mean:
                spread = 0.0
                for fr in frames:
                    ff = np.asarray(fr) - np.mean(fr)
                    spread = max(spread, float(np.max(np.abs(ff))))
                spread = max(spread, 1e-12)
                vmin, vmax = -spread, spread
            else:
                vmin, vmax = -phi_bin, phi_bin
        else:
            stack_min = min(float(np.min(np.asarray(fr))) for fr in frames)
            stack_max = max(float(np.max(np.asarray(fr))) for fr in frames)
            vmin, vmax = stack_min, stack_max

    fig, ax = plt.subplots(figsize=(6.0, 5.0))
    field0 = np.array(frames[0], copy=True)
    if center_on_mean:
        field0 = field0 - np.mean(field0)

    im = ax.imshow(field0, origin="lower", cmap=cmap, vmin=vmin, vmax=vmax)
    cbar = fig.colorbar(im, ax=ax)
    cbar.set_label(which)
    title = ax.set_title(f"{which} at t = {history['times'][0]:.0f}")
    ax.set_xlabel("x")
    ax.set_ylabel("y")

    def update(i):
        field = np.array(frames[i], copy=True)
        if center_on_mean:
            field = field - np.mean(field)
        if autoscale:
            im.set_clim(float(np.min(field)), float(np.max(field)))
        im.set_data(field)
        title.set_text(f"{which} at t = {history['times'][i]:.0f}")
        return (im, title)

    ani = animation.FuncAnimation(
        fig, update, frames=nframes, interval=interval, blit=False, repeat=repeat
    )
    plt.close(fig)
    return ani


def display_animation(*args, **kwargs):
    ani = animate_history(*args, **kwargs)
    return HTML(ani.to_jshtml())


def phase_diagram_curves(
    lam: float = 1.1,
    n0: float = 1.0,
    nT: int = 400,
    Tmin: float = 0.0,
    Tmax: Optional[float] = None,
) -> Dict[str, Array]:
    if Tmax is None:
        Tmax = lam / 2.0
    T = np.linspace(Tmin, Tmax, nT)
    phi_spin = exact_spinodal_phi(T, lam, n0)
    phi_bin = exact_binodal_phi(T, lam, n0)
    return {"T": T, "phi_spin": phi_spin, "phi_bin": phi_bin, "Tc": lam / 2.0}


def plot_phase_diagram(
    lam: float = 1.1,
    n0: float = 1.0,
    ax=None,
):
    curves = phase_diagram_curves(lam=lam, n0=n0)
    T = curves["T"]
    phi_spin = curves["phi_spin"]
    phi_bin = curves["phi_bin"]

    if ax is None:
        fig, ax = plt.subplots(figsize=(5.8, 4.6))
    else:
        fig = ax.figure

    ax.plot(phi_bin, T, label="binodal")
    ax.plot(-phi_bin, T)
    ax.plot(phi_spin, T, "--", label="spinodal")
    ax.plot(-phi_spin, T, "--")
    ax.axhline(curves["Tc"], linestyle=":", linewidth=1.0, label=r"$T_c$")
    ax.set_xlabel(r"$\Delta n$")
    ax.set_ylabel(r"$T$")
    ax.set_title("Binary-fluid phase diagram")
    ax.legend()
    return fig, ax


def parameter_summary(sim: OrlandiniBinaryFluid2D) -> Dict[str, float]:
    phi_bin = exact_binodal_phi(sim.T, sim.lam, sim.n0)
    phi_spin = exact_spinodal_phi(sim.T, sim.lam, sim.n0)
    return {
        "T": sim.T,
        "Tc": sim.Tc,
        "T_over_Tc": sim.T / sim.Tc,
        "phi_mean_target": sim.phi_mean_target,
        "phi_binodal": phi_bin,
        "phi_spinodal": phi_spin,
        "inside_spinodal": abs(sim.phi_mean_target) < phi_spin,
        "metastable": (abs(sim.phi_mean_target) > phi_spin) and (abs(sim.phi_mean_target) < phi_bin),
        "stable_one_phase": abs(sim.phi_mean_target) >= phi_bin,
        "kappa": sim.kappa,
        "kT": sim.kT,
    }
