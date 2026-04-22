# Publishing the optimized Rust binaries

This document is for colleagues who want to run the Rust ports of the statistical-physics
exercise reference implementations without installing the full toolchain. It covers
what's provided, how to build standalone binaries, and how to ship them via GitHub
Releases.

## Goal

Two of the TU Wien Statistical Physics 2 exercise sheets ship Python reference
implementations that become the bottleneck for long runs:

| Exercise | Python reference | Rust port |
|---|---|---|
| 4 — Monte Carlo hard disks (NVT / NPT) | `exercises/exercise-4/MC/hard_disks_mc.py` | `src/mc/` + `src/bin/P4_1`, `src/bin/P4_2` |
| 5 — Binary-fluid lattice Boltzmann | `exercises/exercise-5/binary_LB.py` | `src/lb/` + `src/bin/binary_lb`, `src/bin/P5_1` |

The Rust ports preserve the Python calling conventions (parameter names, output
schema) so switching from `import binary_LB` to the Rust binary only requires
changing the invocation, not the analysis pipeline.

## CLI surface parity

### Exercise 4 (`P4_1`, `P4_2`)
Matches the exercise-sheet tasks. Pick a subcommand (`timing`, `acceptance`,
`packing`, `henderson`, …) and optionally `--backend python` to run the Python
reference instead of the Rust implementation — useful for parity checks.

### Exercise 5 — general-purpose (`binary_lb`)
Drop-in replacement for `binary_LB.make_spinodal_example` + `run_and_collect`:

```
binary_lb spinodal   [--nx --ny --n0 --lam --temperature --kappa --mobility
                      --tau --dt --spinodal-fraction --phi-noise
                      --no-hydrodynamics --seed --steps --snapshot-every --output]
binary_lb metastable [... --fraction-of-binodal --kt ...]
binary_lb bench      [--nx --ny --steps]
```

All parameter names map 1:1 with the Python API (only `T` → `--temperature`,
`M` → `--mobility`, `kT` → `--kt`). Output JSON schema mirrors Python's
`run_and_collect` dict (`times`, `steps`, `phi`, `phi_mean`, `params`, `Tc`,
`phi_binodal`, `phi_spinodal`).

### Exercise 5 — exercise presets (`P5_1`)
`temperatures`, `timesteps`, `asymmetric`, `domain-growth`, `nucleation`,
`minority-count`, `all`, `bench`. These wrap the task-sheet parameter sets.

### Precision
Every Rust run accepts `--precision f32|f64`. f32 is ~1.5× faster on the LB
binary; parity with f64 is within 3e-4 on phi extrema at 128² grids.

## Building standalone binaries

The default build dynamically links libpython (for the `--backend python`
parity comparison). To produce a binary you can hand to a colleague without
a Python install, turn off the `python-backend` feature:

```sh
# Dynamic libc (smallest, runs on any recent Linux)
cargo build --profile release-dist --no-default-features --bin binary_lb
cargo build --profile release-dist --no-default-features --bin P4_1
cargo build --profile release-dist --no-default-features --bin P4_2
cargo build --profile release-dist --no-default-features --bin P5_1

# Fully static (works on any Linux kernel, no glibc dependency)
cargo build --profile release-dist --no-default-features \
            --target x86_64-unknown-linux-musl --bin binary_lb
```

Or use the helper:

```nu
nu scripts/build-dist.nu binary_lb
nu scripts/build-dist.nu binary_lb --musl          # fully static
```

The output lives in `target/release-dist/<bin>` (dynamic) or
`dist/<bin>` (via the script). All binaries are 2–3 MB and link only libc and
libm.

## Publishing via GitHub Releases

1. **Push to GitHub.** The repo is already a git repo; add a remote and push:
   ```sh
   git remote add origin git@github.com:<user>/statphys.git
   git push -u origin main
   ```
2. **Tag a release locally**:
   ```sh
   git tag -a v0.1.0 -m "Exercise 4 + 5 Rust ports"
   git push origin v0.1.0
   ```
3. **Build release artifacts** and attach them via the GitHub web UI or `gh`:
   ```sh
   mkdir -p dist
   for bin in binary_lb P4_1 P4_2 P5_1; do
     nu scripts/build-dist.nu $bin --musl
   done
   gh release create v0.1.0 dist/binary_lb dist/P4_1 dist/P4_2 dist/P5_1 \
     --title "v0.1.0" \
     --notes "Rust ports of exercise 4 (Monte Carlo hard disks) and 5 (binary-fluid LB)."
   ```
4. **Automate with CI** (optional). A `.github/workflows/release.yml` that triggers
   on version tags and runs the musl cross-build will produce the same artifacts
   on every tag. Not required for the first release.

## For colleagues — running a downloaded binary

No install required. Download the binary from the release, mark it executable,
and run it:

```sh
chmod +x binary_lb
./binary_lb spinodal --temperature 0.3 --steps 20000 --output hist.json
```

The output `hist.json` is the same schema `run_and_collect` returns in Python,
so existing analysis notebooks continue to work:

```python
import json, numpy as np
d = json.load(open("hist.json"))
phi_final = np.asarray(d["phi"][-1])
print(f"phi range: [{phi_final.min():+.3f}, {phi_final.max():+.3f}]")
```

## Performance notes

At 128×128 on a typical laptop CPU (single-threaded, best-of-8):

| Backend | ms/step |
|---|---|
| Python `numpy.fft` (reference) | ~17 |
| Rust f64 | ~4 |
| Rust f32 | ~3.5 |

~3–5× speedup end-to-end. Further headroom is in replacing the FFT-based
semi-implicit Cahn-Hilliard solve (~50% of runtime) with a Jacobi or realfft
implementation, and/or Rayon for parameter sweeps.
