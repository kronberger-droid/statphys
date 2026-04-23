#import "@preview/lilaq:0.5.0" as lq

#let data = json("../data/P5_1/asymmetric.json")

#set page(flipped: true)

#show lq.selector(lq.diagram): set align(center + horizon)
#show lq.selector(lq.title): set text(size: 12pt)
#show lq.selector(lq.label): set text(size: 10pt)

#align(center)[#text(size: 20pt)[Kronberger\_P5\_1c: Asymmetric sweep ($T = 0.4$)]]

#let ny = data.snapshots.at(0).phi_final.len()
#let nx = data.snapshots.at(0).phi_final.at(0).len()
#let xedges = range(nx)
#let yedges = range(ny)

#let panel(s) = {
  let p = s.params
  figure(
    box(
      height: 100%,
      lq.diagram(
        title: [sfrac $= #p.spinodal_fraction$],
        width: 100%,
        height: 100%,
        xlabel: [x],
        ylabel: [y],
        lq.colormesh(
          xedges,
          yedges,
          s.phi_final,
          map: color.map.turbo,
        ),
      ),
    ),
    caption: [$phi_"final"$ at $T = #p.T$, sfrac $= #p.spinodal_fraction$],
  )
}

#grid(
  columns: (1fr, 1fr, 1fr),
  column-gutter: 1em,
  ..data.snapshots.map(panel),
)

#pagebreak()

= Task 1c — interpretation

_(to be filled in on paper)_

Reading off the spinodal / binodal diagram at $T = 0.4$ ($T\/T_c = 0.727$):

- *sfrac $= 0.4$*: $phi_0 = -0.2 phi_"spin" approx -0.14$, well inside the spinodal.
  Deep instability — every Fourier mode with $k < k^*$ grows; the system forms a
  bicontinuous pattern similar to the symmetric case but slightly skewed toward the
  majority phase.
- *sfrac $= 0.2$*: $phi_0 approx -0.6 phi_"spin"$, outside the spinodal but inside the
  binodal — *metastable*. With $k T = 0$ in the spinodal preset, fluctuations cannot
  cross the nucleation barrier, so the system remains (approximately) homogeneous and
  shows only slow coarsening / static noise. With thermal noise it would nucleate.
- *sfrac $= 0.1$*: $phi_0 approx -0.8 phi_"spin"$, outside the binodal — the single
  mixed phase is *stable*, there is no phase transition, and $phi$ stays near $phi_0$
  except for transient fluctuations.

Key physical distinction: bicontinuous spinodal decomposition vs. droplet nucleation vs.
no transition. All three regimes are visible in a single temperature scan just by moving
along the spinodal axis.

