#import "@preview/lilaq:0.5.0" as lq

#let data = json("data/nucleation.json")

#set page(flipped: true)

#show lq.selector(lq.diagram): set align(center + horizon)
#show lq.selector(lq.title): set text(size: 16pt)
#show lq.selector(lq.label): set text(size: 12pt)

#align(center)[#text(size: 20pt)[Kronberger\_P5\_3a: Largest minority cluster (nucleation)]]

#let curves = data.curves

#figure(
  box(
    height: 85%,
    lq.diagram(
      title: [Largest connected minority cluster vs. time],
      width: 100%,
      height: 100%,
      xlabel: [$t$],
      ylabel: [largest-cluster size (cells)],
      legend: (position: top + left),
      cycle: (lq.color.map.petroff6),
      ..curves.enumerate().map(((i, c)) => lq.plot(
        c.times,
        c.largest_cluster,
        mark: none,
        stroke: 1.2pt,
        label: [run #{i + 1}: $T = #c.temperature$],
      )),
    ),
  ),
  caption: [
    Three metastable runs at fraction-binodal $= 0.2$, $kappa = 0.2$, $tau = 0.9$, $M = 0.2$, $k T = 0.004$, 200000 steps ($d t = 0.5$), hydrodynamics off. Runs 1 and 2 share $T = 0.525$ with different seeds; run 3 is at $T = 0.45$. Threshold is the binodal $phi_"bin"(T)$.
  ],
)

#pagebreak()

= Task 3a — interpretation

All three runs sit at fraction-binodal $= 0.2$ with thermal noise $k T = 0.004$ on.

- *Runs 1, 2 — $T = 0.525$, different seeds.* Both are in the metastable strip between
  binodal and spinodal. A barrier $Delta F^*$ separates the homogeneous state from
  phase separation, and nucleation is a stochastic barrier-crossing event with waiting
  time $t_"nuc" prop e^(Delta F^* \/ k T)$. Same parameters, different seeds → wildly
  different trajectories: one may nucleate early and grow, the other stalls as
  sub-critical nuclei keep dying. This is the fingerprint of metastability.
- *Run 3 — $T = 0.45$.* Deeper quench pushes past the spinodal, no barrier, everything
  grows at once. Largest cluster climbs monotonically and is seed-insensitive.

*Why no metastability at sfrac $= 0.5$.* At $phi_0 = 0$ the binodal and spinodal curves
meet at the top of the dome: $Delta F^* = 0$. Any quench below $T_C$ is automatically
inside the spinodal. Metastability needs $phi_"spin" < |phi_0| < phi_"bin"$, i.e. an
asymmetric composition.

*Predictable?* Only statistically. Nucleation _start_ is exponentially distributed,
variance = mean, so realisations span many decades. Once the critical nucleus is there,
growth is reasonably deterministic.

