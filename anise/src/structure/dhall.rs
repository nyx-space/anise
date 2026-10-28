/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use dhall::Parsed;
use dhall::syntax::{Expr, ExprKind, ImportTarget};

/// Checks that a Dhall string only imports environment variables.
///
/// serde_dhall resolves every import by default, so a Dhall string handed to one of the
/// `from_dhall` parsers could otherwise splice any file readable by the host process into the
/// parsed value (`/etc/hosts as Text`), and a remote import aborts the process because the
/// build of the dhall crate used here does not implement them. Environment variable imports
/// (`env:MY_VAR as Text`) are the supported way of configuring ANISE and are still resolved.
/// Imports between Dhall files remain available when loading a configuration file from disk.
pub(crate) fn check_imports(repr: &str) -> Result<(), String> {
    let parsed = Parsed::parse_str(repr).map_err(|e| e.to_string())?;
    check_expr(&parsed.to_expr())
}

fn check_expr(expr: &Expr) -> Result<(), String> {
    if let ExprKind::Import(import) = expr.kind() {
        match &import.location {
            ImportTarget::Env(_) => {}
            ImportTarget::Local(_, _) => {
                return Err("local file imports are not allowed in a Dhall string".to_string());
            }
            ImportTarget::Remote(_) => {
                return Err("remote imports are not allowed in a Dhall string".to_string());
            }
            ImportTarget::Missing => {
                return Err("missing imports are not allowed in a Dhall string".to_string());
            }
        }
    }
    expr.kind().traverse_ref(check_expr)?;
    Ok(())
}

#[cfg(test)]
mod ut_dhall_imports {
    use super::check_imports;

    #[test]
    fn only_env_imports_are_allowed() {
        assert!(check_imports("{ uri = \"file.bsp\" }").is_ok());
        assert!(check_imports("{ uri = env:CARGO_MANIFEST_DIR as Text }").is_ok());
        // Imports nested in expressions and alternatives are found too.
        assert!(check_imports("{ uri = \"a\" ++ env:CARGO_MANIFEST_DIR as Text }").is_ok());
        assert!(check_imports("{ uri = env:NOPE as Text ? ./Cargo.toml as Text }").is_err());
        assert!(check_imports("{ uri = ./Cargo.toml as Text }").is_err());
        assert!(check_imports("{ uri = /etc/hosts as Text }").is_err());
        assert!(check_imports("{ uri = ~/.bashrc as Text }").is_err());
        assert!(check_imports("{ uri = https://example.com/x as Text }").is_err());
        assert!(check_imports("{ uri = missing as Text }").is_err());
        assert!(check_imports("{ uri = (./other.dhall).uri }").is_err());
        // A string that does not parse is reported rather than accepted.
        assert!(check_imports("{ uri = ").is_err());
    }
}
