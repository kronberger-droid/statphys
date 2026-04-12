#import "@preview/lilaq:0.5.0" as lq
#let data = json("../data/P4_1/acceptance.json")
#let densities = (0.1, 0.5, 0.8)

#let curves = densities.map(rho => {
  let pts = data.points.filter(p => p.density == rho)
  lq.plot(
    pts.map(p => p.max_displacement),
    pts.map(p => p.acceptance_rate),
    stroke: 1.5pt,
    label: [$rho = #rho$],
  )
})

#figure(box(
  height: 85%,
  lq.diagram(
    xlabel: [max displacement],
    ylabel: [acceptance rate],
    legend: (position: top + right),
    ..curves,
  ),
))
