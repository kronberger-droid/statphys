#import "@preview/lilaq:0.5.0" as lq
#let data = json("../data/P1_1.json")

#set page(flipped: true)

#let x2_mean = data.at("x2_mean")
#let n = x2_mean.enumerate().map(((i, value)) => i)
#let n_steps = data.at("n_steps")
#let sigma2 = data.at("sigma2")

#show lq.selector(lq.diagram): set align(center + horizon)
#show lq.selector(lq.title): set text(size: 18pt)
#show lq.selector(lq.label): set text(size: 14pt)

#align(center)[#text(size: 20pt)[Kronberger_P1_1a: Gaussian random walk]]

#figure(
  box(
    height: 85%,
    lq.diagram(
      title: $chevron.l x^2 chevron.r "over" n$,
      width: 100%,
      height: 100%,
      cycle: (lq.color.map.petroff6),
      xlabel: [n],
      ylabel: $chevron.l x^2 chevron.r$,
      legend: (position: top + left),
      lq.plot(
        n,
        n.map(v => v * sigma2),
        mark: none,
        stroke: 1.5pt,
        label: $chevron.l x_n^2 chevron.r = n dot sigma^2$,
      ),
      lq.plot(
        n,
        x2_mean,
        stroke: none,
        label: [simulation],
      ),
    ),
  ),
  caption: [Mean square displacement $chevron.l x^2 chevron.r$ as a function of step number $n$ for 10000 Gaussian random walkers ($sigma^2 = #sigma2$) compared with the theoretical prediction $chevron.l x^2 chevron.r = n sigma^2$.],
)
