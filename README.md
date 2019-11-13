# cef-simple

[![Crates.io](https://img.shields.io/crates/v/tilecoding.svg)](https://crates.io/crates/cef-simple)
[![Docs](https://docs.rs/cef-simple/badge.svg)](https://docs.rs/crate/cef-simple/)

These are simple bindings for [CEF](https://bitbucket.org/chromiumembedded/cef/src/master/) `C API` in [Rust](https://www.rust-lang.org/). These bindings are far from complete and are currently geared towards my own uses. The library follows the "CEFSimple" examples from CEF, and so doesn't work on mac.

It's also not documented yet, but you can follow the [example](examples/simple.rs) to figure out how to use. It's pretty simple.

I haven't sorted out how best to include the CEF distribution, so for now it's just bundled with the repo. This should be changed to something more ergonomic asap.

In order to run the examples, the CEF supporting files must be placed beside the executable. That means, from the CEF directory, copy the contents of the `Release` and `Resources` folders into `target/debug/` or `target/release/` as necessary, and make sure these files are included in any binary distributions. Making this more ergonomic is also on the TODO list.
