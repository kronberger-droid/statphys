#import "@preview/lilaq:0.5.0" as lq

#let short = json("data/temperatures.json")
#let long = json("data/temperatures_long.json")

#set page(flipped: true)

#show lq.selector(lq.diagram): set align(center + bottom)
#show lq.selector(lq.title): set text(size: 12pt)
#show lq.selector(lq.label): set text(size: 10pt)

#let ny = short.snapshots.at(0).phi_final.len()
#let nx = short.snapshots.at(0).phi_final.at(0).len()
#let xedges = range(nx)
#let yedges = range(ny)

#let panel(s) = {
  let p = s.params
  figure(
    lq.diagram(
      title: [$T = #p.T$ ($T\/T_c = #calc.round(p.T / (p.lam / 2.0), digits: 3)$)],
      width: 100%,
      height: 50%,
      xlabel: [x],
      ylabel: [y],
      lq.colormesh(
        xedges,
        yedges,
        s.phi_final,
        map: color.map.turbo,
      ),
    ),
    caption: [$phi_"final"$\ $lambda = #p.lam$, $d t = #p.dt$, steps $= #p.steps$],
  )
}

#let page-of(collection, title) = {
  align(center)[#text(size: 20pt)[#title]]

  grid(
    columns: (1fr, 1fr, 1fr),
    column-gutter: 1em,
    ..collection.snapshots.map(panel),
  )
}

#page-of(short, [Kronberger\_P5\_1a: Temperature sweep (sfrac = 0.5, short run)])

#pagebreak()

#page-of(long, [Kronberger\_P5\_1a: Temperature sweep (sfrac = 0.5, long run)])

#align(center)[
  #text(size: 10pt, style: "italic")[
    Short-run morphology (page 1) is seed-dependent — different random initial
    conditions trap the system in different metastable domain patterns. The
    long-run state (this page) is seed-insensitive and coarsens toward the
    single-interface equilibrium regardless of the starting noise.
  ]
]

#pagebreak()

= Task 1a — interpretation

_(to be filled in on paper)_

- $T < T_C$ (panels at $T = 0.45$, $0.3$): spinodal decomposition — $phi$ splits into
  the two coexisting phases ($plus.minus phi_"bin"(T)$) separated by narrow domain walls of
  width $~ sqrt(kappa \/ |psi''|)$. Morphology is co-continuous / bicontinuous because
  the mean order parameter $chevron.l phi chevron.r$ is zero.
- $T = T_C$ (panel at $T = 0.55$): no phase separation — critical fluctuations only,
  bounded by initial noise amplitude.
- Equilibrium prediction: a single flat interface cutting the box into two halves
  minimises total wall length under periodic boundaries. The co-continuous state on
  page 1 is metastable — many interfaces, all local minima.
- Long-run panel (page 2) confirms coarsening toward few big drops / stripes, but still
  has not reached the single-interface state: $kappa$ is finite so every configuration with
  domain walls is locally stable.

