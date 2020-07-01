# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.9] - 2020-07-01
### Added
- Unsigned multiplication op to `Signal` API (`mul`)

## [0.1.8] - 2020-06-28
### Fixed
- Clarified docs for `Mem` read port values when `enable` is not asserted
- Various small doc fixes/regularizations

### Changed
- Added more `if_` sugar variants for tuples with up to 12 elements (previously 8)

## [0.1.7] - 2020-03-27
### Added
- Complete Verilog codegen
- Validation tests for Verilog codegen
- `Context::modules` method to borrow a `Context`'s `Module`s, primarily useful for iterating over them for generating Verilog code

### Changed
- Simultaneous reads/writes to the same location in a `Mem` on a given cycle results in reads returning the value previously at that memory location, **not** the newly-written value

### Fixed
- Wrong publish date for 0.1.6 in changelog

## [0.1.6] - 2020-02-22
### Fixed
- Broken default value for `Mem`s with single-bit elements in generated simulators

## [0.1.5] - 2020-02-15
### Added
- `Mem` construct for creating synchronous memories

### Changed
- Internal sim compiler refactorings to simplify/unify some implementation details

### Fixed
- Missing shift doc tests

## [0.1.4] - 2020-02-09
### Fixed
- Link errors in top-level docs
- Error in `rhs_arithmetic` docs for underflow case

## [0.1.3] - 2020-02-09
### Added
- Subtraction and shift ops to `Signal` API (`sub`, `shl`, `shr`, `shr_arithmetic`)

### Changed
- Small readme edits/link fixes

### Fixed
- Module naming convention in top-level docs

## [0.1.2] - 2020-02-02
### Added
- Implement Eq/PartialEq/Hash for `Signal` (note that these are not documented/tested, which we might want to revisit later)

### Changed
- Switched naming convention for `Module`s from `snake_case` to `CamelCase`
- Redesigned entire (unstable) sugar API
- Small changelog formatting fixes

### Fixed
- Removed the last remaining `unsafe` block in the API impl

## [0.1.1] - 2020-01-30
### Added
- Signed comparison ops to `Signal` API (`lt_signed`, `le_signed`, `gt_signed`, `ge_signed`)
- Error check for `concat` to ensure its input `Signal`s belong to the same `Module`
- This changelog

### Changed
- Small typo/link fixes in API docs
- Small clarifications in top-level docs/examples
- Broken link fixes in README
- Changed tag format to be `vx.y.z` instead of `x.y.z`, and not use annotated tags

## [0.1.0] - 2020-01-25 (Initial release)

[Unreleased]: https://github.com/yupferris/kaze/compare/v0.1.9...HEAD
[0.1.9]: https://github.com/yupferris/kaze/compare/v0.1.8..v0.1.9
[0.1.8]: https://github.com/yupferris/kaze/compare/v0.1.7..v0.1.8
[0.1.7]: https://github.com/yupferris/kaze/compare/v0.1.6..v0.1.7
[0.1.6]: https://github.com/yupferris/kaze/compare/v0.1.5..v0.1.6
[0.1.5]: https://github.com/yupferris/kaze/compare/v0.1.4..v0.1.5
[0.1.4]: https://github.com/yupferris/kaze/compare/v0.1.3..v0.1.4
[0.1.3]: https://github.com/yupferris/kaze/compare/v0.1.2..v0.1.3
[0.1.2]: https://github.com/yupferris/kaze/compare/v0.1.1..v0.1.2
[0.1.1]: https://github.com/yupferris/kaze/compare/v0.1.0..v0.1.1
[0.1.0]: https://github.com/yupferris/kaze/releases/tag/v0.1.0
