// Shared utilities for statistical physics exercises.

#import "@preview/lilaq:0.5.0" as lq

/// Gaussian PDF: p(x) = 1/sqrt(2*pi*variance) * exp(-x^2 / (2*variance))
#let gauss(x, variance) = {
  calc.exp(-x * x / (2 * variance)) / calc.sqrt(2 * calc.pi * variance)
}
