![Test](https://github.com/hundsdor/dirk/actions/workflows/cargo-test.yml/badge.svg?branch=main)

_Dirk_ is a compile-time Dependency Injection tool for Rust, focusing on usability and developer experience.

# Etymology

> _Dependency Injection for Rust at (K)Compile-time_

A [_dirk_](https://en.wikipedia.org/wiki/Dirk) is a long-bladed thrusting weapon used as personal sidearm by Scottish Highlanders during the 18th century.
This tool may be used accordingly as a sidearm for injecting instances into your code.

Furthermore, a dirk resembles a dagger, which pays tribute to the framework [Dagger](https://dagger.dev/) by Google, from which dirk has been inspired.

# Comparison with other DI frameworks for Rust

The following small comparison has been derived from an attempt to implement some of the examples using other DI tools for Rust.
If you are convinced that I absolutely missed or misinterpreted some aspects here, feel free to contact me and I will do my best to put things right.

| Feature                      | dirk                 | teloc | shaku                 | waiter_di |
| ---------------------------- | -------------------- | ----- | --------------------- | --------- |
| injection at compile-time    | yes                  | yes   | yes                   | yes       |
| injection of generic types   | yes                  | no    | only with manual impl | no        |
| global singletons            | yes                  | no    | only with manual impl | no        |
| expressive error messages    | as good as possible* | no    | no                    | no        |
| allows (interior) mutability | yes                  | yes   | no                    | no        |

*procedural macros do not always allow access to all information necessary to yield good error messages
