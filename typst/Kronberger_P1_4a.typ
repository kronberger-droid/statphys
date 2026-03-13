#import "@preview/lilaq:0.5.0" as lq

#set page(flipped: true, margin: (x: 1.2cm, top: 0.8cm, bottom: 1cm))

#let data = json("../data/P1_4a.json")
#let reflect = data.at("methods").at(0)
#let analytical = data.at("analytical")

#let make-plot(i) = {
  let hist = reflect.at("curves").at(i)
  let ana = analytical.at(i)
  lq.diagram(
    title: [$D t slash L^2 = #hist.at("dt_over_l2")$],
    xlabel: $x$,
    ylabel: $p(x, t)$,
    xlim: (-0.5, 0.5),
    ylim: (0, 10),
    width: 100%,
    height: 100%,
    lq.bar(
      hist.at("bin_centers"),
      hist.at("density"),
      width: hist.at("bin_centers").at(1) - hist.at("bin_centers").at(0),
    ),
    lq.plot(
      ana.at("x"),
      ana.at("p"),
      mark: none,
      stroke: 1.5pt + red,
    ),
    legend: none,
  )
}

#align(center)[#text(size: 16pt)[Kronberger P1\_4a: Numerical diffusion with reflecting walls]]
#v(0.2em)
#align(center)[
  #box(stroke: 0.5pt + luma(180), inset: 5pt, radius: 3pt)[
    #set text(size: 9pt)
    #box(rect(width: 10pt, height: 7pt, fill: blue.lighten(40%))) simulation
    #h(1.2em)
    #box(line(length: 14pt, stroke: 1.5pt + red)) analytical
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
    Numerical simulation of $10^5$ Gaussian random walkers with reflecting walls compared to the analytical solution. The walker is reflected via $x(t + Delta t) = L - [x(t) + Delta x]$ when hitting a wall.
  ]],
)
