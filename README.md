# statphys

Statistical physics 2 exercises (TU Wien), implemented in Rust with Typst/Lilaq for plotting.

## Usage

```sh
nix develop
run P1_1a         # build, compile typst, open zathura
run P1_1a --kill  # stop typst watch
```

## Structure

```
statphys/
├── src/
│   ├── lib.rs                              # shared helpers
│   ├── lb/                                 # binary-fluid LB library (exercise 5)
│   ├── mc/                                 # hard-disks MC library (exercise 4)
│   └── bin/
│       ├── binary_lb/                      # standalone LB binary (wraps src/lb)
│       ├── hard_disks_mc/                  # standalone MC binary (wraps src/mc)
│       └── P<sheet>_<ex>/                  # exercise-specific task presets
├── exercises/
│   └── exercise-<n>/
│       ├── Aufgaben*.pdf                   # problem sheet (committed)
│       ├── *.py                            # Python reference code (committed)
│       ├── Kronberger_P*.typ               # plot sources (committed)
│       ├── Kronberger_P*.pdf               # compiled PDFs (gitignored)
│       └── data/                           # generated JSON (gitignored)
├── scripts/build-dist.nu                   # local standalone-binary builder
├── .github/workflows/release.yml           # multi-platform release on `v*` tags
├── run.nu                                  # build + preview script (optional)
└── flake.nix                               # nix dev environment
```

## Dependencies

- **Rust**: rustup (dependencies defined in `Cargo.toml`)
- **Typst**: lilaq
- **Dev**: nix (flakes enabled)

---

## Quick start for colleagues — no Rust install

Two standalone binaries are published on the [Releases page](../../releases) that are
drop-in replacements for the Python reference simulations the course ships:

| Python reference | Rust binary |
|---|---|
| `hard_disks_mc.py::run_simulation` | `hard_disks_mc` |
| `binary_LB.py::make_{spinodal,metastable}_example` + `run_and_collect` | `binary_lb` |

Same parameter names, same JSON output schema. Download the binary for your
platform (Linux-musl / macOS x86_64 / macOS arm64 / Windows), mark it
executable, and run it directly — no Python or libpython install required.

### CLI surface

```
hard_disks_mc [--n-particles --t-end --temperature --box-length --pressure
               --sigma --epsilon --max-displacement --max-delta-log-area
               --ensemble {nvt,npt} --save-every --seed
               --initialization {square,random} --output]

binary_lb spinodal   [--nx --ny --n0 --lam --temperature --kappa --mobility
                      --tau --dt --spinodal-fraction --phi-noise
                      --no-hydrodynamics --seed --steps --snapshot-every --output]
binary_lb metastable [... --fraction-of-binodal --kt ...]
binary_lb bench      [--nx --ny --steps]
```

Parameter names mirror the Python APIs (only `T` → `--temperature`,
`M` → `--mobility`, `kT` → `--kt`, `max_delta_log_volume` → `--max-delta-log-area`).
`binary_lb` also accepts `--precision f32|f64` (default f64); f32 is ~1.5× faster
on the LB binary and matches f64 on phi extrema to ~3e-4 at 128² grids.

### Using from a Python script

Progress lines go to stderr and the JSON payload to stdout. Pass `--output -` to
stream JSON on stdout instead of writing a file, then parse with `json.loads`.

```python
import json, subprocess
import numpy as np

proc = subprocess.run(
    ["./binary_lb", "spinodal",
     "--temperature", "0.3", "--steps", "20000", "--snapshot-every", "500",
     "--seed", "1", "--output", "-"],
    capture_output=True, check=True, text=True,
)
history = json.loads(proc.stdout)             # same dict `run_and_collect` returns
phi = np.asarray(history["phi"][-1])
print(f"phi range at t={history['times'][-1]:.0f}: "
      f"[{phi.min():+.3f}, {phi.max():+.3f}], "
      f"binodal=±{history['phi_binodal']:.3f}")
```

Hard-disk MC is identical:

```python
proc = subprocess.run(
    ["./hard_disks_mc",
     "--ensemble", "npt", "--pressure", "2.0",
     "--n-particles", "64", "--t-end", "5000", "--save-every", "10",
     "--output", "-"],
    capture_output=True, check=True, text=True,
)
result = json.loads(proc.stdout)              # matches hard_disks_mc.MonteCarloResult
trajectory = np.asarray(result["trajectory"]) # shape (n_frames, n_particles, 2)
```

### Drop-in replacement helpers

Copy this into your analysis script and usage is within one character of the
Python API:

```python
import json, subprocess
from pathlib import Path

LB_BIN   = Path("./binary_lb")
HDMC_BIN = Path("./hard_disks_mc")

# CLI flags the binaries expose as presence-only switches (default off).
# When a user passes `--no-hydrodynamics` / `hydrodynamics=False`, we still
# want the flag emitted, so these are normalized to `no_<name>` below.
_INVERTED_BOOL_FLAGS = {"hydrodynamics"}

def _call(argv, **kwargs):
    args = list(argv) + ["--output", "-"]
    for k, v in kwargs.items():
        if k in _INVERTED_BOOL_FLAGS:           # hydrodynamics=False → --no-hydrodynamics
            if not v:
                args.append(f"--no-{k.replace('_', '-')}")
            continue
        flag = "--" + k.replace("_", "-")
        if isinstance(v, bool):
            if v:
                args.append(flag)
        else:
            args += [flag, str(v)]
    proc = subprocess.run(args, capture_output=True, check=True, text=True)
    return json.loads(proc.stdout)

def run_spinodal(**kwargs):
    """Drop-in replacement for make_spinodal_example(...) + run_and_collect."""
    return _call([str(LB_BIN), "spinodal"], **kwargs)

def run_metastable(**kwargs):
    return _call([str(LB_BIN), "metastable"], **kwargs)

def run_hard_disks(**kwargs):
    """Drop-in replacement for hard_disks_mc.run_simulation(...)."""
    return _call([str(HDMC_BIN)], **kwargs)
```

```python
hist = run_spinodal(temperature=0.3, steps=20000, snapshot_every=500, seed=1)
res  = run_hard_disks(ensemble="npt", pressure=2.0, n_particles=64, t_end=5000)
```

### Notebook workflow

In Jupyter, stream stderr so colleagues see the `=== Spinodal ===` progress
banner while the simulation runs:

```python
proc = subprocess.Popen(
    [str(LB_BIN), "spinodal", "--steps", "100000", "--output", "-"],
    stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
)
for line in proc.stderr:
    print(line, end="")
proc.wait()
history = json.loads(proc.stdout.read())
```

### Output schema

`binary_lb` → same keys as `binary_LB.run_and_collect`:

```
{ "times": [...], "steps": [...], "phi": [[[...]]],
  "phi_mean": [...], "params": {...},
  "Tc": float, "phi_binodal": float, "phi_spinodal": float }
```

`hard_disks_mc` → same keys as `hard_disks_mc.MonteCarloResult`:

```
{ "positions": [[x, y], ...],
  "trajectory": [[[x, y], ...], ...],
  "box_lengths": [...], "energies": [...], "saved_sweeps": [...],
  "move_acceptance": float, "volume_acceptance": float,
  "metadata": { "n_particles": ..., "temperature": ..., ... } }
```

### Performance note on JSON

JSON inflates large phi grids ~3–5× vs raw f64 bytes, and `json.load` on a
400 MB output (128² × 1000 frames) takes several seconds. For long runs,
increase `--snapshot-every` so fewer frames are serialized.

---

## Building releases

Tag a version and push; GitHub Actions builds `hard_disks_mc` and `binary_lb`
for Linux (musl), macOS (x86_64 + arm64), and Windows, and attaches the
artifacts to the Release.

```sh
git tag -a v0.1.0 -m "hard-disks MC + binary-fluid LB Rust ports"
git push origin v0.1.0
```

For a local-only build:

```nu
nu scripts/build-dist.nu binary_lb           # dynamic libc (~2.6 MB)
nu scripts/build-dist.nu binary_lb --musl    # fully static
```
