#import "@preview/lilaq:0.5.0" as lq

#let data = json("data/nucleation.json")

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

The two observables answer different questions:

- *Largest cluster (3a):* max over connected components — size of the _biggest_
  minority domain. Jumps on merges, resets when a sub-critical nucleus dies.
  Close to a percolation / critical-nucleus observable.
- *Minority-cell count (this plot):* total number of minority cells regardless of
  connectivity — the _volume_ of the new phase. Smooth and monotonic once
  nucleation has happened.

*Comparison.* For the deep-quench spinodal run ($T = 0.45$) the two curves look
almost identical because one dominant domain contains essentially all the minority
phase. For the metastable runs ($T = 0.525$) the cluster signal is noisy (jumps, resets)
while the cell count grows smoothly.

*Which to use.* "When did a stable nucleus form?" → largest cluster. "How much of the
system has flipped?" → cell count.

