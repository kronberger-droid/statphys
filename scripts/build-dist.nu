# Build a standalone (no libpython) release of a statphys binary.
#
# Usage:
#   nu scripts/build-dist.nu P5_1          # dynamic libc, requires glibc compat
#   nu scripts/build-dist.nu P5_1 --musl   # fully static, runs on any Linux
def main [
  binary: string          # e.g. "P5_1" or "P4_1"
  --musl                  # produce a fully-static musl build
  --out-dir: string = "dist"
] {
  mkdir $out_dir
  let target_args = if $musl { ["--target", "x86_64-unknown-linux-musl"] } else { [] }
  let target_subdir = if $musl { "x86_64-unknown-linux-musl/release-dist" } else { "release-dist" }

  print $"(ansi cyan)Building(ansi reset) ($binary) with --profile release-dist --no-default-features ...($target_args | str join ' ')"
  cargo build --profile release-dist --no-default-features --bin $binary ...$target_args

  let src = $"target/($target_subdir)/($binary)"
  let dst = $"($out_dir)/($binary)"
  cp $src $dst
  let size = (ls $dst | first | get size)
  print $"(ansi green)Wrote(ansi reset) ($dst) \(($size)\)"

  # Sanity: list dynamic dependencies.
  print $"(ansi cyan)Dynamic deps:(ansi reset)"
  ldd $dst
}
