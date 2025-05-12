# Contributing to RustPods

Thank you for your interest in contributing to RustPods! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

Please be respectful and considerate of others when contributing to this project. We aim to foster an inclusive and welcoming community.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```sh
   git clone https://github.com/YOUR_USERNAME/RustPods.git
   cd RustPods
   ```
3. Add the original repository as an upstream remote:
   ```sh
   git remote add upstream https://github.com/ORIGINAL_OWNER/RustPods.git
   ```
4. Create a new branch for your changes:
   ```sh
   git checkout -b feature/your-feature-name
   ```

## Development Environment

1. Ensure you have Rust and Cargo installed (https://www.rust-lang.org/tools/install)
2. Install dependencies:
   ```sh
   cargo build
   ```
3. Run the application:
   ```sh
   cargo run
   ```
4. Run tests:
   ```sh
   cargo test
   ```

## Making Changes

1. Make your changes to the codebase
2. Follow the code style and conventions used in the project
3. Add or update tests as necessary
4. Update documentation to reflect your changes

## Commit Guidelines

- Use clear, descriptive commit messages
- Start with a capitalized, imperative verb (e.g., "Add", "Fix", "Update")
- Reference issue numbers if applicable (e.g., "Fix #42: Resolve battery level display issue")
- Keep commits focused on a single change

## Submitting Changes

1. Push your changes to your fork:
   ```sh
   git push origin feature/your-feature-name
   ```
2. Open a pull request against the main repository
3. Provide a clear description of the changes and any relevant issue numbers
4. Be responsive to feedback and be prepared to make additional changes if requested

## Pull Request Process

1. Ensure your code follows the project's style and conventions
2. Update documentation as needed
3. Include screenshots or GIFs for UI changes
4. Add appropriate tests for your changes
5. Your pull request will be reviewed by maintainers
6. Address any feedback or suggestions from the review

## Types of Contributions

We welcome the following types of contributions:

- Bug fixes
- Feature enhancements
- Documentation improvements
- Performance optimizations
- UI/UX improvements
- Test coverage improvements
- Support for additional headphone models

## Project Structure

The project is structured as follows:

```
RustPods/
├── src/               # Source code
│   ├── bluetooth/     # Bluetooth functionality
│   ├── airpods/       # AirPods-specific parsing and logic
│   ├── ui/            # User interface components
│   └── config/        # Application configuration
├── tests/             # Integration tests
├── examples/          # Example code
├── assets/            # Icons and other assets
└── docs/              # Documentation
```

## Code Style

- Follow Rust's official style guide
- Use meaningful variable and function names
- Write comments for complex logic
- Include documentation comments for public APIs
- Run `cargo fmt` before submitting your code
- Ensure your code passes `cargo clippy` with no warnings

## Testing

- Write unit tests for your code
- Ensure all tests pass with `cargo test`
- For UI changes, include tests that verify the functionality
- For Bluetooth functionality, provide mocks where appropriate

## Documentation

- Update documentation to reflect your changes
- Document public APIs with doc comments
- For significant changes, update the relevant guides in the `docs/` directory

## Licensing

By contributing to this project, you agree that your contributions will be licensed under the project's MIT license.

## Questions?

If you have any questions or need help with contributing, please open an issue on GitHub or reach out to the maintainers.

Thank you for contributing to RustPods! 