# Build, plot, and preview a statphys exercise.
# Blocks on typst watch. Ctrl+C kills watch and closes zathura.
# Usage: run P1_1a
def main [name: string] {
  let typ = $"typst/Kronberger_($name).typ"

  # Map exercise names to binary + subcommand
  # both_backends: run with --backend python in addition to the default rust
  let task = match $name {
    "P4_1_1a" => { bin: "P4_1", sub: "timing",     both_backends: true }
    "P4_1_1b" => { bin: "P4_1", sub: "acceptance",  both_backends: false }
    "P4_1_2a" => { bin: "P4_1", sub: "packing",     both_backends: false }
    "P4_1_2b" => { bin: "P4_1", sub: "henderson",   both_backends: true }
    "P4_2a"   => { bin: "P4_2", sub: "energy",      both_backends: false }
    "P4_2b"   => { bin: "P4_2", sub: "rdf",         both_backends: false }
    "P5_1a"   => { bin: "P5_1", sub: "temperatures",  both_backends: false }
    "P5_1b"   => { bin: "P5_1", sub: "timesteps",     both_backends: false }
    "P5_1c"   => { bin: "P5_1", sub: "asymmetric",    both_backends: false }
    "P5_2a"   => { bin: "P5_1", sub: "domain-growth", both_backends: false }
    "P5_3a"   => { bin: "P5_1", sub: "nucleation",    both_backends: false }
    "P5_3b"   => { bin: "P5_1", sub: "minority-count", both_backends: false }
    _ => {
      let bin = $name | str replace -r '[a-z]$' ''
      { bin: $bin, sub: null, both_backends: false }
    }
  }

  # Derive output folder from exercise name (P4_1_1a -> P4_1, P1_1a -> P1, P4_2a -> P4_2)
  let folder = $task.bin
  let pdf = ($"output/($folder)/Kronberger_($name).pdf" | path expand)
  mkdir $"output/($folder)"

  if $task.sub != null {
    print $"(ansi cyan)Building(ansi reset) cargo run --bin ($task.bin) -- ($task.sub)"
    cargo run --release --bin $task.bin -- $task.sub
    if $task.both_backends {
      print $"(ansi cyan)Building(ansi reset) cargo run --bin ($task.bin) -- ($task.sub) --backend python"
      cargo run --release --bin $task.bin -- $task.sub --backend python
    }
  } else {
    print $"(ansi cyan)Building(ansi reset) cargo run --bin ($task.bin)"
    cargo run --release --bin $task.bin
  }

  # Compile once so the PDF exists before opening zathura
  print $"(ansi cyan)Compiling(ansi reset) ($typ)"
  typst compile --root ../ $typ $pdf

  # Open zathura if not already open
  let windows = niri msg -j windows | from json
  let not_open = ($windows | where app_id == "org.pwmt.zathura" | where title =~ $pdf | is-empty)

  if $not_open {
    print $"(ansi cyan)Opening(ansi reset) zathura"
    niri msg action spawn -- zathura $pdf
    sleep 500ms
    niri msg action consume-or-expel-window-right
    niri msg action focus-column-left
  }

  # Block on typst watch with trap for cleanup
  print $"(ansi cyan)Watching(ansi reset) ($typ) \(Ctrl+C to stop\)"
  let bash_cmd = "trap 'pkill -f \"zathura.*" + $name + "\"' EXIT; typst watch --root ../ " + $typ + " " + $pdf
  bash -c $bash_cmd
}
