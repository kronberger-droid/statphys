#import "@preview/lilaq:0.5.0" as lq
#let py_data = json("../data/P4_1/timing_python.json")
#let rs_data = json("../data/P4_1/timing_rust.json")

#set page(flipped: true)

#show lq.selector(lq.diagram): set align(center + horizon)
#show lq.selector(lq.title): set text(size: 16pt)
#show lq.selector(lq.label): set text(size: 12pt)

#let mean(arr) = arr.sum() / arr.len()
#let std(arr) = {
  let m = mean(arr)
  let variance = arr.map(x => calc.pow(x - m, 2)).sum() / arr.len()
  calc.sqrt(variance)
}

#let curves(data) = {
  (("NVT", "NPT")).map(ensemble => {
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
  })
}

#let ref-line-n2(data) = {
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
}

#let ref-line-n(data) = {
  let nvt = data.points.filter(p => p.ensemble == "NVT")
  let xs = nvt.map(p => p.n_particles)
  let t0 = mean(nvt.first().times_s)
  let n0 = xs.first()
  lq.plot(
    xs,
    xs.map(n => t0 * (n / n0)),
    mark: none,
    stroke: (dash: "dashed", thickness: 1pt),
    color: gray,
    label: [$tilde N$],
  )
}

#align(center)[#text(size: 20pt)[Kronberger\_P4\_1\_1a: MC Performance Scaling]]

#grid(
  columns: (1fr, 1fr),
  column-gutter: 1em,
  figure(
    box(
      height: 85%,
      lq.diagram(
        title: [Python (no cell list)],
        width: 100%,
        height: 100%,
        xscale: "log",
        yscale: "log",
        xlabel: [$N$ (particles)],
        ylabel: [time (s)],
        legend: (position: top + left),
        cycle: (lq.color.map.petroff6),
        ..curves(py_data),
        ref-line-n2(py_data),
      ),
    ),
    caption: [Python O($N^2$) implementation],
  ),
  figure(
    box(
      height: 85%,
      lq.diagram(
        title: [Rust + cell list],
        width: 100%,
        height: 100%,
        xlabel: [$N$ (particles)],
        ylabel: [time (s)],
        legend: (position: top + left),
        cycle: (lq.color.map.petroff6),
        ..curves(rs_data),
        ref-line-n(rs_data),
      ),
    ),
    caption: [Rust O($N$) with cell list optimization],
  ),
)
