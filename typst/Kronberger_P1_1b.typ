#import "@preview/lilaq:0.5.0" as lq

#let data = json("../data/P1_1.json")
#let sigma2 = data.at("sigma2")

#set page(flipped: true)

#show lq.selector(lq.title): set text(size: 14pt)
#show lq.selector(lq.label): set text(size: 11pt)

#align(center)[#text(size: 20pt)[Kronberger_P1_1b]]

// Gaussian PDF: p(x, n) = 1/sqrt(2*pi*n*sigma2) * exp(-x^2/(2*n*sigma2))
#let gauss(x, n) = {
  let v = n * sigma2
  calc.exp(-x * x / (2 * v)) / calc.sqrt(2 * calc.pi * v)
}

#let make-hist-plot(hist) = {
  let step = hist.at("step")
  let centers = hist.at("bin_centers")
  let density = hist.at("density")
  let bin-with = centers.at(1) - centers.at(0)

  // Gaussian theory curve sampled over the range
  let x-min = centers.first()
  let x-max = centers.last()
  let x-theory = range(0, 201).map(i => x-min + (x-max - x-min) * i / 200)
  let y-theory = x-theory.map(x => gauss(x, step))

  lq.diagram(
    title: [n = #step],
    width: 100%,
    height: 100%,
    xlabel: $x$,
    ylabel: $p(x)$,
    lq.bar(centers, density, width: bin-with, label: [simulation]),
    lq.plot(
      x-theory,
      y-theory,
      mark: none,
      stroke: 1.5pt + red,
      label: [theory],
    ),
    legend: (position: top + right),
  )
}

#box(height: 85%)[
  #grid(
    columns: (1fr, 1fr),
    rows: (1fr, 1fr),
    gutter: 1em,
    ..data.at("histograms").map(h => make-hist-plot(h))
  )
]

#v(1fr)
#align(center)[
  Theoretical solution: $p(x, n) = 1 / sqrt(2 pi dot n sigma^2) dot exp(-x^2 / (2 dot n sigma^2))$
]
