# Tinate
A toy text editor. I developed it as a way to learn Rust more thoroughly.

It is intented to be a vim-like editor. You can perform simple text edits with it.

## Support
Tinate has only been tested in Linux(Debian), but the [crossterm](https://github.com/crossterm-rs/crossterm) crate works on Windows and Mac.

Tinate uses the latest stable Rust release.

## Compiling

First you need to clone the repository using git:

`git clone https://github.com/CastilloDel/tinate.git`

Then you can compile using Cargo:

```cargo build --release```

This should build the binary inside target/release which you can then put in your $PATH.

## Contributions
Any contributions would be highly appreciated. Just remember they will be licensed under the MIT License. And please remember to use rustfmt if needed.

## License
This project is licensed under the MIT License
