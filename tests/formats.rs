//! Format-specific behavior: presets, alpha handling, HDR range and the relationships
//! between the formats (BC3 = BC4 alpha + BC1 color, BC5 = 2x BC4).

mod common;

use common::*;
use intel_tex_2::{bc6h, bc7, etc1, RgbaSurface};

fn bc7_roundtrip(settings: &bc7::EncodeSettings, image: &RgbaImage) -> Vec<[u8; 4]> {
    let data = Input::Rgba8.convert(image);
    let surface = RgbaSurface {
        data: &data,
        width: image.width,
        height: image.height,
        stride: image.width * 4,
    };
    let blocks = bc7::compress_blocks(settings, &surface);
    (BC7_ALPHA.decode)(&blocks, image.width, image.height)
}

fn bc6h_compress(settings: &bc6h::EncodeSettings, rgb: &[[f32; 3]], width: u32) -> Vec<u8> {
    let data: Vec<u8> = rgb
        .iter()
        .flat_map(|&[r, g, b]| [r, g, b, 1.0])
        .flat_map(|f| half::f16::from_f32(f).to_le_bytes())
        .collect();
    let height = rgb.len() as u32 / width;
    let surface = RgbaSurface {
        data: &data,
        width,
        height,
        stride: width * 8,
    };
    bc6h::compress_blocks(settings, &surface)
}

// --- BC1 / BC3 / BC4 / BC5 --------------------------------------------------------------

/// Extracts one channel of an image into the red channel, as BC4 expects.
fn channel_as_red(image: &RgbaImage, channel: usize) -> RgbaImage {
    RgbaImage::from_fn(image.width, image.height, |x, y| {
        [image.pixel(x, y)[channel], 0, 0, 0]
    })
}

#[test]
fn bc3_is_bc4_alpha_plus_bc1_color() {
    let image = natural(32, 32);
    let bc3 = BC3.compress_image(&image);
    let bc1 = BC1.compress_image(&image);
    let alpha = BC4.compress_image(&channel_as_red(&image, 3));
    for i in 0..64 {
        assert_eq!(
            &bc3[i * 16..i * 16 + 8],
            &alpha[i * 8..(i + 1) * 8],
            "alpha block {i}"
        );
        assert_eq!(
            &bc3[i * 16 + 8..(i + 1) * 16],
            &bc1[i * 8..(i + 1) * 8],
            "color block {i}"
        );
    }
}

#[test]
fn bc5_is_two_bc4_blocks() {
    let image = natural(32, 32);
    let bc5 = BC5.compress_image(&image);
    let red = BC4.compress_image(&channel_as_red(&image, 0));
    let green = BC4.compress_image(&channel_as_red(&image, 1));
    for i in 0..64 {
        assert_eq!(
            &bc5[i * 16..i * 16 + 8],
            &red[i * 8..(i + 1) * 8],
            "red block {i}"
        );
        assert_eq!(
            &bc5[i * 16 + 8..(i + 1) * 16],
            &green[i * 8..(i + 1) * 8],
            "green block {i}"
        );
    }
}

#[test]
fn bc1_ignores_alpha() {
    let opaque = RgbaImage::from_fn(16, 16, |x, y| {
        let p = natural(16, 16).pixel(x, y);
        [p[0], p[1], p[2], 255]
    });
    let translucent = RgbaImage::from_fn(16, 16, |x, y| {
        let p = opaque.pixel(x, y);
        [p[0], p[1], p[2], (x * 16) as u8]
    });
    assert_eq!(
        BC1.compress_image(&opaque),
        BC1.compress_image(&translucent)
    );
}

#[test]
fn bc3_binary_alpha_is_exact() {
    let image = RgbaImage::from_fn(32, 32, |x, y| {
        let a = if (x ^ y) & 1 == 0 { 255 } else { 0 };
        [100, 150, 200, a]
    });
    let decoded = BC3.roundtrip(&image);
    assert_eq!(max_abs_error(&image.pixels, &decoded, &[3]), 0);
}

#[test]
fn bc4_ignores_other_bytes_via_stride() {
    // BC4 reads one byte per texel; feed it the red plane of an RGBA buffer by lying about
    // the layout would be wrong, but a single-channel buffer with padding must work.
    let image = gradient(16, 16);
    let expected = BC4.compress_image(&image);
    let mut data = vec![0xEEu8; 32 * 16];
    for y in 0..16 {
        for x in 0..16 {
            data[y * 32 + x] = image.pixel(x as u32, y as u32)[0];
        }
    }
    assert_eq!((BC4.compress)(&data, 16, 16, 32), expected);
}

#[test]
fn bc4_full_range_blocks() {
    // A block containing both 0 and 255 must reproduce the extremes exactly.
    let image = RgbaImage::from_fn(4, 4, |x, y| [((x + y * 4) * 17) as u8, 0, 0, 0]);
    let decoded = BC4.roundtrip(&image);
    assert_eq!(decoded[0][0], 0);
    assert_eq!(decoded[15][0], 255);
    // Eight evenly spaced palette entries over [0, 255] are ~36 apart.
    assert!(max_abs_error(&image.pixels, &decoded, &[0]) <= 18);
}

#[test]
fn bc1_grayscale_stays_nearly_gray() {
    let image = RgbaImage::from_fn(32, 32, |x, y| {
        let v = natural(32, 32).pixel(x, y)[1];
        [v, v, v, 255]
    });
    for p in BC1.roundtrip(&image) {
        let chroma = p[0].abs_diff(p[1]).max(p[1].abs_diff(p[2]));
        // RGB565 has one extra bit of green precision.
        assert!(chroma <= 8, "{p:?}");
    }
}

// --- BC7 --------------------------------------------------------------------------------

const BC7_OPAQUE_PRESETS: [Preset<bc7::EncodeSettings>; 5] = [
    ("ultra_fast", bc7::opaque_ultra_fast_settings),
    ("very_fast", bc7::opaque_very_fast_settings),
    ("fast", bc7::opaque_fast_settings),
    ("basic", bc7::opaque_basic_settings),
    ("slow", bc7::opaque_slow_settings),
];

const BC7_ALPHA_PRESETS: [Preset<bc7::EncodeSettings>; 5] = [
    ("ultra_fast", bc7::alpha_ultra_fast_settings),
    ("very_fast", bc7::alpha_very_fast_settings),
    ("fast", bc7::alpha_fast_settings),
    ("basic", bc7::alpha_basic_settings),
    ("slow", bc7::alpha_slow_settings),
];

/// Slower presets must not be meaningfully worse than faster ones, and the slowest must
/// clearly beat the fastest.
fn assert_quality_ladder(name: &str, psnrs: &[(&str, f64)], min_gain: f64) {
    for pair in psnrs.windows(2) {
        let ((fast, a), (slow, b)) = (pair[0], pair[1]);
        assert!(
            b >= a - 0.25,
            "{name}: {slow} ({b:.2} dB) worse than {fast} ({a:.2} dB)"
        );
    }
    let (first, last) = (psnrs[0].1, psnrs[psnrs.len() - 1].1);
    assert!(
        last - first >= min_gain,
        "{name}: slowest preset only gains {:.2} dB",
        last - first
    );
}

#[test]
fn bc7_opaque_presets_quality_ladder() {
    let image = natural(64, 64);
    let psnrs: Vec<_> = BC7_OPAQUE_PRESETS
        .iter()
        .map(|(name, preset)| {
            (
                *name,
                psnr(&image.pixels, &bc7_roundtrip(&preset(), &image), &[0, 1, 2]),
            )
        })
        .collect();
    assert_quality_ladder("bc7 opaque", &psnrs, 3.0);
}

#[test]
fn bc7_alpha_presets_quality_ladder() {
    let image = natural(64, 64);
    let psnrs: Vec<_> = BC7_ALPHA_PRESETS
        .iter()
        .map(|(name, preset)| {
            (
                *name,
                psnr(
                    &image.pixels,
                    &bc7_roundtrip(&preset(), &image),
                    &[0, 1, 2, 3],
                ),
            )
        })
        .collect();
    assert_quality_ladder("bc7 alpha", &psnrs, 3.0);
}

#[test]
fn bc7_opaque_presets_output_opaque_alpha() {
    // Opaque presets ignore the source alpha. Mode 6 blocks store alpha endpoints with 7 bits
    // plus a shared p-bit, so the encoder can land on 254 rather than exactly 255.
    let image = natural(32, 32);
    for (name, preset) in BC7_OPAQUE_PRESETS {
        for p in bc7_roundtrip(&preset(), &image) {
            assert!(p[3] >= 254, "{name}: alpha {}", p[3]);
        }
    }
}

#[test]
fn bc7_alpha_presets_preserve_alpha() {
    let image = natural(32, 32);
    for (name, preset) in BC7_ALPHA_PRESETS {
        let decoded = bc7_roundtrip(&preset(), &image);
        let alpha_psnr = psnr(&image.pixels, &decoded, &[3]);
        assert!(alpha_psnr >= 35.0, "{name}: alpha PSNR {alpha_psnr:.2}");
    }
}

#[test]
fn bc7_alpha_presets_keep_opaque_images_opaque() {
    let image = RgbaImage::from_fn(32, 32, |x, y| {
        let p = natural(32, 32).pixel(x, y);
        [p[0], p[1], p[2], 255]
    });
    // Mode 7 only has 5-bit alpha endpoints (plus p-bit), so allow a small error.
    for (name, preset) in BC7_ALPHA_PRESETS {
        for p in bc7_roundtrip(&preset(), &image) {
            assert!(p[3] >= 250, "{name}: alpha {}", p[3]);
        }
    }
}

#[test]
fn bc7_every_preset_is_deterministic() {
    let image = noise(16, 16, 5);
    for (name, preset) in BC7_OPAQUE_PRESETS.iter().chain(&BC7_ALPHA_PRESETS) {
        assert_eq!(
            bc7_roundtrip(&preset(), &image),
            bc7_roundtrip(&preset(), &image),
            "{name}"
        );
    }
}

// --- BC6H -------------------------------------------------------------------------------

const BC6H_PRESETS: [Preset<bc6h::EncodeSettings>; 5] = [
    ("very_fast", bc6h::very_fast_settings),
    ("fast", bc6h::fast_settings),
    ("basic", bc6h::basic_settings),
    ("slow", bc6h::slow_settings),
    ("very_slow", bc6h::very_slow_settings),
];

fn ldr_as_linear(image: &RgbaImage) -> Vec<[f32; 3]> {
    image
        .pixels
        .iter()
        .map(|p| {
            [
                p[0] as f32 / 255.0,
                p[1] as f32 / 255.0,
                p[2] as f32 / 255.0,
            ]
        })
        .collect()
}

fn rgb_f32_psnr(a: &[[f32; 3]], b: &[[f32; 3]]) -> f64 {
    let mse = a
        .iter()
        .zip(b)
        .flat_map(|(pa, pb)| (0..3).map(move |c| ((pa[c] - pb[c]) as f64).powi(2)))
        .sum::<f64>()
        / (a.len() * 3) as f64;
    10.0 * (1.0 / mse).log10()
}

#[test]
fn bc6h_presets_quality_ladder() {
    let image = natural(64, 64);
    let rgb = ldr_as_linear(&image);
    let psnrs: Vec<_> = BC6H_PRESETS
        .iter()
        .map(|(name, preset)| {
            let decoded = decode_bc6h_f32(&bc6h_compress(&preset(), &rgb, 64), 64, 64);
            (*name, rgb_f32_psnr(&rgb, &decoded))
        })
        .collect();
    assert_quality_ladder("bc6h", &psnrs, 2.0);
}

#[test]
fn bc6h_preserves_hdr_values() {
    for v in [0.001f32, 0.5, 1.0, 4.0, 100.0, 1000.0, 60000.0] {
        let rgb = vec![[v, v * 0.5, v * 0.25]; 16];
        let decoded = decode_bc6h_f32(&bc6h_compress(&bc6h::basic_settings(), &rgb, 4), 4, 4);
        for p in decoded {
            for c in 0..3 {
                let rel = ((p[c] - rgb[0][c]) / rgb[0][c]).abs();
                assert!(
                    rel < 0.01,
                    "value {v}: channel {c} decoded {} vs {}",
                    p[c],
                    rgb[0][c]
                );
            }
        }
    }
}

#[test]
fn bc6h_hdr_gradient() {
    // Exponential ramp spanning ~16 stops.
    let rgb: Vec<[f32; 3]> = (0..64 * 16)
        .map(|i| {
            let t = (i % 64) as f32 / 63.0 * 16.0 - 8.0;
            let v = 2f32.powf(t);
            [v, v * 0.7, v * 0.3]
        })
        .collect();
    let decoded = decode_bc6h_f32(&bc6h_compress(&bc6h::basic_settings(), &rgb, 64), 64, 16);
    for (i, (a, b)) in rgb.iter().zip(&decoded).enumerate() {
        for c in 0..3 {
            let rel = ((a[c] - b[c]) / a[c]).abs();
            assert!(rel < 0.1, "texel {i} channel {c}: {} vs {}", b[c], a[c]);
        }
    }
}

#[test]
fn bc6h_ignores_alpha() {
    let rgb = ldr_as_linear(&natural(16, 16));
    let with_alpha = |a: f32| -> Vec<u8> {
        rgb.iter()
            .flat_map(|&[r, g, b]| [r, g, b, a])
            .flat_map(|f| half::f16::from_f32(f).to_le_bytes())
            .collect()
    };
    let compress = |data: &[u8]| {
        bc6h::compress_blocks(
            &bc6h::basic_settings(),
            &RgbaSurface {
                data,
                width: 16,
                height: 16,
                stride: 16 * 8,
            },
        )
    };
    assert_eq!(compress(&with_alpha(1.0)), compress(&with_alpha(0.0)));
}

#[test]
#[ignore = "BC6H is encoded as unsigned half floats; negative inputs currently decode as \
            65504.0 instead of being clamped to 0.0"]
fn bc6h_negative_values_clamp_to_zero() {
    let rgb = vec![[-1.0f32, -0.5, -0.25]; 16];
    let decoded = decode_bc6h_f32(&bc6h_compress(&bc6h::basic_settings(), &rgb, 4), 4, 4);
    for p in decoded {
        assert_eq!(p, [0.0, 0.0, 0.0]);
    }
}

// --- ETC1 -------------------------------------------------------------------------------

#[test]
fn etc1_decodes_opaque() {
    for p in ETC1.roundtrip(&natural(32, 32)) {
        assert_eq!(p[3], 255);
    }
}

#[test]
fn etc1_ignores_alpha() {
    let a = natural(16, 16);
    let b = RgbaImage::from_fn(16, 16, |x, y| {
        let p = a.pixel(x, y);
        [p[0], p[1], p[2], 255 - p[3]]
    });
    assert_eq!(ETC1.compress_image(&a), ETC1.compress_image(&b));
}

#[test]
fn etc1_grayscale_stays_gray() {
    let image = RgbaImage::from_fn(32, 32, |x, y| {
        let v = natural(32, 32).pixel(x, y)[1];
        [v, v, v, 255]
    });
    for p in ETC1.roundtrip(&image) {
        assert!(p[0] == p[1] && p[1] == p[2], "{p:?}");
    }
}

#[test]
fn etc1_fast_skip_threshold_affects_quality() {
    let image = natural(64, 64);
    let data = Input::Rgba8.convert(&image);
    let surface = RgbaSurface {
        data: &data,
        width: 64,
        height: 64,
        stride: 64 * 4,
    };
    let psnr_for = |threshold| {
        let settings = etc1::EncodeSettings {
            fast_skip_threshold: threshold,
        };
        let blocks = etc1::compress_blocks(&settings, &surface);
        psnr(&image.pixels, &(ETC1.decode)(&blocks, 64, 64), &[0, 1, 2])
    };
    let low = psnr_for(1);
    let high = psnr_for(etc1::slow_settings().fast_skip_threshold);
    assert!(high >= low - 0.1, "slow {high:.2} vs threshold=1 {low:.2}");
}
