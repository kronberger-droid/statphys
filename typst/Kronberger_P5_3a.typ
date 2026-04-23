#import "@preview/lilaq:0.5.0" as lq

#let data = json("../data/P5_1/nucleation.json")

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

_(to be filled in on paper)_

- Runs 1 and 2 (both $T = 0.525$): metastable regime — system sits between binodal and
  spinodal. Nucleation is stochastic: clusters appear, sometimes shrink, and only one
  (or a few) grow past the critical radius. Identical parameters, different seeds → very
  different trajectories.
- Run 3 ($T = 0.45$): quenched deep into the spinodal. No barrier; the system breaks
  into the new phase everywhere simultaneously. Largest cluster grows monotonically and
  quickly.
- *Why no metastability in the symmetric case* (sfrac $= 0.5$): the composition sits at
  the maximum of the binodal curve where the binodal and spinodal meet. There is no
  free-energy barrier, so the system is always unstable below $T_c$. Metastability requires
  $phi_"spin" < |phi_0| < phi_"bin"$, which only happens at asymmetric compositions.
- *Can we predict when nucleation starts/ends?* No — only the statistical distribution is
  predictable. The critical-nucleus formation time is exponential in $Delta F^*\/k T$, so
  the waiting time has a long tail and finite samples look very random.

