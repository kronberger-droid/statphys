#import "@preview/lilaq:0.5.0" as lq
#let data = json("data/P4_1/packing_rust.json")

#set page(flipped: true)

#show lq.selector(lq.diagram): set align(center + horizon)
#show lq.selector(lq.title): set text(size: 18pt)
#show lq.selector(lq.label): set text(size: 14pt)

#align(center)[#text(size: 20pt)[Kronberger\_P4\_1\_2a: Packing Fraction Equilibration]]

#figure(
  box(
    height: 85%,
    lq.diagram(
      title: [Packing fraction vs. sweeps (NPT)],
      width: 100%,
      height: 100%,
      xlabel: [sweep],
      ylabel: [$phi$ (packing fraction)],
      legend: (position: top + right),
      cycle: (lq.color.map.petroff6),
      ..data.curves.map(c => lq.plot(
        c.sweeps,
        c.packing_fractions,
        mark: none,
        stroke: 1.5pt,
        label: [$P = #c.pressure$],
      )),
    ),
  ),
  caption: [Packing fraction $phi = N pi (sigma\/2)^2 / L^2$ as a function of MC sweeps for NPT simulations with $N = 64$ particles at pressures $P = 1, 2, 5$.],
)
