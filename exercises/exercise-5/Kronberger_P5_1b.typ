#import "@preview/lilaq:0.5.0" as lq

#let data = json("data/timesteps.json")

#set page(flipped: true, margin: (x: 1.5cm, y: 1.5cm))

#show lq.selector(lq.title): set text(size: 10pt)
#show lq.selector(lq.label): set text(size: 9pt)
#show: lq.set-diagram(width: 5cm, height: 5cm)

#align(center)[#text(size: 14pt, weight: "bold")[
  Kronberger\_P5\_1b: Timestep sweep ($T = 0.45$, sfrac = 0.5)
]]
#v(0.3em)

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
  let mesh = lq.colormesh(
    xedges,
    yedges,
    z,
    map: color.map.turbo,
  )
  figure(
    stack(
      dir: ltr,
      spacing: 0.4em,
      lq.diagram(
        title: [$d t = #s.params.dt$ #if unstable [— *unstable*]],
        xlabel: [x],
        ylabel: [y],
        mesh,
      ),
      lq.colorbar(mesh, label: $phi$),
    ),
    caption: [$phi_"final"$ at $d t = #s.params.dt$
      #if unstable [(NaN)]],
  )
}

#grid(
  columns: (1fr, 1fr, 1fr),
  rows: (auto, auto),
  column-gutter: 0.8em,
  row-gutter: 0.6em,
  ..data.snapshots.map(panel),
)

#pagebreak()

= Task 1b — interpretation

The Cahn–Hilliard equation is stiff, so $d t$ trades speed against stability.

- *$d t = 0.01$:* stable but under-evolved. Total simulated time $N dot d t = 200$ is
  tiny, so we only see the first linear spinodal modes growing out of the initial noise.
- *$d t = 0.3, 1, 2$:* all stable. $phi$ has saturated at $plus.minus phi_"bin"$ and the
  morphology is co-continuous, qualitatively like the $T = 0.45$ panel of 1a.
- *$d t = 10$:* blows up to NaN. The panel is tagged _unstable_.

*Why large $d t$ explodes.* CH has a fourth-order operator $M kappa nabla^4 phi$. Explicit
time stepping requires
$d t lt.tilde (d x)^4 \/ (M kappa)$; above that bound the highest-$k$ modes amplify instead
of decay and the field diverges in a few steps. Other limits (CFL, $tau > 0.5$,
$|phi| < 1$) normally don't bite first.

*How to study stability systematically.* Either linearise the update and do _von Neumann
analysis_ (compute the amplification factor $g(k, d t)$ and find where $|g| > 1$ first),
or sweep $(d t, M kappa)$ empirically and mark the blow-up boundary — it comes out a
straight line in log-log with slope $-1$, matching the linear prediction.
