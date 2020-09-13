# Tinate
A text editor in progress. I'm developing it as a way to learn Rust more thoroughly.

It is intented to be a vim-like editor. In the current state you can already read and edit text files.

## Support
Tinate has only been tested in Linux(Debian), but the [crossterm](https://github.com/crossterm-rs/crossterm) crate works on Windows and Mac.

Tinate uses the latest stable Rust release.

## Compiling

First you need to clone the repository using git:

`git clone https://github.com/CastilloDel/tinate.git`

Then you can compile using Cargo:

```cargo build --release```

This should build the binary inside target/release which you can then put in your $PATH.
