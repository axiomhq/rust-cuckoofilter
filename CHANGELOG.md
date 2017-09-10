# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
### Changed
### Deprecated
### Removed
### Fixed
### Security

## [v0.3.2]
### Added
- Filters now have a memory_usage() function that return how much bytes a given filter occupies in memory.
  Let's show how little memory the filters need for their capacity!
### Fixed
- Use std::collections::hash_map::DefaultHasher as replacement for std::hah::SipHasher as default hasher, as
  SipHasher is deprecated since Rust 1.13.
- The same part of the item hash was used for generating the fingerprint as well as the index positions. This means that
  equal fingerprints always had the same index positions, resulting in increased rebucketing and less items fitting in
  the filter.

[Unreleased]: https://github.com/seiflotfy/rust-cuckoofilter/compare/v0.3.2...HEAD
[v0.3.2]: https://github.com/seiflotfy/rust-cuckoofilter/compare/v0.3.1...v0.3.2
