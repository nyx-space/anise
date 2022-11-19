/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::Polynomial;
use crate::errors::MathErrorKind;
use core::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CommonPolynomial {
    Constant(f64),
    /// Linear(a, b) <=> f(x) = ax + b (order is FLIPPED from Polynomial<N> structure)
    Linear(f64, f64),
    /// Quadratic(a, b, c) <=> f(x) = ax^2 + bx + c (order is FLIPPED from Polynomial<N> structure)
    Quadratic(f64, f64, f64),
}

impl CommonPolynomial {
    pub fn eval(&self, x: f64) -> f64 {
        match *self {
            Self::Constant(a) => Polynomial::<1> { coefficients: [a] }.eval(x),
            Self::Linear(a, b) => Polynomial::<2> {
                coefficients: [b, a],
            }
            .eval(x),
            Self::Quadratic(a, b, c) => Polynomial::<3> {
                coefficients: [c, b, a],
            }
            .eval(x),
        }
    }

    pub fn deriv(&self, x: f64) -> f64 {
        match *self {
            Self::Constant(a) => Polynomial::<1> { coefficients: [a] }.deriv(x),
            Self::Linear(a, b) => Polynomial::<2> {
                coefficients: [b, a],
            }
            .deriv(x),
            Self::Quadratic(a, b, c) => Polynomial::<3> {
                coefficients: [c, b, a],
            }
            .deriv(x),
        }
    }

    pub fn coeff_in_order(&self, order: usize) -> Result<f64, MathErrorKind> {
        match *self {
            Self::Constant(a) => {
                if order == 0 {
                    Ok(a)
                } else {
                    Err(MathErrorKind::PolynomialOrderError(order))
                }
            }
            Self::Linear(a, b) => match order {
                0 => Ok(b),
                1 => Ok(a),
                _ => Err(MathErrorKind::PolynomialOrderError(order)),
            },
            Self::Quadratic(a, b, c) => match order {
                0 => Ok(c),
                1 => Ok(b),
                2 => Ok(a),
                _ => Err(MathErrorKind::PolynomialOrderError(order)),
            },
        }
    }

    pub fn with_val_in_order(self, new_val: f64, order: usize) -> Result<Self, MathErrorKind> {
        match self {
            Self::Constant(_) => {
                if order != 0 {
                    Err(MathErrorKind::PolynomialOrderError(order))
                } else {
                    Ok(Self::Constant(new_val))
                }
            }
            Self::Linear(x, y) => match order {
                0 => Ok(Self::Linear(new_val, y)),
                1 => Ok(Self::Linear(x, new_val)),
                _ => Err(MathErrorKind::PolynomialOrderError(order)),
            },
            Self::Quadratic(x, y, z) => match order {
                0 => Ok(Self::Quadratic(new_val, y, z)),
                1 => Ok(Self::Quadratic(x, new_val, z)),
                2 => Ok(Self::Quadratic(x, y, new_val)),
                _ => Err(MathErrorKind::PolynomialOrderError(order)),
            },
        }
    }

    pub fn add_val_in_order(self, new_val: f64, order: usize) -> Result<Self, MathErrorKind> {
        match self {
            Self::Constant(x) => {
                if order != 0 {
                    Err(MathErrorKind::PolynomialOrderError(order))
                } else {
                    Ok(Self::Constant(new_val + x))
                }
            }
            Self::Linear(x, y) => match order {
                0 => Ok(Self::Linear(new_val + x, y)),
                1 => Ok(Self::Linear(x, new_val + y)),
                _ => Err(MathErrorKind::PolynomialOrderError(order)),
            },
            Self::Quadratic(x, y, z) => match order {
                0 => Ok(Self::Quadratic(new_val + x, y, z)),
                1 => Ok(Self::Quadratic(x, new_val + y, z)),
                2 => Ok(Self::Quadratic(x, y, new_val + z)),
                _ => Err(MathErrorKind::PolynomialOrderError(order)),
            },
        }
    }
}

impl fmt::Display for CommonPolynomial {
    /// Prints the polynomial with the least significant coefficients first
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Constant(a) => write!(f, "{}", Polynomial::<1> { coefficients: [a] }),
            Self::Linear(a, b) => write!(
                f,
                "{}",
                Polynomial::<2> {
                    coefficients: [b, a],
                }
            ),
            Self::Quadratic(a, b, c) => write!(
                f,
                "{}",
                Polynomial::<3> {
                    coefficients: [c, b, a],
                }
            ),
        }
    }
}

#[test]
fn poly_constant() {
    let c = CommonPolynomial::Constant(10.0);
    for i in -100..=100 {
        assert!(
            (c.eval(i as f64) - 10.0).abs() < 2e-16,
            "Constant polynomial returned wrong value"
        );
    }
}

#[test]
fn poly_linear() {
    let c = CommonPolynomial::Linear(2.0, 10.0);
    for i in -100..=100 {
        let x = i as f64;
        let expect = 2.0 * x + 10.0;
        assert!(
            (c.eval(x) - expect).abs() < 2e-16,
            "Constant polynomial returned wrong value"
        );
    }
}

#[test]
fn poly_quadratic() {
    let p = Polynomial {
        coefficients: [101.0, -2.0, 3.0],
    };
    let p2 = 2.0 * p;
    let c = CommonPolynomial::Quadratic(3.0, -2.0, 101.0);
    for i in -100..=100 {
        let x = i as f64;
        let expect = 3.0 * x.powi(2) - 2.0 * x + 101.0;
        let expect_deriv = 6.0 * x - 2.0;
        assert!(
            (c.eval(x) - expect).abs() < 2e-16,
            "Polynomial returned wrong value"
        );
        assert!(
            (p.deriv(x) - expect_deriv).abs() < 2e-16,
            "Polynomial derivative returned wrong value"
        );

        assert!(
            (p.eval(x) - expect).abs() < 2e-16,
            "Polynomial returned wrong value"
        );
        assert!(
            (p2.eval(x) - 2.0 * expect).abs() < 2e-16,
            "Polynomial returned wrong value"
        );
    }
}

#[test]
fn poly_print() {
    let p = Polynomial {
        coefficients: [101.0, -2.0, 3.0],
    };
    println!("{}", p);
    assert_eq!(
        format!("{}", p),
        format!("{}", CommonPolynomial::Quadratic(3.0, -2.0, 101.0))
    );
}
