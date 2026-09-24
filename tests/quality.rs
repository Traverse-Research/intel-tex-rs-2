//! Round-trip quality tests: compress with the ISPC encoders, decode with an independent
//! reference decoder, and check the reconstruction error against per-format floors.
//!
//! Thresholds sit a couple of dB below the measured values so they catch real regressions
//! (broken bindings, wrong settings plumbing, channel swizzles) without being flaky.

mod common;

use common::*;

/// Minimum PSNR (dB) per codec for one test image, in `ALL_CODECS` order:
/// bc1, bc3, bc4, bc5, bc6h, bc7_opaque, bc7_alpha, etc1.
fn check_psnr(image: &RgbaImage, floors: [f64; 8]) {
    for (codec, floor) in ALL_CODECS.iter().zip(floors) {
        let psnr = codec.psnr(image);
        assert!(
            psnr >= floor,
            "{}: PSNR {psnr:.2} dB below floor {floor} dB",
            codec.name
        );
    }
}

#[test]
fn gradient_psnr() {
    check_psnr(
        &gradient(64, 64),
        [36.0, 37.0, 50.0, 50.0, 40.0, 43.0, 43.0, 36.0],
    );
}

#[test]
fn natural_psnr() {
    check_psnr(
        &natural(64, 64),
        [28.0, 29.0, 39.0, 39.0, 31.0, 36.0, 35.0, 27.0],
    );
}

#[test]
fn natural_large_psnr() {
    check_psnr(
        &natural(256, 128),
        [28.0, 29.0, 39.0, 39.0, 31.0, 36.0, 35.0, 27.0],
    );
}

#[test]
fn two_color_blocks_psnr() {
    // Each 4x4 block holds two colors in a 2x2 checker. Endpoint formats handle this well;
    // ETC1 has a single base chroma per 2x4 sub-block, so it only has to beat garbage.
    check_psnr(
        &checker(64, 64, 2),
        [
            38.0,
            39.0,
            f64::INFINITY,
            f64::INFINITY,
            50.0,
            f64::INFINITY,
            52.0,
            9.0,
        ],
    );
}

#[test]
fn noise_psnr() {
    // Uncorrelated noise is the worst case; this only guards against garbage output.
    check_psnr(
        &noise(64, 64, 1234),
        [12.0, 13.0, 27.0, 27.0, 13.0, 16.0, 14.0, 12.0],
    );
}

#[test]
fn ramp_psnr() {
    // Slow luminance ramp: every 4x4 block spans only a handful of values.
    let ramp = RgbaImage::from_fn(64, 16, |x, y| {
        let v = ((y * 64 + x) / 4) as u8;
        [v, v, v, v]
    });
    check_psnr(
        &ramp,
        [
            40.0,
            40.0,
            40.0,
            40.0,
            48.0,
            f64::INFINITY,
            f64::INFINITY,
            40.0,
        ],
    );
}

#[test]
fn solid_colors_are_nearly_lossless() {
    // Worst-case per-channel error for a flat block, per codec (endpoint quantization only).
    let max_errors = [4u8, 4, 0, 0, 0, 0, 1, 3];
    let mut rng = Rng::new(3);
    for _ in 0..100 {
        let color = [rng.next_u8(), rng.next_u8(), rng.next_u8(), rng.next_u8()];
        let image = solid(8, 8, color);
        for (codec, max) in ALL_CODECS.iter().zip(max_errors) {
            let decoded = codec.roundtrip(&image);
            let err = max_abs_error(&image.pixels, &decoded, codec.channels);
            assert!(
                err <= max,
                "{}: solid {color:?} error {err} > {max}",
                codec.name
            );
        }
    }
}

#[test]
fn extreme_solid_colors() {
    for color in [
        [0, 0, 0, 0],
        [255, 255, 255, 255],
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [0, 0, 0, 255],
        [255, 255, 255, 0],
    ] {
        let image = solid(4, 4, color);
        for codec in ALL_CODECS {
            // ETC1 can't hit fully saturated primaries exactly via its modifier tables.
            let max = if codec.name == "etc1" { 3 } else { 1 };
            let decoded = codec.roundtrip(&image);
            let err = max_abs_error(&image.pixels, &decoded, codec.channels);
            assert!(err <= max, "{}: solid {color:?} error {err}", codec.name);
        }
    }
}

#[test]
fn channels_are_not_swizzled() {
    // Each quadrant is dominated by a different channel; a swizzle would wreck the PSNR.
    let image = RgbaImage::from_fn(32, 32, |x, y| match (x / 16, y / 16) {
        (0, 0) => [240, 10, 10, 250],
        (1, 0) => [10, 240, 10, 180],
        (0, 1) => [10, 10, 240, 90],
        _ => [120, 60, 200, 20],
    });
    for codec in ALL_CODECS {
        let decoded = codec.roundtrip(&image);
        let err = max_abs_error(&image.pixels, &decoded, codec.channels);
        assert!(err <= 4, "{}: max error {err}", codec.name);
    }
}

#[test]
fn quality_scales_with_image_content_not_size() {
    // Encoding is per block, so a tiled image must have exactly the tile's error.
    let tile = natural(16, 16);
    let tiled = RgbaImage::from_fn(64, 64, |x, y| tile.pixel(x % 16, y % 16));
    for codec in ALL_CODECS {
        let a = codec.psnr(&tile);
        let b = codec.psnr(&tiled);
        assert!((a - b).abs() < 1e-9, "{}: {a} vs {b}", codec.name);
    }
}
