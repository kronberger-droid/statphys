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
│   └── bin/
│       └── P<sheet>_<ex>.rs                # simulation binaries
├── typst/
│   └── Kronberger_P<sheet>_<ex><part>.typ  # plot files
├── data/                                   # generated JSON (gitignored)
├── output/                                 # generated PDFs
├── exercises/                              # problem sheets
├── run.nu                                  # build + preview script (optional)
└── flake.nix                               # nix dev environment
```

## Dependencies

- **Rust**: rustup (dependencies defined in src/Cargo.toml)
- **Typst**: lilaq
- **Dev**: nix (flakes enabled)
