#import "@preview/lilaq:0.5.0" as lq
#let rs_data = json("../data/P4_1/henderson_rust.json")
#let py_data = json("../data/P4_1/henderson_python.json")

#set page(flipped: true)

#show lq.selector(lq.diagram): set align(center + horizon)
#show lq.selector(lq.title): set text(size: 18pt)
#show lq.selector(lq.label): set text(size: 14pt)

// Henderson theoretical curve: P = Z * rho * kT, with kT=1
// Z = 1 + phi^2 / (8*(1-phi)^2)
// rho = N * phi / (N * V_particle) = phi / V_particle
// V_particle = pi * (sigma/2)^2 = pi/4 for sigma=1
// So: P = (1 + phi^2 / (8*(1-phi)^2)) * phi / (pi/4)
//       = (1 + phi^2 / (8*(1-phi)^2)) * 4*phi / pi
#let henderson-pressure(phi) = {
  let z = (1 + calc.pow(phi, 2) / 8) / calc.pow(1 - phi, 2)
  z * 4 * phi / calc.pi
}

#let phi-max = calc.pi / (2 * calc.sqrt(3))

#let n-theory = 200
#let phi-theory = range(1, n-theory).map(i => i / n-theory * 0.85)
#let p-theory = phi-theory.map(phi => henderson-pressure(phi))

#align(center)[#text(size: 20pt)[Kronberger\_P4\_1\_2b: Henderson Equation of State]]

#figure(
  box(
    height: 85%,
    lq.diagram(
      title: [Pressure vs. packing fraction],
      width: 100%,
      height: 100%,
      xlabel: [$phi$ (packing fraction)],
      ylabel: [$P$],
      legend: (position: top + left),
      cycle: (lq.color.map.petroff6),
      lq.plot(
        phi-theory,
        p-theory,
        mark: none,
        stroke: 1.5pt,
        label: [Henderson],
      ),
      lq.plot(
        rs_data.points.map(p => p.packing_fraction),
        rs_data.points.map(p => p.pressure),
        xerr: rs_data.points.map(p => p.packing_std),
        mark: "o",
        stroke: none,
        label: [MC (Rust)],
      ),
      lq.plot(
        py_data.points.map(p => p.packing_fraction),
        py_data.points.map(p => p.pressure),
        xerr: py_data.points.map(p => p.packing_std),
        mark: "x",
        stroke: none,
        label: [MC (Python)],
      ),
      lq.plot(
        (phi-max, phi-max),
        (0, 120),
        mark: none,
        stroke: (dash: "dashed", thickness: 1pt),
        color: gray,
        label: [$phi_m = pi \/ (2 sqrt(3)) approx #calc.round(phi-max, digits: 3)$],
      ),
    ),
  ),
  caption: [Measured packing fraction from NPT simulations compared to the Henderson equation of state $P = Z rho k_B T$ with $Z = (1 + phi^2\/8) \/ (1-phi)^2$. Error bars show standard deviation of $phi$ over equilibrated sweeps.],
)
