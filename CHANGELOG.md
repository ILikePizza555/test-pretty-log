0.6.2
---
- Move most of the code into runtime
- Remove internal init module generated for each test
- Fix test panic because a default global subscriber is already set

0.6.1
---
- Fixed broken dependency on upstream macro

0.6.0
---
- Remove dependency on env_logger.
- Remove all the feature flags as they are pretty much unused.
- Added a runtime library for static macro code that doesn't change.

0.5.1
---
- Update links in cargo.toml

0.5.0
---
- Forked from `test-log`
- Switched to using SemVer
- Tracing subscriber now has ansi and the `pretty` formatter enabled on it by default
  - Usage of ANSI color codes can be controlled with the new `color` attribute argument, or the `RUST_LOG_COLOR` environment variable.
  - Formatter can be changed with the `RUST_LOG_FORMAT` environment variable.

0.2.14
------
- Factored out `test-log-macros` crate to relieve users from having to
  care about tracing/logging dependencies themselves
- Introduced `default_log_filter` attribute for setting the default log
  filter on a per-test basis behind new `unstable` feature
- Improved compile error output on wrong usage
- Bumped minimum supported Rust version to `1.61`
- Bumped `env_logger` dependency to `0.10`


0.2.13
------
- Improved interaction with nested attributes (such as those used by the
  `test_case` crate), that may not have been parsable in the past
- Removed generated `test_impl` function, which could have "leaked" into
  test cases (#28)
- Eliminated dependency on `tracing` crate
- Bumped minimum supported Rust version to `1.56`
- Bumped `syn` dependency to `2.0`


0.2.12
------
- Fixed handling of inner `#[test]` attributes that add arguments to
  test function
- Added GitHub Actions workflow for publishing the crate


0.2.11
------
- Fixed potential build failure due to unhygienic procedural macros
- Switched to using GitHub Actions as CI provider


0.2.10
------
- Fixed potential build failure when used from edition 2021 crates


0.2.9
-----
- Added support for inner `#[test]` attribute arguments
- Added CI checks for auto generated code documentation
- Bumped minimum supported Rust version to `1.49`


0.2.8
-----
- Renamed crate `test-log`


0.2.7
-----
- Changed `tracing` behavior to capture output and only emit it on test
  failure or when explicitly requested
- Bumped minimum `tracing-subscriber` version to `0.2.17`


0.2.6
-----
- Introduced support for `RUST_LOG_SPAN_EVENTS` environment variable
  that can be used to configure emitting of synthetic trace events
- Updated documentation to include wrapping of other attributes
- Bumped minimum supported Rust version to `1.45`


0.2.5
-----
- Eliminated emitting of `-> ()` constructs in test function signatures


0.2.4
-----
- Eliminated need for emitting of `::f` test function
- Excluded unnecessary files from being contained in release bundle


0.2.3
-----
- Initialize `tracing` globally instead of individually for the run time
  of each test
- Bumped minimum supported Rust version to `1.42`


0.2.2
-----
- Added support for initializing `tracing` infrastructure
  - Introduced `log` (enabled by default) and `trace` features (disabled
    by default)
- Dropped `env_logger` dependency


0.2.1
-----
- Relicensed project under terms of `Apache-2.0 OR MIT`


0.2.0
-----
- Added support for providing inner `#[test]` attribute
- Bumped minimum required Rust version to `1.39.0`


0.1.1
-----
- Updated `README.md` with instructions on how to retrieve test output
  and change log level
- Bumped `env_logger` dependency to `0.7`
- Bumped `syn` dependency to `1.0`
- Bumped `quote` dependency to `1.0`
- Bumped `proc-macro` dependency to `1.0`


0.1.0
-----
- Initial release
