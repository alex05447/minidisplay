# minidisplay

A small Rust library that enumerates the system's displays / monitors.

Implemented for Windows only.

NOTE: minimum supported Windows version is Windows 10, version 1607 (because of `SetThreadDpiAwarenessContext()`, used to query display DPI scale).

## Dependencies

[`bitflags`](http://crates.io/crates/bitflags).

On Windows, [`winapi`](http://crates.io/crates/winapi).