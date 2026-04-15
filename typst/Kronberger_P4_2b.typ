#import "@preview/lilaq:0.5.0" as lq

#let states = (
  "rho0.05-T0.5",
  "rho0.05-T1.0",
  "rho0.3-T0.5",
  "rho0.3-T1.0",
)
#let data = states.map(s => json("../data/P4_2/rdf_" + s + ".json"))

#set page(flipped: true)

#show lq.selector(lq.diagram): set align(center + horizon)
#show lq.selector(lq.title): set text(size: 14pt)
#show lq.selector(lq.label): set text(size: 11pt)

#align(center)[#text(size: 20pt)[Kronberger\_P4\_2b: Radial Distribution Function]]

#let rdf-plot(d) = {
  box(
    height: 100%,
    lq.diagram(
      title: [$rho = #d.rho, T = #d.temperature$],
      width: 100%,
      height: 100%,
      xlabel: [$r$],
      ylabel: [$g(r)$],
      lq.plot(
        d.r,
        d.g_r,
        mark: none,
        stroke: 1.5pt,
      ),
      // ideal gas reference g(r) = 1
      lq.plot(
        (d.r.first(), d.r.last()),
        (1, 1),
        mark: none,
        stroke: (dash: "dashed", thickness: 1pt),
        color: gray,
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
    rdf-plot(d),
    caption: [$rho = #d.rho$, $T = #d.temperature$],
  ))
)
