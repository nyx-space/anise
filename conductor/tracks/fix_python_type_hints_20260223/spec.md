# Specification: Fix Python Type Hints and Mypy Errors

## Goal
Improve the quality and accuracy of Python type hints in the `anise-py` package. Ensure that the generated stubs (.pyi files) are complete and that all `mypy` errors are resolved.

## Scope
- Rebuild the Python library using `maturin develop` in the `anise-py` folder.
- Install `mypy` in the UV-managed `.venv` folder of `anise-py`.
- Update each `.pyi` file in `anise-py/anise` using the `generate_stubs.py` script, module by module.
- Run `mypy` to identify type-related errors.
- Fix all identified `mypy` errors in the Python source and stubs.

## Success Criteria
- `maturin develop` completes successfully in `anise-py`.
- `generate_stubs.py` runs without errors for all modules.
- `mypy` reports zero errors for the `anise-py` package.
