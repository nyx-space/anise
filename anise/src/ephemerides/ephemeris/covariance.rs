/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::math::Matrix6;
use core::fmt;
use nalgebra::SymmetricEigen;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub enum LocalFrame {
    Inertial,
    RIC,
    VNC,
    RCN,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct Covariance {
    pub matrix: Matrix6,
    pub local_frame: LocalFrame,
}

impl fmt::Display for Covariance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Covariance in {:?}", self.local_frame)?;
        write!(f, "{:.6}", self.matrix)
    }
}

/// Computes the Matrix Logarithm of a Symmetric Positive Definite matrix.
/// Returns None if the matrix is not positive definite (has eigenvalues <= 0).
fn matrix_log_spd(mat: Matrix6) -> Option<Matrix6> {
    // 1. Decompose P = Q * Lambda * Q^T
    //    nalgebra's SymmetricEigen is robust for this.
    let decomp = SymmetricEigen::new(mat);

    // 2. Check strict positive definiteness
    if decomp.eigenvalues.iter().any(|&e| e <= 0.0) {
        return None;
    }

    // 3. Take natural log of eigenvalues: log(Lambda)
    let log_eigenvalues = decomp.eigenvalues.map(|e| e.ln());

    // 4. Reconstruct: log(P) = Q * log(Lambda) * Q^T
    let log_diag = Matrix6::from_diagonal(&log_eigenvalues);

    // Note: Reconstitution is Q * D * Q^T
    // In nalgebra, eigenvectors are stored in columns of decomp.eigenvectors
    Some(decomp.eigenvectors * log_diag * decomp.eigenvectors.transpose())
}

/// Interpolates between P0 and P1 at ratio alpha [0.0, 1.0].
/// The Log-Euclidan interpolation maps the covariances onto the manifold of symmetric matrices.
/// Then, it finds the path that links both matrices in that manifold, ensuring that the matrix remains symmetric.
/// Finally, it returns to the original base ensuring that the matrix is PSD.
pub(crate) fn interpolate_covar_log_euclidean(
    covar0: Matrix6,
    covar1: Matrix6,
    alpha: f64,
) -> Option<Matrix6> {
    let log_p0 = matrix_log_spd(covar0)?;
    let log_p1 = matrix_log_spd(covar1)?;

    // Linear interpolation in the tangent space (Lie Algebra)
    let log_interp = log_p0 * (1.0 - alpha) + log_p1 * alpha;

    // Map back using exp() - nalgebra might not have a native exp() for DMatrix
    // either, so we reuse the spectral method: exp(P) = Q * exp(Lambda) * Q^T
    matrix_exp_symmetric(log_interp)
}

fn matrix_exp_symmetric(mat: Matrix6) -> Option<Matrix6> {
    let decomp = SymmetricEigen::new(mat);
    let exp_eigenvalues = decomp.eigenvalues.map(|e| e.exp());
    let exp_diag = Matrix6::from_diagonal(&exp_eigenvalues);
    Some(decomp.eigenvectors * exp_diag * decomp.eigenvectors.transpose())
}
