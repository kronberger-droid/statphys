"""Analysis helpers for exercise 5.1 lifted out of the notebook.

Keeping them in a proper module makes them easy to import from the Rust
driver via PyO3 (mirrors hard_disks_mc.py for exercise 4.1).
"""
from __future__ import annotations

import numpy as np
from scipy import ndimage


def compute_L(phi: np.ndarray) -> float:
    """Domain size L(t) = 2*pi / <k>, with <k> the first moment of S(k)."""
    phi0 = phi - phi.mean()
    phi_k = np.fft.fftn(phi0)
    S_k = np.abs(phi_k) ** 2

    Nx, Ny = phi.shape
    kx = 2 * np.pi * np.fft.fftfreq(Nx)
    ky = 2 * np.pi * np.fft.fftfreq(Ny)
    KX, KY = np.meshgrid(kx, ky, indexing="ij")
    K = np.sqrt(KX**2 + KY**2).flatten()
    S_k = S_k.flatten()

    mask = K > 0
    K = K[mask]
    S_k = S_k[mask]
    k_mean = np.sum(K * S_k) / np.sum(S_k)
    return float(2 * np.pi / k_mean)


def _minority_phase_from_mean(phi_mean: float) -> str:
    """Sign of the minority phase: if <phi> < 0, minority is positive, and vice versa."""
    return "positive" if phi_mean <= 0.0 else "negative"


def cluster_stats(
    phi: np.ndarray,
    threshold: float = 0.1,
    phase: str = "auto",
    connectivity: int = 1,
) -> int:
    """Size (in cells) of the largest connected minority-phase cluster.

    phase="auto" picks the minority sign from the mean of phi; pass "positive"
    or "negative" to override.
    """
    if phase == "auto":
        phase = _minority_phase_from_mean(float(phi.mean()))
    mask = phi > threshold if phase == "positive" else phi < -threshold
    structure = ndimage.generate_binary_structure(2, connectivity)
    labeled, ncomp = ndimage.label(mask, structure=structure)
    if ncomp == 0:
        return 0
    sizes = np.bincount(labeled.ravel())
    sizes[0] = 0
    return int(sizes.max())


def minority_cell_count(
    phi: np.ndarray,
    threshold: float = 0.1,
    phase: str = "auto",
) -> int:
    """P3(b): total cells whose phi sits on the minority side of threshold."""
    if phase == "auto":
        phase = _minority_phase_from_mean(float(phi.mean()))
    if phase == "positive":
        return int(np.sum(phi > threshold))
    return int(np.sum(phi < -threshold))


def compute_L_series(history: dict) -> list[float]:
    return [compute_L(np.asarray(p)) for p in history["phi"]]


def _resolve_minority_phase(history: dict) -> str:
    means = history.get("phi_mean")
    if means:
        return _minority_phase_from_mean(float(np.mean(means)))
    return _minority_phase_from_mean(float(np.asarray(history["phi"][0]).mean()))


def largest_cluster_series(history: dict, **kwargs) -> list[int]:
    kwargs.setdefault("phase", _resolve_minority_phase(history))
    return [cluster_stats(np.asarray(p), **kwargs) for p in history["phi"]]


def minority_count_series(
    history: dict,
    threshold: float | None = None,
    phase: str | None = None,
) -> list[int]:
    """If threshold is None, use phi_binodal(T) from the history dict.

    phase defaults to whichever side of zero the mean phi sits opposite to.
    """
    if threshold is None:
        threshold = float(history["phi_binodal"])
    if phase is None:
        phase = _resolve_minority_phase(history)
    return [minority_cell_count(np.asarray(p), threshold, phase) for p in history["phi"]]
