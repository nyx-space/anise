/*
    Nyx, blazing fast astrodynamics
    Copyright (C) 2022 Christopher Rabotin <christopher.rabotin@gmail.com>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

/*
 * SOURCES:
 * 1. bacon-sci, MIT licensed, Copyright (c) Wyatt Campbell.
 * 2. Nyx, AGPL v3 license, copied here with permission of redistribution under Mozilla Public License by Chris Rabotin.
 */

use crate::errors::MathErrorKind;
use crate::math::polyfit::hrmint::hrmint_;
use crate::math::polyfit::{F64TupleArray, MAX_SAMPLES};
use crate::math::utils::normalize;
use crate::{
    math::polyfit::polynomial::{multiply, Polynomial},
    prelude::AniseError,
};
use log::warn;

const Q_LENGTH: usize = MAX_SAMPLES * MAX_SAMPLES;

impl<const DEGREE: usize> Polynomial<DEGREE> {
    pub fn hermite(xs: &[f64], ys: &[f64], derivs: &[f64]) -> Result<Self, AniseError> {
        if xs.is_empty() || ys.is_empty() || derivs.is_empty() {
            return Err(AniseError::MathError(
                MathErrorKind::InvalidInterpolationData("No data to interpolate".to_string()),
            ));
        }

        if xs.len() != ys.len() || xs.len() != derivs.len() {
            return Err(AniseError::MathError(
                MathErrorKind::InvalidInterpolationData(
                    "Abscissa, evaluations, and derivatives must be of the same size".to_string(),
                ),
            ));
        }

        // We need to define the number of samples here because when parsing the data from DAF files, we actually do not know the length.
        // Therefore, we can't specify in the parameters that length (compiler complains that `&[f64]` is different from `&[f64; N]`).

        if DEGREE < 2 * xs.len() - 1 {
            warn!(
                "Building Hermite interpolation of degree {} with {} samples, {} degree recommended",
                DEGREE,
                xs.len(),
                2 * xs.len()- 1
            );
        }

        let mut zs = F64TupleArray::<2, MAX_SAMPLES>::zeros();
        let mut qs = F64TupleArray::<4, Q_LENGTH>::zeros();

        for i in 0..xs.len() {
            zs[2 * i] = xs[i];
            zs[2 * i + 1] = xs[i];
            qs[2 * i] = ys[i];
            qs[2 * i + 1] = ys[i];
            qs[2 * i + 1 + (2 * xs.len())] = derivs[i];

            if i != 0 {
                qs[2 * i + (2 * xs.len())] =
                    (qs[2 * i] - qs[2 * i - 1]) / (zs[2 * i] - zs[2 * i - 1]);
            }
        }

        for i in 2..2 * xs.len() {
            for j in 2..=i {
                qs[i + j * (2 * xs.len())] = (qs[i + (j - 1) * (2 * xs.len())]
                    - qs[i - 1 + (j - 1) * (2 * xs.len())])
                    / (zs[i] - zs[i - j]);
            }
        }

        let mut hermite = Polynomial::<DEGREE>::zeros();
        for i in (1..2 * xs.len()).rev() {
            hermite += qs[i + i * (2 * xs.len())];
            let new_poly = Polynomial::<2>::from_most_significant([1.0, -xs[(i - 1) / 2]]);
            hermite = multiply::<DEGREE, 2, DEGREE>(hermite, new_poly);
        }
        hermite += qs[0];

        if hermite.is_nan() {
            return Err(AniseError::MathError(
                MathErrorKind::InvalidInterpolationData(format!(
                    "Invalid interpolation {:x}",
                    hermite
                )),
            ));
        }

        Ok(hermite)
    }
}

/// Stores a Hermite series
pub struct HermiteSeries<const N: usize> {
    coefficients: [f64; N],
}

impl<const N: usize> HermiteSeries<N> {
    /// Convert a Hermite series to a Polynomial
    pub fn to_polynomial(&self) -> Polynomial<N> {
        let mut rtn = Polynomial {
            coefficients: self.coefficients,
        };
        if N == 1 {
            // Do nothing more
            return rtn;
        } else if N == 2 {
            rtn.coefficients[1] *= 2.0;
        } else {
            let mut c0 = Polynomial::<N>::zeros();
            let mut c1 = Polynomial::<N>::zeros();
            c0.coefficients[0] = self.coefficients[self.coefficients.len() - 2];
            c1.coefficients[0] = self.coefficients[self.coefficients.len() - 1];

            for i in (2..self.coefficients.len()).rev() {
                let tmp = c0;
                let mut c_im2 = Polynomial::<N>::zeros();
                c_im2.coefficients[0] = self.coefficients[i - 2];
                c0 = c_im2 - c1 * (2 * (i - 1)) as f64;
                c1.shift_by_one();
                c1 = tmp + 2.0 * c1;
            }
            c1.shift_by_one();
            rtn = c0 + 2.0 * c1;
        }
        rtn
    }
}

#[test]
fn hermite_sine_test() {
    use super::MAX_DEGREE;
    let xs: Vec<_> = (0..8).map(|i| i as f64).collect();
    let ys: Vec<_> = xs.iter().map(|x| x.cos()).collect();
    let derivs: Vec<_> = xs.iter().map(|x| -x.sin()).collect();

    let tol = 1e-10;
    let poly = Polynomial::<MAX_DEGREE>::hermite(&xs, &ys, &derivs).unwrap();

    println!("{:x}", poly);

    let mut max_eval_err: f64 = 0.0;
    let mut max_deriv_err: f64 = 0.0;

    for x in xs {
        let (eval, deriv) = poly.eval_n_deriv(x);
        let eval_err = (eval - x.cos()).abs();
        assert!(eval_err < tol);
        max_eval_err = max_eval_err.max(eval_err);

        let deriv_err = (deriv - -x.sin()).abs();
        assert!(deriv_err < tol);
        max_deriv_err = max_eval_err.max(eval_err);
    }

    println!(
        "Max eval error: {:.e}\tMax deriv error: {:.e}\t",
        max_eval_err, max_deriv_err
    );
}

#[test]
fn hermite_constant_test() {
    use crate::math::polyfit::LargestPolynomial;
    let xs: Vec<_> = (0..8).map(|i| i as f64).collect();
    let ys: Vec<_> = xs.iter().map(|_| 2.0159).collect();
    let derivs: Vec<_> = xs.iter().map(|_| 0.0).collect();

    let tol = 1e-10;
    let poly = LargestPolynomial::hermite(&xs, &ys, &derivs).unwrap();

    println!("{:x}", poly);

    let mut max_eval_err: f64 = 0.0;
    let mut max_deriv_err: f64 = 0.0;

    for x in xs {
        let (eval, deriv) = poly.eval_n_deriv(x);
        let eval_err = (eval - 2.0159).abs();
        assert!(eval_err < tol);
        max_eval_err = max_eval_err.max(eval_err);

        let deriv_err = (deriv).abs();
        assert!(deriv_err < tol);
        max_deriv_err = max_eval_err.max(eval_err);
    }

    println!(
        "Max eval error: {:.e}\tMax deriv error: {:.e}\t",
        max_eval_err, max_deriv_err
    );
}

#[test]
fn hermite_ephem_spline_test() {
    use super::MAX_DEGREE;
    let ts = [
        -1.0,
        -0.7142321608948587,
        -0.4284548929983568,
        -0.14272281352821248,
        0.1430009063036013,
        0.4286973024022658,
        0.714367019041751,
        1.0,
    ];
    let values = [
        -1200.6957374089038,
        -1649.3350718512218,
        -2088.1291193578113,
        -2514.3714789070427,
        -2925.5702772667646,
        -3319.240151300038,
        -3693.030156393982,
        -4044.695271513933,
    ];
    let values_dt = [
        -5.450221271198159,
        -5.3475633589540585,
        -5.212915678573803,
        -5.0471031201910135,
        -4.851091887968967,
        -4.626059429784994,
        -4.373345524123602,
        -4.094465775216765,
    ];

    let tol = 2e-7;
    let tol_deriv = 3e-6;
    let poly = Polynomial::<MAX_DEGREE>::hermite(&ts, &values, &values_dt).unwrap();

    println!("{:x}", poly);

    let mut max_eval_err: f64 = 0.0;
    let mut max_deriv_err: f64 = 0.0;

    for (i, t) in ts.iter().enumerate() {
        let (eval, deriv) = poly.eval_n_deriv(*t);
        let eval_err = (eval - values[i]).abs();
        assert!(dbg!(eval_err) < tol);
        max_eval_err = max_eval_err.max(eval_err);

        let deriv_err = (deriv - values_dt[i]).abs();
        assert!(dbg!(deriv_err) < tol_deriv);
        max_deriv_err = max_deriv_err.max(deriv_err);
    }

    println!(
        "Max eval error: {:.e}\tMax deriv error: {:.e}\t",
        max_eval_err, max_deriv_err
    );
}

#[test]
fn hermite_duplication_test() {
    use super::MAX_DEGREE;
    let ts = [-1.0, 0.0, 1.0];
    let values = [239213.98224426163, 239342.1452415863, 239492.31122918683];
    let values_dt = [5.856883346456119, 1259.7108315572618, 737.5474327513627];

    let tol = 2e-16;
    let tol_deriv = 1e-11;
    let poly = Polynomial::<MAX_DEGREE>::hermite(&ts, &values, &values_dt).unwrap();

    println!("{:x}", poly);

    let mut max_eval_err: f64 = 0.0;
    let mut max_deriv_err: f64 = 0.0;

    for (i, t) in ts.iter().enumerate() {
        let (eval, deriv) = poly.eval_n_deriv(*t);
        let eval_err = (eval - values[i]).abs();
        assert!(dbg!(eval_err) < tol);
        max_eval_err = max_eval_err.max(eval_err);

        let deriv_err = (deriv - values_dt[i]).abs();
        assert!(dbg!(deriv_err) < tol_deriv);
        max_deriv_err = max_deriv_err.max(deriv_err);
    }

    println!(
        "Max eval error: {:.e}\tMax deriv error: {:.e}\t",
        max_eval_err, max_deriv_err
    );
}

#[test]
fn herm2poly() {
    let series = HermiteSeries {
        coefficients: [
            -364.319505276875,
            -230.472812950625,
            -817.857413263125,
            -134.8289486859375,
            -229.266493323125,
            -15.82103409828125,
            -17.08533955890625,
            -0.443532253984375,
            -0.3394307234765625,
        ],
    };
    let expected = Polynomial {
        coefficients: [
            0.1945330000000354,
            3.61185323000015,
            -6.133532429999718,
            -37.53450715000004,
            -29.24982842000058,
            89.83425820999997,
            123.0579811700001,
            -56.77212851,
            -86.89426521,
        ],
    };
    let poly = series.to_polynomial();
    println!("{}", poly);
    println!("{}", expected);
    let delta = poly - expected;
    println!("DELTA = {}", delta);
    for c in delta.coefficients {
        assert!(c.abs() < 1e-10);
    }
}

#[test]
fn hermite_spice_docs_example() {
    use super::MAX_DEGREE;
    let ts = [-1.0, 0.0, 3.0, 5.0];
    let values = [6.0, 5.0, 2210.0, 78180.0];
    let values_dt = [3.0, 0.0, 5115.0, 109395.0];

    let tol = 2e-7;
    let tol_deriv = 3e-6;
    let poly = Polynomial::<MAX_DEGREE>::hermite(&ts, &values, &values_dt).unwrap();

    println!("{:x}", poly);

    let mut max_eval_err: f64 = 0.0;
    let mut max_deriv_err: f64 = 0.0;

    for (i, t) in ts.iter().enumerate() {
        let (eval, deriv) = poly.eval_n_deriv(*t);
        let eval_err = (eval - values[i]).abs();
        assert!(dbg!(eval_err) < tol);
        max_eval_err = max_eval_err.max(eval_err);

        let deriv_err = (deriv - values_dt[i]).abs();
        assert!(dbg!(deriv_err) < tol_deriv);
        max_deriv_err = max_deriv_err.max(deriv_err);
    }

    println!(
        "Max eval error: {:.e}\tMax deriv error: {:.e}\t",
        max_eval_err, max_deriv_err
    );

    println!("{:?}", poly.eval_n_deriv(2.0));
}

#[test]
fn hermite_spice_data() {
    use super::MAX_DEGREE;
    use crate::file_mmap;
    use crate::naif::daf::NAIFDataSet;
    use crate::naif::spk::datatypes::Type2ChebyshevSet;
    use crate::prelude::*;
    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de438s.bsp";
    let buf = file_mmap!(path).unwrap();
    let spk = SPK::parse(&buf).unwrap();
    let ctx = Context::from_spk(&spk).unwrap();

    let data = ctx.spk_data[0]
        .unwrap()
        .nth_data::<Type2ChebyshevSet>(0)
        .unwrap();

    // Now, build the X, Y, Z data from the record data.
    let record = data.nth_record(0).unwrap();

    let poly = Polynomial::<MAX_DEGREE>::hermite(record.z_coeffs, record.x_coeffs, record.y_coeffs)
        .unwrap();
    println!("{poly}");
}

#[test]
fn hermite_ephem_spline_test2() {
    use super::MAX_DEGREE;
    let epoch_et = 773064069.1841084;
    let epochs = [
        773063753.0320327,
        773063842.6860328,
        773063932.1790327,
        773064021.5950327,
        773064111.0160326,
        773064200.4970326,
        773064290.0490326,
        773064379.5660326,
        773064467.8020325,
    ];
    let mut ts = [0.0; 9];
    let values = [
        1264.0276092333008,
        1169.380111723055,
        1067.501355281949,
        958.9770086109238,
        844.4072328473662,
        724.4430188794065,
        599.8186349004518,
        471.46623936222625,
        342.04349989730264,
    ];
    let values_dt = [
        -1.0119972729331588,
        -1.0982621220038147,
        -1.1773202325269372,
        -1.248793644639029,
        -1.3123304769876323,
        -1.3675873394086253,
        -1.414230273831576,
        -1.4519274117465721,
        -1.4801351852184736,
    ];

    // Normalize the epochs between -1.0 and 1.0
    let samples = 7;
    let central_idx = 4;
    let start_idx = central_idx - (samples - 1) / 2 - 1;
    let end_idx = central_idx + (samples - 1) / 2 + 1;

    let first_sample_epoch_et_s = epochs[start_idx];
    let last_sample_epoch_et_s = epochs[end_idx];

    let min_x = dbg!(first_sample_epoch_et_s);
    let max_x = dbg!(last_sample_epoch_et_s);

    for idx in 0..epochs.len() {
        ts[idx] = normalize(epochs[idx], min_x, max_x);
    }

    dbg!(&ts);

    let tol = 1e-9;
    let tol_deriv = 1e-9;
    let poly = Polynomial::<MAX_DEGREE>::hermite(
        dbg!(&epochs[start_idx..end_idx + 1]),
        dbg!(&values[start_idx..end_idx + 1]),
        dbg!(&values_dt[start_idx..end_idx + 1]),
    )
    .unwrap();

    println!("{:x}", poly);

    let mut max_eval_err: f64 = 0.0;
    let mut max_deriv_err: f64 = 0.0;

    for i in start_idx..end_idx + 1 {
        let t = epochs[i];
        let (eval, deriv) = poly.eval_n_deriv(t);
        let eval_err = (eval - values[i]).abs();
        // assert!(dbg!(eval_err) < tol);
        max_eval_err = max_eval_err.max(eval_err);

        let deriv_err = (deriv - values_dt[i]).abs();
        // assert!(dbg!(deriv_err) < tol_deriv);
        max_deriv_err = max_deriv_err.max(deriv_err);
    }

    println!(
        "Max eval error: {:.e}\tMax deriv error: {:.e}\t",
        max_eval_err, max_deriv_err
    );

    let (x, vx) = poly.eval_n_deriv(epoch_et);

    let want_x = 8.9871033515359500e+02;
    let want_vx = -1.2836208430532707e+00;

    dbg!(x, vx);

    println!("{}\t{}", x - want_x, vx - want_vx);
}
