#import "@preview/lilaq:0.5.0" as lq

#set page(flipped: true, margin: (x: 1.2cm, top: 0.8cm, bottom: 1cm))

#let data = json("data/P1_4b.json")
#let analytical = data.at("analytical")
#let method_names = ("reflect", "stop at wall", "don't move", "redraw")
#let method_colors = (blue, orange, green, purple)

#let make-plot(i) = {
  let ana = analytical.at(i)
  let tau = ana.at("dt_over_l2")
  lq.diagram(
    title: [$D t slash L^2 = #tau$],
    xlabel: $x$,
    ylabel: $p(x, t)$,
    xlim: (-0.5, 0.5),
    width: 100%,
    height: 100%,
    lq.plot(
      ana.at("x"),
      ana.at("p"),
      mark: none,
      stroke: 1.5pt + red,
    ),
    ..data
      .at("methods")
      .enumerate()
      .map(((j, method)) => {
        let hist = method.at("curves").at(i)
        lq.plot(
          hist.at("bin_centers"),
          hist.at("density"),
          mark: none,
          stroke: 1pt + method_colors.at(j),
        )
      }),
    legend: none,
  )
}

#align(center)[#text(size: 16pt)[Kronberger P1\_4b: Comparing wall-handling methods]]
#v(0.2em)
#align(center)[
  #box(stroke: 0.5pt + luma(180), inset: 5pt, radius: 3pt)[
    #set text(size: 9pt)
    #box(line(length: 14pt, stroke: 1.5pt + red)) analytical
    #h(1em)
    #for (j, name) in method_names.enumerate() {
      box(line(length: 14pt, stroke: 1pt + method_colors.at(j)))
      [ #name]
      if j < method_names.len() - 1 { h(1em) }
    }
  ]
]
#v(0.3em)

#block(height: 78%, width: 100%, grid(
  columns: (1fr, 1fr),
  rows: (1fr, 1fr),
  gutter: 0.8em,
  ..range(4).map(i => make-plot(i)),
))

#pagebreak()

#grid(
  columns: (1fr, 1fr),
  rows: (1fr, 1fr),
  gutter: 0.8em,
  ..range(4, 7).map(i => make-plot(i)),
  align(center + horizon)[#text(size: 9pt, style: "italic")[
    Comparison of four wall-handling methods. "Reflect" correctly reproduces the analytical solution. "Stop at wall" accumulates density at the boundaries. "Don't move" also leads to boundary accumulation. "Redraw" biases the distribution toward the center.
  ]],
)
