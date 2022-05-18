# Rust bindings for ISPC Texture Compressor


[![Latest version](https://img.shields.io/crates/v/intel_tex_2.svg)](https://crates.io/crates/intel_tex_2)
[![Documentation](https://docs.rs/intel_tex_2/badge.svg)](https://docs.rs/intel_tex_2)
[![CI](https://github.com/Traverse-Research/intel-tex-rs-2/actions/workflows/build.yaml/badge.svg)](https://github.com/Traverse-Research/intel-tex-rs-2/actions/workflows/build.yaml)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![APACHE2](https://img.shields.io/badge/license-APACHE2-blue.svg)
[![Contributor Covenant](https://img.shields.io/badge/contributor%20covenant-v1.4%20adopted-ff69b4.svg)](../main/CODE_OF_CONDUCT.md)

[![Banner](banner.png)](https://traverseresearch.nl)

## About

This is a forked crate from from Graham Wihlidal's rust bindings repo for the ISPC texture compressor. The fork includes updates to the latest Intel ISPC texture compression, as well as some patches that were required to make it useful in production.

* Graham's repo: https://github.com/gwihlidal/intel-tex-rs
* ISPC texture compressor: https://github.com/GameTechDev/ISPCTextureCompressor

State of the art texture compression for BC6H, BC7, ETC1, ASTC and BC1/BC3.

ISPC and `libclang` are not required, unless regenerating the ISPC kernels:

```console
$cargo build --features=ispc
```

* ISPC compiler:
  * https://ispc.github.io/
* Also need `libclang` installed (for rust-bindgen)
  * https://rust-lang.github.io/rust-bindgen/requirements.html

For convenience, ISPC binaries for macOS, Linux, and Windows are in the repository (but not the crate).

Additionally, libclang exists in the LLVM installer for Windows, also included.
https://github.com/gwihlidal/intel-tex-rs/tree/master/dependencies

## Supported compression formats:

* BC1, BC3 (aka DXT1, DXT5) and BC4, BC5 (aka ATI1N, ATI2N)
* BC6H (FP16 HDR input)
* BC7
* ETC1

## Pending compression formats:

* ASTC (LDR, block sizes up to 8x8)
  * Work in progress

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
intel_tex_2 = "0.2.0"
```

## Example

```console
$ cargo run --release --example main

Width is 4096
Height is 4096
ColorType is RGB(8)
Converting RGB -> RGBA
Block count: 1048576
Compressing to BC7...
  Done!
Saving lambertian.dds file
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

Contributions are always welcome; please look at the [issue tracker](https://github.com/Traverse-Research/intel-tex-rs-2/issues) to see what
known improvements are documented.

## Code of Conduct

Contribution to the intel_tex_2 crate is organized under the terms of the
Contributor Covenant, the maintainers of intel_tex_2, Traverse Research BV, promise to
intervene to uphold that code of conduct.
