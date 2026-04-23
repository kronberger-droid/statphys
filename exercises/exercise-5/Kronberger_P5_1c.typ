#import "@preview/lilaq:0.5.0" as lq

#let data = json("data/asymmetric.json")

#set page(flipped: true, margin: (x: 1.5cm, y: 1.5cm))

#show lq.selector(lq.title): set text(size: 11pt)
#show lq.selector(lq.label): set text(size: 10pt)
#show: lq.set-diagram(width: 7cm, height: 7cm)

#align(center)[#text(size: 14pt, weight: "bold")[
  Kronberger\_P5\_1c: Asymmetric sweep ($T = 0.4$)
]]
#v(0.3em)

#let ny = data.snapshots.at(0).phi_final.len()
#let nx = data.snapshots.at(0).phi_final.at(0).len()
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
        title: [sfrac $= #p.spinodal_fraction$],
        xlabel: [x],
        ylabel: [y],
        mesh,
      ),
      lq.colorbar(mesh, label: $phi$),
    ),
    caption: [$phi_"final"$ at $T = #p.T$, sfrac $= #p.spinodal_fraction$],
  )
}

#grid(
  columns: (1fr, 1fr, 1fr),
  column-gutter: 0.8em,
  ..data.snapshots.map(panel),
)

#pagebreak()

= Task 1c — interpretation

At $T = 0.4$ ($T\/T_C = 0.727$) the binodal and spinodal curves split the
$phi$-axis into three regions. `sfrac` sets the mean composition as
$phi_0 = (2 dot #raw("sfrac") - 1) phi_"spin"$, so sweeping sfrac $= 0.4, 0.2, 0.1$ walks
us progressively away from the symmetric point.

- *sfrac $= 0.4$ — inside the spinodal.* $phi_0 approx -0.2 phi_"spin"$. No barrier;
  every long-wavelength mode grows: classic spinodal decomposition, bicontinuous
  morphology slightly biased toward the majority phase.
- *sfrac $= 0.2$ — between spinodal and binodal (metastable).* A free-energy barrier
  $Delta F^*$ separates the homogeneous state from phase-separated coexistence.
  In this preset $k T = 0$, so fluctuations can't cross the barrier and the system
  just sits near $phi_0$. With thermal noise we'd see _nucleation_ (task 3).
- *sfrac $= 0.1$ — outside the binodal.* The mixed state is now the _global_ minimum;
  there is no phase transition at this composition. The field relaxes to uniform $phi_0$.

*Upshot.* Walking along $phi_0$ takes you through all three regimes — spinodal,
metastable, stable — for the same fluid at the same temperature. Which regime you're
in is set purely by where $phi_0$ sits relative to the spinodal and binodal curves.
