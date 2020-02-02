# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
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

[Unreleased]: https://github.com/yupferris/kaze/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/yupferris/kaze/compare/v0.1.0..v0.1.1
[0.1.0]: https://github.com/yupferris/kaze/releases/tag/v0.1.0
