# Tech Stack

## Core
- **Rust:** The primary language for the core ANISE toolkit, ensuring memory safety, thread safety, and high performance.
- **hifitime:** High-fidelity time measurement and manipulation.
- **nalgebra:** High-performance linear algebra for celestial mechanics and orientations.

## Bindings & Interfaces
- **Python (pyo3/maturin):** Idiomatic Python bindings for data scientists and mission designers.
- **C++ (cxx):** Safe and efficient C++ bindings for legacy system integration.
- **GUI (eframe/egui):** Modern, cross-platform graphical user interface for visualizing mission data.

## Infrastructure
- **Monorepo:** A single repository managing the core library, CLI, GUI, and all bindings for consistent versioning and development.
- **GitHub Actions:** Continuous integration and deployment for multiple platforms (Rust, Python, C++, GUI).
