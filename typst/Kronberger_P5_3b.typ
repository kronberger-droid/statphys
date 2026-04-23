#import "@preview/lilaq:0.5.0" as lq

#let data = json("../data/P5_1/nucleation.json")

#set page(flipped: true)

#show lq.selector(lq.diagram): set align(center + horizon)
#show lq.selector(lq.title): set text(size: 16pt)
#show lq.selector(lq.label): set text(size: 12pt)

#align(center)[#text(size: 20pt)[Kronberger\_P5\_3b: Minority-phase cell count]]

#let curves = data.curves

#figure(
  box(
    height: 85%,
    lq.diagram(
      title: [Total minority-phase cells vs. time],
      width: 100%,
      height: 100%,
      xlabel: [$t$],
      ylabel: [cells with $phi$ past the minority threshold],
      legend: (position: top + left),
      cycle: (lq.color.map.petroff6),
      ..curves.enumerate().map(((i, c)) => lq.plot(
        c.times,
        c.minority_count,
        mark: none,
        stroke: 1.2pt,
        label: [run #{i + 1}: $T = #c.temperature$],
      )),
    ),
  ),
  caption: [
    Same three runs as P5\_1\_3a. Counted are all cells whose $phi$ lies on the minority side of the binodal threshold $phi_"bin"(T)$, without grouping into clusters. Unlike 3a this observable grows monotonically with the extent of the new phase regardless of how it is distributed over separate nuclei.
  ],
)

#pagebreak()

= Task 3b — interpretation

_(to be filled in on paper)_

Comparison of the two observables:

- *Largest cluster* (3a): max over connected components. Grows with the biggest nucleus;
  can plateau or shrink when a sub-critical nucleus dissolves, or jump when two nuclei
  merge. It measures the *size of the dominant domain*.
- *Minority cell count* (this plot): sums all minority cells regardless of connectivity.
  It measures the *total volume* of the new phase.

Physically the second is closer to an order-parameter / concentration measurement,
the first is closer to a percolation / critical-nucleus measurement.

For the deep-quench (run 3) spinodal case the two look similar because one growing
cluster dominates. For the metastable runs (1, 2) the cluster signal is noisier — it
can fluctuate or reset when a nucleus dies — whereas the cell count climbs monotonically
with the growing phase volume.

