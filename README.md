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
src/bin/P<sheet>_<ex>.rs                  # Rust simulation binaries
typst/Kronberger_P<sheet>_<ex><part>.typ  # Typst plot files
data/                                     # Generated JSON (gitignored)
output/                                   # Generated PDFs
exercises/                                # Problem sheets
```

## Dependencies

- **Rust**: rand, rand_distr, ndarray, statrs, serde, serde_json
- **Typst**: lilaq
- **System**: nix (flake), typst, zathura, niri
