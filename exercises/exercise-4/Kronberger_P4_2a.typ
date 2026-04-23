#import "@preview/lilaq:0.5.0" as lq

#let states = (
  "rho0.05-T0.5",
  "rho0.05-T1.0",
  "rho0.3-T0.5",
  "rho0.3-T1.0",
)
#let data = states.map(s => json("data/P4_2/energy_" + s + ".json"))

#set page(flipped: true)

#show lq.selector(lq.diagram): set align(center + horizon)
#show lq.selector(lq.title): set text(size: 14pt)
#show lq.selector(lq.label): set text(size: 11pt)

#align(center)[#text(size: 20pt)[Kronberger\_P4\_2a: Potential Energy along Trajectory]]

#let energy-plot(d) = {
  box(
    height: 100%,
    lq.diagram(
      title: [$rho = #d.rho, T = #d.temperature$],
      width: 100%,
      height: 100%,
      xlabel: [timestep],
      ylabel: [$U\/N$],
      lq.plot(
        d.timesteps,
        d.energy_per_particle,
        mark: none,
        stroke: 1pt,
        label: [MC],
      ),
      lq.plot(
        (d.timesteps.first(), d.timesteps.last()),
        (d.average_energy, d.average_energy),
        mark: none,
        stroke: (dash: "dashed", thickness: 1.5pt),
        color: red,
        label: [$chevron.l U\/N chevron.r_"ref"$],
      ),
    ),
  )
}

#grid(
  columns: (1fr, 1fr),
  rows: (1fr, 1fr),
  column-gutter: 2em,
  row-gutter: 3em,
  ..data.map(d => figure(
    energy-plot(d),
    caption: [$rho = #d.rho$, $T = #d.temperature$, $chevron.l U\/N chevron.r_"ref" = #calc.round(d.average_energy, digits: 3)$],
  ))
)
