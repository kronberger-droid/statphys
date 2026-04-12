#import "@preview/lilaq:0.5.0" as lq
#let data = json("../data/P4_1/timing.json")

#set page(flipped: true)

#show lq.selector(lq.diagram): set align(center + horizon)
#show lq.selector(lq.title): set text(size: 18pt)
#show lq.selector(lq.label): set text(size: 14pt)

#let mean(arr) = arr.sum() / arr.len()
#let std(arr) = {
  let m = mean(arr)
  let variance = arr.map(x => calc.pow(x - m, 2)).sum() / arr.len()
  calc.sqrt(variance)
}

#let curve(ensemble) = {
  let pts = data.points.filter(p => p.ensemble == ensemble)
  let xs = pts.map(p => p.n_particles)
  let ys = pts.map(p => mean(p.times_s))
  let errs = pts.map(p => std(p.times_s))
  lq.plot(
    xs, ys,
    yerr: errs,
    mark: "o",
    stroke: 1.5pt,
    label: [#ensemble],
  )
}

#align(center)[#text(size: 20pt)[Kronberger\_P4\_1\_1a: MC Performance Scaling]]

#figure(
  box(
    height: 85%,
    lq.diagram(
      title: [Simulation time vs. particle count],
      width: 100%,
      height: 100%,
      xscale: "log",
      yscale: "log",
      xlabel: [$N$ (particles)],
      ylabel: [time (s)],
      legend: (position: top + left),
      cycle: (lq.color.map.petroff6),
      curve("NVT"),
      curve("NPT"),
      // N^2 reference line, anchored to NVT at N=36
      {
        let nvt = data.points.filter(p => p.ensemble == "NVT")
        let xs = nvt.map(p => p.n_particles)
        let t0 = mean(nvt.first().times_s)
        let n0 = xs.first()
        lq.plot(
          xs,
          xs.map(n => t0 * calc.pow(n / n0, 2)),
          mark: none,
          stroke: (dash: "dashed", thickness: 1pt),
          color: gray,
          label: [$tilde N^2$],
        )
      },
    ),
  ),
  caption: [Wall-clock time of MC simulations as a function of particle count $N$ at fixed density $N\/V = 0.5$ with $t_"end" = 200$ sweeps. Error bars show standard deviation over 5 runs.],
)
