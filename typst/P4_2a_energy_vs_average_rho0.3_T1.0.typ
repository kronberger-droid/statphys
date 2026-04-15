#import "@preview/lilaq:0.5.0" as lq
#let d = json("../data/P4_2/energy_rho0.3-T1.0.json")
#set page(flipped: true)

#figure(
  box(
    height: 85%,
    lq.diagram(
      title: [Potential energy per particle — $rho = #d.rho$, $T = #d.temperature$],
      width: 100%, height: 100%,
      xlabel: [timestep], ylabel: [$U\/N$],
      lq.plot(d.timesteps, d.energy_per_particle, mark: none, stroke: 1pt),
      lq.plot(
        (d.timesteps.first(), d.timesteps.last()),
        (d.average_energy, d.average_energy),
        mark: none, stroke: (dash: "dashed", thickness: 1.5pt), color: red,
        label: [$chevron.l U\/N chevron.r_"ref" = #calc.round(d.average_energy, digits: 3)$],
      ),
    ),
  ),
)
