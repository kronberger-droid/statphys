#import "@preview/lilaq:0.5.0" as lq

#let data = json("../data/P5_1/timesteps.json")

#set page(flipped: true)

#show lq.selector(lq.diagram): set align(center + horizon)
#show lq.selector(lq.title): set text(size: 11pt)
#show lq.selector(lq.label): set text(size: 9pt)

#align(center)[#text(size: 20pt)[Kronberger\_P5\_1b: Timestep sweep ($T = 0.45$, sfrac = 0.5)]]

#let ny = data.snapshots.at(0).phi_final.len()
#let nx = data.snapshots.at(0).phi_final.at(0).len()
#let xedges = range(nx)
#let yedges = range(ny)

// NaN blow-ups serialize as JSON nulls — render them as 0 so the panel still
// paints, and tag the panel "unstable" so it's obvious.
#let sanitize(grid) = grid.map(row => row.map(v => if v == none { 0.0 } else { v }))
#let has-null(grid) = grid.any(row => row.any(v => v == none))

#let panel(s) = {
  let unstable = has-null(s.phi_final)
  let z = sanitize(s.phi_final)
  figure(
    box(
      height: 100%,
      lq.diagram(
        title: [$d t = #s.params.dt$ #if unstable [— *unstable*]],
        width: 100%,
        height: 100%,
        xlabel: [x],
        ylabel: [y],
        lq.colormesh(
          xedges,
          yedges,
          z,
          map: color.map.turbo,
        ),
      ),
    ),
    caption: [
      $phi_"final"$ at $d t = #s.params.dt$
      #if unstable [(NaN after blow-up — rendered as zeros)]
    ],
  )
}

#grid(
  columns: (1fr, 1fr, 1fr),
  rows: (1fr, 1fr),
  column-gutter: 1em,
  row-gutter: 2em,
  ..data.snapshots.map(panel),
)

#pagebreak()

= Task 1b — interpretation

_(to be filled in on paper)_

- $d t = 0.01$: very small step — simulation is accurate but under-evolved; barely any
  coarsening visible at the same total step count.
- $d t = 0.3, 1, 2$: all stable; $phi$ saturates at $plus.minus phi_"bin"$ with
  similar coarsening progress.
- $d t = 10$: blow-up (NaN) — shown as uniform colour. The Cahn–Hilliard semi-implicit
  scheme has a stability limit driven by $d t dot M dot kappa dot k_max^4$; beyond that
  the inversion of $1 + d t M kappa k^4$ no longer tames the stiff $nabla^4$ term.
- Other stability factors: $tau > 0.5$ (LB relaxation), advective CFL
  $|u| d t \/ d x < 1$, and $|phi| < n_0$ (enforced by the clip).
- Systematic stability study: linearise $phi_"step"$ around a flat $phi_0$ and apply
  von Neumann analysis on each Fourier mode; or scan $(d t, kappa, M)$ and tag the
  blow-up boundary.

