#import "@preview/lilaq:0.5.0" as lq

#let short = json("data/temperatures.json")
#let long = json("data/temperatures_long.json")

#set page(flipped: true, margin: (x: 1.5cm, y: 1.5cm))

#show lq.selector(lq.title): set text(size: 11pt)
#show lq.selector(lq.label): set text(size: 10pt)
#show: lq.set-diagram(width: 7cm, height: 7cm)

#let ny = short.snapshots.at(0).phi_final.len()
#let nx = short.snapshots.at(0).phi_final.at(0).len()
#let xedges = range(nx)
#let yedges = range(ny)

#let panel(s) = {
  let p = s.params
  let mesh = lq.colormesh(
    xedges,
    yedges,
    s.phi_final,
    map: color.map.turbo,
  )
  figure(
    stack(
      dir: ltr,
      spacing: 0.4em,
      lq.diagram(
        title: [$T = #p.T$ ($T\/T_c = #calc.round(p.T / (p.lam / 2.0), digits: 3)$)],
        xlabel: [x],
        ylabel: [y],
        mesh,
      ),
      lq.colorbar(mesh, label: $phi$),
    ),
    caption: [$phi_"final"$ — $d t = #p.dt$, steps $= #p.steps$],
  )
}

#let page-of(collection, title) = {
  align(center)[#text(size: 14pt, weight: "bold")[#title]]
  v(0.3em)
  grid(
    columns: (1fr, 1fr, 1fr),
    column-gutter: 0.8em,
    ..collection.snapshots.map(panel),
  )
}

#page-of(short, [Kronberger\_P5\_1a: Temperature sweep (sfrac = 0.5, short run)])

#pagebreak()

#page-of(long, [Kronberger\_P5\_1a: Temperature sweep (sfrac = 0.5, long run)])

#align(center)[
  #text(size: 10pt, style: "italic")[
    Short-run morphology (page 1) is seed-dependent; the long-run state is
    seed-insensitive and coarsens toward the single-interface equilibrium.
  ]
]

#pagebreak()

= Task 1a — interpretation

With $lambda = 1.1$ the critical temperature is $T_C = lambda\/2 = 0.55$, so the three
temperatures sit at $T\/T_C = 1.0, 0.818, 0.545$.

- *$T = 0.55 = T_C$:* no phase separation. The free-energy double well is gone,
  only small critical fluctuations remain — the panel looks like the washed-out initial
  noise.
- *$T = 0.45, 0.3$ (below $T_C$):* spinodal decomposition. The mixed state is unstable
  and the field jumps to the coexisting values $plus.minus phi_"bin"(T)$ everywhere at once.
  Because $chevron.l phi chevron.r = 0$ the two phases occupy ~50% of the area, so we get the
  _co-continuous / labyrinth_ morphology. Deeper quench → larger $phi_"bin"$ → sharper
  contrast at $T = 0.3$.

*Equilibrium prediction.* On a periodic box the configuration with the shortest total
wall is a single flat interface (one stripe cutting the box in half). The short-run
panels are local minima with many interfaces — metastable.

*Long run.* After $200000$ steps the system has coarsened into far fewer, bigger
domains, but has still not reached the single-stripe state. Coarsening is slow and
follows a power law (see task 2a): spinodal decomposition happens fast, full
equilibration takes forever.
