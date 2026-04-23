# Build simulation data and compile a typst plot for an exercise subpoint.
# Usage:
#   run P5_1a                   # run simulation + compile PDF
#   run P5_1a --skip-sim        # only recompile the PDF (skip cargo)
#   run P5_1a --open            # spawn zathura on the PDF after compile
#   run P5_1a --both-backends   # force both rust + python backend runs
#
# Typst watching is intentionally not part of this script — run
# `typst watch --root . exercises/<folder>/<name>.typ <pdf>` separately
# (or a per-folder wrapper) when you're editing prose in a typ file.
def main [
  name: string,
  --skip-sim,        # skip cargo, just recompile the PDF
  --open,            # open zathura on the PDF after compile
  --both-backends,   # run both rust + python backends (if task supports it)
] {
  # Map exercise name -> (cargo bin, subcommand, exercise folder, default both-backends).
  # A `null` sub means the bin has no subcommand (P1_*).
  let task = match $name {
    "P4_1_1a" => { bin: "P4_1", sub: "timing",         folder: "exercise-4", both: true  }
    "P4_1_1b" => { bin: "P4_1", sub: "acceptance",     folder: "exercise-4", both: false }
    "P4_1_2a" => { bin: "P4_1", sub: "packing",        folder: "exercise-4", both: false }
    "P4_1_2b" => { bin: "P4_1", sub: "henderson",      folder: "exercise-4", both: true  }
    "P4_2a"   => { bin: "P4_2", sub: "energy",         folder: "exercise-4", both: false }
    "P4_2b"   => { bin: "P4_2", sub: "rdf",            folder: "exercise-4", both: false }
    "P5_1a"   => { bin: "P5_1", sub: "temperatures",   folder: "exercise-5", both: false }
    "P5_1b"   => { bin: "P5_1", sub: "timesteps",      folder: "exercise-5", both: false }
    "P5_1c"   => { bin: "P5_1", sub: "asymmetric",     folder: "exercise-5", both: false }
    "P5_2a"   => { bin: "P5_1", sub: "domain-growth",  folder: "exercise-5", both: false }
    "P5_3a"   => { bin: "P5_1", sub: "nucleation",     folder: "exercise-5", both: false }
    "P5_3b"   => { bin: "P5_1", sub: "minority-count", folder: "exercise-5", both: false }
    _ => {
      # Fallback: P<sheet>_<ex><part> -> bin strips trailing letter, folder from sheet digit.
      let bin = $name | str replace -r '[a-z]$' ''
      let sheet = $name | parse -r 'P(?<n>\d+)_' | get 0.n?
      { bin: $bin, sub: null, folder: $"exercise-($sheet)", both: false }
    }
  }

  let run_both = ($both_backends or $task.both)
  let typ = $"exercises/($task.folder)/Kronberger_($name).typ"
  let pdf = $"exercises/($task.folder)/Kronberger_($name).pdf"

  if not $skip_sim {
    if $task.sub != null {
      print $"(ansi cyan)Simulating(ansi reset) cargo run --bin ($task.bin) -- ($task.sub)"
      cargo run --release --bin $task.bin -- $task.sub
      if $run_both {
        print $"(ansi cyan)Simulating(ansi reset) cargo run --bin ($task.bin) -- ($task.sub) --backend python"
        cargo run --release --bin $task.bin -- $task.sub --backend python
      }
    } else {
      print $"(ansi cyan)Simulating(ansi reset) cargo run --bin ($task.bin)"
      cargo run --release --bin $task.bin
    }
  }

  print $"(ansi cyan)Compiling(ansi reset) ($typ)"
  typst compile --root . $typ $pdf

  if $open {
    let windows = niri msg -j windows | from json
    let pdf_abs = ($pdf | path expand)
    let not_open = ($windows | where app_id == "org.pwmt.zathura" | where title =~ $pdf_abs | is-empty)
    if $not_open {
      print $"(ansi cyan)Opening(ansi reset) zathura on ($pdf)"
      niri msg action spawn -- zathura $pdf_abs
    } else {
      print $"(ansi yellow)zathura already open on ($pdf)(ansi reset)"
    }
  }
}
