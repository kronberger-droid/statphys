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


def cluster_stats(
    phi: np.ndarray,
    threshold: float = 0.1,
    phase: str = "negative",
    connectivity: int = 1,
) -> int:
    """Size (in cells) of the largest connected minority-phase cluster."""
    mask = phi < -threshold if phase == "negative" else phi > threshold
    structure = ndimage.generate_binary_structure(2, connectivity)
    labeled, ncomp = ndimage.label(mask, structure=structure)
    if ncomp == 0:
        return 0
    sizes = np.bincount(labeled.ravel())
    sizes[0] = 0
    return int(sizes.max())


def minority_cell_count(phi: np.ndarray, threshold: float = 0.1) -> int:
    """P3(b): total cells with phi < -threshold (minority-phase volume)."""
    return int(np.sum(phi < -threshold))


def compute_L_series(history: dict) -> list[float]:
    return [compute_L(np.asarray(p)) for p in history["phi"]]


def largest_cluster_series(history: dict, **kwargs) -> list[int]:
    return [cluster_stats(np.asarray(p), **kwargs) for p in history["phi"]]


def minority_count_series(history: dict, threshold: float | None = None) -> list[int]:
    """If threshold is None, use phi_binodal(T) from the history dict."""
    if threshold is None:
        threshold = float(history["phi_binodal"])
    return [minority_cell_count(np.asarray(p), threshold) for p in history["phi"]]
