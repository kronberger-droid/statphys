# Publishing the Rust binaries

This document is for colleagues who want to run the Rust ports of the
TU Wien Statistical Physics 2 reference simulations without installing
the Python toolchain. It covers what's shipped, how to build standalone
binaries, and how to publish them via GitHub Releases.

## Goal

Two Python reference implementations are the bottleneck for long runs.
Each one gets its own drop-in Rust binary:

| Python reference | Rust binary |
|---|---|
| `exercises/exercise-4/MC/hard_disks_mc.py::run_simulation` | `hard_disks_mc` |
| `exercises/exercise-5/binary_LB.py::make_spinodal_example` + `run_and_collect` | `binary_lb` |

Same parameter names, same output JSON schema — existing analysis
notebooks keep working after swapping `subprocess.run(["./hard_disks_mc", ...])`
for the Python call.

## CLI surface

### `hard_disks_mc` — 2D hard-disk MC (NVT / NPT)

```
hard_disks_mc [--n-particles --t-end --temperature --box-length --pressure
               --sigma --epsilon --max-displacement --max-delta-log-area
               --ensemble {nvt,npt} --save-every --seed
               --initialization {square,random} --output]
```

Parameter names mirror `hard_disks_mc.run_simulation` (only `max_delta_log_volume`
→ `--max-delta-log-area` to match the more common 2D naming).
Output schema matches the `MonteCarloResult` dataclass: `positions`,
`trajectory`, `box_lengths`, `energies`, `saved_sweeps`, `move_acceptance`,
`volume_acceptance`, `metadata`.

### `binary_lb` — binary-fluid lattice Boltzmann

```
binary_lb spinodal   [--nx --ny --n0 --lam --temperature --kappa --mobility
                      --tau --dt --spinodal-fraction --phi-noise
                      --no-hydrodynamics --seed --steps --snapshot-every --output]
binary_lb metastable [... --fraction-of-binodal --kt ...]
binary_lb bench      [--nx --ny --steps]
```

Parameter names map 1:1 with `binary_LB.make_{spinodal,metastable}_example`
(only `T` → `--temperature`, `M` → `--mobility`, `kT` → `--kt`).
Output schema matches Python's `run_and_collect` dict: `times`, `steps`, `phi`,
`phi_mean`, `params`, `Tc`, `phi_binodal`, `phi_spinodal`.

Accepts `--precision f32|f64` (default f64). f32 is ~1.5× faster at 128²;
parity with f64 is within 3e-4 on phi extrema.

### Parity comparison

Both binaries accept `--backend python` to run the Python reference through
PyO3 (only in builds with the `python-backend` feature — the default local
build). This flag is there for parity checks and does not ship in the public
release binaries.

## Building standalone binaries locally

To produce a binary without libpython (no Python install required to run it):

```sh
# Dynamic libc (~2.6 MB, runs on any recent Linux)
cargo build --profile release-dist --no-default-features --bin hard_disks_mc
cargo build --profile release-dist --no-default-features --bin binary_lb

# Fully static (works on any Linux kernel, no glibc dependency)
cargo build --profile release-dist --no-default-features \
            --target x86_64-unknown-linux-musl --bin binary_lb
```

Or use the helper:

```nu
nu scripts/build-dist.nu binary_lb           # dynamic libc
nu scripts/build-dist.nu binary_lb --musl    # fully static
```

## Publishing via GitHub Releases

The repo ships a workflow at `.github/workflows/release.yml` that builds
standalone `hard_disks_mc` and `binary_lb` for Linux (musl), macOS (x86_64 +
arm64), and Windows on every `v*` tag push, then creates a GitHub Release
with the artifacts attached.

```sh
git remote add origin git@github.com:<user>/statphys.git
git push -u origin main

git tag -a v0.1.0 -m "hard-disks MC + binary-fluid LB Rust ports"
git push origin v0.1.0            # triggers the release workflow
```

You can also trigger it manually from the Actions tab via `workflow_dispatch`.

Release artifacts are named `<bin>-<target>[.exe]`:

```
hard_disks_mc-x86_64-unknown-linux-musl
hard_disks_mc-x86_64-apple-darwin
hard_disks_mc-aarch64-apple-darwin
hard_disks_mc-x86_64-pc-windows-msvc.exe
binary_lb-x86_64-unknown-linux-musl
binary_lb-x86_64-apple-darwin
binary_lb-aarch64-apple-darwin
binary_lb-x86_64-pc-windows-msvc.exe
```

## For colleagues — running a downloaded binary

No install required. Download the binary for your platform, mark it
executable, and run:

```sh
chmod +x binary_lb
./binary_lb spinodal --temperature 0.3 --steps 20000 --output hist.json

chmod +x hard_disks_mc
./hard_disks_mc --ensemble npt --pressure 2.0 --n-particles 64 --t-end 5000 \
                --output run.json
```

Outputs are the same JSON schemas the Python references produce, so existing
analysis code keeps working:

```python
import json, numpy as np
d = json.load(open("hist.json"))
phi_final = np.asarray(d["phi"][-1])
print(f"phi range: [{phi_final.min():+.3f}, {phi_final.max():+.3f}]")
```

## Performance baseline

`binary_lb` at 128×128 on a typical laptop CPU (single-threaded, best-of-8):

| Backend | ms/step |
|---|---|
| Python `numpy.fft` (reference) | ~17 |
| Rust f64 | ~4 |
| Rust f32 | ~3.5 |

`hard_disks_mc` at N=100, density=0.5, NVT: Rust is 500–1000× faster than
the Python reference (the Python reference does not use a cell list; the Rust
side does).
