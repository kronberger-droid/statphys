
#import "@preview/lilaq:0.5.0" as lq
#import "lib.typ": gauss

#align(center)[#text(size: 20pt)[Kronberger_P1: Unbiased random walk]]

At each timestep $theta$ the particle moves $plus.minus Delta$ with probability
$1/2$ each.

Thus after n steps the particle has taken $m$ steps to the right and $(n - m)$
to the left, with the total displacement $s$:
$
  s = (2 m - n)Delta
$

Due to each step being unrelated from another and the equal probability each we
can write the probability of m steps to the right after n steps using the
binomial distribution:
$
  P(m, n) = 1/2^n binom(n, m)
$

Thus, if we can calculate the mean square displacement using:
$
  chevron.l s^2 chevron.r
  = sum_(m=0)^(+ infinity) s^2(m, n) P(m, n)
  = Delta^2 sum_(m=0)^(+ infinity) (2m-n)^2 P(m, n)
  = Delta^2/2^n sum_(m=0)^(+ infinity) (2m-n)^2 binom(n, m)
$
Expanding:

$
  chevron.l s^2 chevron.r
  = Delta^2/2^n sum_(m=0)^(+ infinity) (4m^2 - 4m n + n^2) binom(n, m)
$
Now using the 3 standard identities:
$
  (1) space
  sum_(m=0)^(+ infinity) binom(n, m) = 2^n
  quad quad (2) space
  m sum_(m=0)^(+ infinity) binom(n, m) = n dot 2^(n-1) \
  quad quad (3) space
  m^2 sum_(m=0)^(+ infinity) binom(n, m) = n(n-1) dot 2^(n-2) + n dot 2^(n-1)
$

We get:

$
  chevron.l s^2 chevron.r
  = Delta^2/2^n
  (
    4 dot [(n-1) dot 2^(n-2) + n dot 2^(n-1)]
    - 4n dot n dot 2^(n-1)
    + n^2 dot 2^n
  ) \
  = Delta^2 (n^2 - n + 2n - 2n^2 + n^2) \
$

Leaving us with:

$
  chevron.l s^2 chevron.r = Delta^2 n
  quad arrow quad
  s prop sqrt(n)
$

#pagebreak()

// Plotting:
#align(center)[#text(size: 20pt)[Kronberger_P1_2b]]

#let data = json("../data/P1_2.json")
#let make-hist-plot(hist) = {
  let step = hist.at("n_steps")
  let density = hist.at("density")
  let positions = density.keys().map(k => int(k))
  let values = density.values()

  let x-theory = lq.linspace(-step, step)
  let y-theory = x-theory.map(x => gauss(x, step))

  lq.diagram(
    title: [n = #step],
    width: 100%,
    height: 100%,
    xlabel: $x$,
    ylabel: $p(x)$,
    xlim: (-21, +21),
    ylim: (0, 0.13),
    lq.bar(positions, values, width: 1.0, label: [random walk]),
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

#figure(
  box(
    height: 85%,
    grid(
      rows: (1fr, 1fr),
      gutter: 1em,
      ..data.at("histograms").map(h => make-hist-plot(h))
    ),
  ),
  caption: [Comparison of descrete random walk histograms with Gaussian
    approximation \ Interesting observation is that the number of steps
    and the possible discrete position have the same parity.],
)
