#import "@preview/lilaq:0.5.0" as lq

#let data = json("../data/P5_1/domain_growth.json")

#set page(flipped: true)

#show lq.selector(lq.diagram): set align(center + horizon)
#show lq.selector(lq.title): set text(size: 14pt)
#show lq.selector(lq.label): set text(size: 12pt)

#align(center)[#text(size: 20pt)[Kronberger\_P5\_2a: Domain growth $L(t) = A t^alpha$ (sfrac = 0.5)]]

// Pre-saturation window. The 128² grid saturates around L ≈ N/6 ≈ 20, after
// which L(t) plateaus and the finite-size "stripe-wrap" spike at late t contaminates
// any power-law fit. Fit only samples with L < L_CAP.
#let L-CAP = 18

// Least-squares fit of ln(L) = ln(A) + alpha * ln(t) on a caller-provided window.
// Note: typst `calc.log` is base-10; use `calc.ln` to stay consistent with `calc.exp`.
#let powerlaw-fit(ts, ls) = {
  let pairs = ts.zip(ls).filter(((t, l)) => t > 0 and l > 0 and l < L-CAP)
  let logs = pairs.map(((t, l)) => (calc.ln(t), calc.ln(l)))
  let n = logs.len()
  if n < 2 {
    return (alpha: 0.0, A: 1.0, n: n)
  }
  let sum-x = logs.map(((x, _)) => x).sum()
  let sum-y = logs.map(((_, y)) => y).sum()
  let mean-x = sum-x / n
  let mean-y = sum-y / n
  let num = logs.map(((x, y)) => (x - mean-x) * (y - mean-y)).sum()
  let den = logs.map(((x, _)) => calc.pow(x - mean-x, 2)).sum()
  let alpha = num / den
  let log-a = mean-y - alpha * mean-x
  let a = calc.exp(log-a)
  (alpha: alpha, A: a, n: n)
}

#let panel(curve) = {
  let fit = powerlaw-fit(curve.times, curve.l_of_t)
  let pos = curve.times.zip(curve.l_of_t).filter(((t, l)) => t > 0 and l > 0)
  let ts-pos = pos.map(((t, _)) => t)
  let ls-pos = pos.map(((_, l)) => l)
  // Fit curve drawn only over the in-window times, so the overlay doesn't extend
  // into the plateau where it would look misleading.
  let fit-pairs = pos.filter(((_, l)) => l < L-CAP)
  let fit-ts = fit-pairs.map(((t, _)) => t)
  let fit-ys = fit-ts.map(t => fit.A * calc.pow(t, fit.alpha))
  figure(
    box(
      height: 85%,
      lq.diagram(
        title: [$tau = #curve.tau$],
        width: 100%,
        height: 100%,
        xscale: "log",
        yscale: "log",
        xlabel: [$t$],
        ylabel: [$L(t)$],
        legend: (position: bottom + right),
        lq.plot(
          ts-pos,
          ls-pos,
          mark: "o",
          stroke: none,
          label: [simulation],
        ),
        lq.plot(
          fit-ts,
          fit-ys,
          mark: none,
          stroke: (dash: "dashed", thickness: 1.5pt),
          color: red,
          label: [fit: $alpha = #calc.round(fit.alpha, digits: 3)$],
        ),
      ),
    ),
    caption: [$tau = #curve.tau$: $A = #calc.round(fit.A, digits: 3)$, $alpha = #calc.round(fit.alpha, digits: 3)$ on #fit.n samples with $L < #L-CAP$],
  )
}

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  ..data.curves.map(panel),
)

#v(1em)
#align(center)[
  #text(size: 10pt, style: "italic")[
    Fit is restricted to the pre-saturation window $L(t) < L_"cap"$ (#L-CAP cells). On this
    128² grid, $L(t)$ plateaus near $N_x\/6 ≈ 21$ once the single-stripe morphology sets in,
    and the late-time spike where that stripe wraps the periodic box would otherwise
    dominate the regression. Expected asymptotic exponents are $alpha = 2\/3$
    (inertial hydrodynamics, low $tau$) and $alpha = 1\/3$ (viscous / Lifshitz–Slyozov, high $tau$).
  ]
]

#pagebreak()

= Task 2a — interpretation

_(to be filled in on paper)_

- Fitted exponents $alpha$ for $tau = 0.7$ and $tau = 5$; comparison against the expected
  $alpha = 2\/3$ (inertial regime) and $alpha = 1\/3$ (viscous / LS).
- Why does one $tau$ scale faster? Role of Reynolds number, hydrodynamic coupling.
- Why does the short 128² grid prevent us from reaching the asymptotic regime?
- Physical meaning of universality classes: identical exponents across systems that share
  conservation laws and coupled fields even though microscopic $lambda$, $kappa$, $M$ differ.
- Why study scaling at all — prediction across experimental systems, critical-phenomena
  language, collapses to a single master curve.

= Task 2b — pseudocode sketch

_(alternative $L(t)$ estimators; to be presented on the board)_

+ *Real-space cluster size.* Threshold $phi$ at $plus.minus phi_"bin"\/2$, label connected
  components via `ndimage.label`, return $L = sqrt(<"cluster_area">)$.

+ *Auto-correlation first zero.* Compute $C(r) = <phi(x) phi(x + r)>$ radially averaged;
  $L$ = smallest $r$ with $C(r) = 0$.

+ *Interface-length per area.* Count domain-wall cells via $|"grad" phi| > "thr"$, then
  $L = A_"total" / A_"wall"$ (Cahn's formula).

The chosen $L = 2 pi \/ <k>$ is equivalent to the second-moment-of-correlation length
and is robust when the structure factor peaks broadly, which is exactly the LB spinodal case.
