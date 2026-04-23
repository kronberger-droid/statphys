#import "@preview/lilaq:0.5.0" as lq

#let data = json("data/domain_growth.json")

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
      height: 72%,
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

#v(0.5em)
#align(center)[
  #text(size: 9pt, style: "italic")[
    Fit window: $L(t) < L_"cap" = #L-CAP$ cells. On a $128^2$ grid $L(t)$ plateaus near
    $N_x\/6 approx 21$ once one stripe wraps the box, which would contaminate the regression.
  ]
]

#pagebreak()

= Task 2a — interpretation

Both fits come out with $alpha approx 0.03$ — _not_ the asymptotic $1\/3$ or $2\/3$ we'd
expect from theory. Why?

The fit is restricted to $L(t) < L_"cap" = 18$ cells to avoid the finite-size plateau.
On a $128^2$ grid that window is very short: for $tau = 0.7$ the system blows through
$L = 18$ almost immediately (simulation curve runs from $L approx 20$ to $L approx 50$),
so only a handful of early-time samples actually enter the fit. For $tau = 5$ things
stay below the cap longer but sit in the pre-asymptotic / linear-instability regime
where $L$ is still saturating from its initial value rather than tracking a clean
power law. In both cases we are sampling the _crossover_, not the scaling regime —
that's why the slopes end up so small and so similar.

What we _would_ expect with a big enough box. Small $tau$ means small viscosity
($nu prop tau - 1\/2$), big Reynolds number, and inertial hydrodynamic coarsening
with $alpha = 2\/3$ (Furukawa). Large $tau$ means small Re, viscous damping wins, and
the classical Lifshitz–Slyozov diffusion exponent $alpha = 1\/3$ takes over. Both are
universality-class results: the exponent depends only on conservation laws and the
Reynolds regime, not on $lambda$, $kappa$, $M$ or the LB collision model.

*Practical consequence for this assignment:* the 128² grid is too small to resolve the
two universality classes cleanly. To recover them one would need roughly $1024^2$ (to
give $L(t)$ at least a decade of clean power-law window before it hits the finite-box
plateau) and then fit across that window.

*Why scaling matters at all.* A single measurement of $L(t)$ places a system in a
universality class; rescaling $L(t) / t^alpha$ collapses data from very different
systems (polymer blends, alloys, binary fluids) onto one master curve. That is what
makes exponents like $1\/3$ and $2\/3$ so useful in practice.

= Task 2b — why $L = 2 pi \/ ⟨k⟩$ and alternatives

Using the first moment of the structure factor works because $S(k, t)$ peaks at
$k^*(t) tilde 1\/L$, so $⟨k⟩$ tracks the inverse domain size — robust even when
the peak is broad. Three alternatives for the board:

+ *Cluster size.* Threshold $phi$, label connected components, $L = sqrt(⟨"area"⟩)$.
+ *Autocorrelation.* $C(r)$ via FFT, $L =$ first zero of $C(r)$.
+ *Interface length* (Cahn / Porod). Count wall cells ($|nabla phi|$ large),
  $L = "total cells" \/ "wall cells"$.

All three recover the same asymptotic $alpha$; they differ in prefactor and noise.
