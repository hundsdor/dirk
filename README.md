_Dirk_ is a compile-time Dependency Injection tool for Rust, focusing on usability and developer experience.

# Etymology

> _Dependency Injection for Rust at (K)Compile-time_

A [_dirk_](https://en.wikipedia.org/wiki/Dirk) is a long-bladed thrusting weapon used as personal sidearm by Scottish Highlanders during the 18th century.
This tool may be used accordingly as a sidearm for injecting instances into your code.

Furthermore, a dirk resembles a dagger, which pays tribute to the framework [Dagger](https://dagger.dev/) by Google, from which dirk has been inspired.

# Comparison with other DI frameworks for Rust

| Feature                                      | dirk | teloc | shaku                 |
| -------------------------------------------- | ---- | ----- | --------------------- |
| global singletons                            | yes  | no    | only with manual impl |
| expressive error messages (where applicable) | yes  | no    | no                    |
| allows (interior) mutability                 | yes  | yes   | no                    |
