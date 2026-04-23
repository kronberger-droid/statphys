#import "@preview/lilaq:0.5.0" as lq
#import "lib.typ": gauss

#align(center)[#text(
  size: 20pt,
)[Kronberger P1\_3: Diffusion with reflecting walls]]

We solve the diffusion equation
$ (partial p) / (partial t) = D (partial^2 p) / (partial x^2) $
on the interval $[-L\/2, +L\/2]$ with reflecting (Neumann) boundary conditions
$ (partial p) / (partial x) bar.v_(plus.minus L\/2) = 0 $
and initial condition $p(x, 0) = delta(x)$.

*Separation of variables.* We set $p(x, t) = X(x) dot T(t)$ and obtain
$ (T') / (D T) = (X'') / X = -k^2. $
This yields two ODEs:
$ T(t) = e^(-D k^2 t), quad X(x) = A cos(k x) + B sin(k x). $

*Boundary conditions.*

From $X'(plus.minus L\/2) = 0$ we get
$ -A k sin(k L\/2) + B k cos(k L\/2) = 0 quad "(at" +L\/2")" $
$ +A k sin(k L\/2) + B k cos(k L\/2) = 0 quad "(at" -L\/2")". $

Adding:
$2 B k cos(k L\/2) = 0 arrow.double B = 0$

Subtracting:
$-2 A k sin(k L\/2) = 0 arrow.double sin(k L\/2) = 0 arrow.double k_n = (2 pi n) / L, quad n = 1, 2, dots$

The case $k = 0$ gives the constant eigenfunction $X_0 = "const"$. Sine solutions are eliminated.

*General solution.*
$
  p(x, t) = a_0 / 2 + sum_(n=1)^infinity a_n cos((2 pi n x) / L) exp[-D ((2 pi n) / L)^2 t]
$

*Initial condition.* Using $p(x, 0) = delta(x)$:
$
  a_n = 2 / L integral_(-L\/2)^(L\/2) delta(x) cos((2 pi n x) / L) dif x = 2 / L cos(0) = 2 / L
$
and $a_0 \/ 2 = 1 \/ L$ (normalization).

$
  p(x, t) = 1 / L + 2 / L sum_(n=1)^infinity cos((2 pi n x) / L) exp[-D ((2 pi n) / L)^2 t]
$

#pagebreak()

#align(center)[#text(
  size: 20pt,
)[Kronberger P1\_3: Diffusion with reflecting walls]]

#let data = json("data/P1_3.json")

#figure(
  lq.diagram(
    xlabel: $x$,
    ylabel: $p(x, t)$,
    width: 100%,
    height: 40%,
    ..data
      .at("curves")
      .map(curve => {
        lq.plot(
          curve.at("x"),
          curve.at("p"),
          mark: none,
          stroke: 1.5pt,
          label: $D t slash L^2 = #curve.at("dt_over_l2")$,
        )
      }),
    legend: (position: top + right),
  ),
  caption: [Analytical solution $p(x,t)$ for a diffusing particle confined to $[-L\/2, +L\/2]$ with reflecting walls, shown for various reduced times $D t \/ L^2$.],
)

