# Build, plot, and preview a statphys exercise.
# Blocks on typst watch. Ctrl+C kills watch and closes zathura.
# Usage: run P1_1a
def main [name: string] {
  let typ = $"typst/Kronberger_($name).typ"
  let pdf = ($"output/Kronberger_($name).pdf" | path expand)

  # Map exercise names to binary + subcommand
  let task = match $name {
    "P4_1_1a" => { bin: "P4_1", sub: "timing" }
    "P4_1_1b" => { bin: "P4_1", sub: "acceptance" }
    "P4_1_2a" => { bin: "P4_1", sub: "packing" }
    "P4_1_2b" => { bin: "P4_1", sub: "henderson" }
    "P4_2a"   => { bin: "P4_2", sub: "energy" }
    "P4_2b"   => { bin: "P4_2", sub: "rdf" }
    _ => {
      let bin = $name | str replace -r '[a-z]$' ''
      { bin: $bin, sub: null }
    }
  }

  if $task.sub != null {
    print $"(ansi cyan)Building(ansi reset) cargo run --bin ($task.bin) -- ($task.sub)"
    cargo run --release --bin $task.bin -- $task.sub
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
