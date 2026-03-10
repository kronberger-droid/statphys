# Build, plot, and preview a statphys exercise.
# Usage: run P1_1a
#        run P1_1a --kill  (stop typst watch for this file)
def main [name: string, --kill] {
  if $kill {
    let procs = ps | where name == "typst" | where command =~ $name
    if ($procs | is-empty) {
      print $"(ansi yellow)No typst watch(ansi reset) running for ($name)"
    } else {
      $procs | each { |p| kill $p.pid }
      print $"(ansi red)Killed(ansi reset) typst watch for ($name)"
    }
    return
  }

  let bin = $name | str replace -r '[a-z]$' ''
  let typ = $"typst/Kronberger_($name).typ"
  let pdf = ($"output/Kronberger_($name).pdf" | path expand)

  print $"(ansi cyan)Building(ansi reset) cargo run --bin ($bin)"
  cargo run --bin $bin

  # Check if typst watch is already running for this file
  let watch_running = (ps | where name == "typst" | where command =~ $name | length) > 0

  if not $watch_running {
    print $"(ansi cyan)Starting(ansi reset) typst watch ($typ)"
    bash -c $"typst watch --root ../ ($typ) ($pdf) > /dev/null 2>&1 &"
  } else {
    print $"(ansi green)Typst watch(ansi reset) already running"
  }

  # Give typst watch a moment to produce the first PDF
  sleep 300ms

  let windows = niri msg -j windows | from json
  let not_open = ($windows | where app_id == "org.pwmt.zathura" | where title =~ $pdf | is-empty)

  if $not_open {
    print $"(ansi cyan)Opening(ansi reset) zathura"
    niri msg action spawn -- zathura $pdf
    sleep 500ms
    niri msg action consume-or-expel-window-right
    niri msg action focus-column-left
  } else {
    print $"(ansi green)Zathura(ansi reset) already open, reloading"
  }
}
