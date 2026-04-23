#import "@preview/lilaq:0.5.0" as lq
#let d = json("data/P4_2/rdf_rho0.05-T1.0.json")
#set page(flipped: true)

#figure(
  box(
    height: 85%,
    lq.diagram(
      title: [Radial distribution function — $rho = #d.rho$, $T = #d.temperature$],
      width: 100%, height: 100%,
      xlabel: [$r$], ylabel: [$g(r)$],
      lq.plot(d.r, d.g_r, mark: none, stroke: 1.5pt),
      lq.plot(
        (d.r.first(), d.r.last()), (1, 1),
        mark: none, stroke: (dash: "dashed", thickness: 1pt), color: gray,
      ),
    ),
  ),
)
