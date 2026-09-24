//! Shared helpers for the integration tests: synthetic image generators, a uniform
//! description of every codec, and a reference decoder used to measure quality.

#![allow(dead_code)]

use std::f32::consts::TAU;

use intel_tex_2::{bc1, bc3, bc4, bc5, bc6h, bc7, etc1, RSurface, RgSurface, RgbaSurface};

/// A tightly packed RGBA8 image.
#[derive(Clone)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[u8; 4]>,
}

impl RgbaImage {
    pub fn from_fn(width: u32, height: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Self {
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                pixels.push(f(x, y));
            }
        }
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.pixels[(y * self.width + x) as usize]
    }

    pub fn crop(&self, x0: u32, y0: u32, width: u32, height: u32) -> Self {
        Self::from_fn(width, height, |x, y| self.pixel(x0 + x, y0 + y))
    }
}

/// Small, deterministic xorshift PRNG so tests don't need a `rand` dependency.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn next_u8(&mut self) -> u8 {
        (self.next_u64() >> 56) as u8
    }
}

pub fn solid(width: u32, height: u32, color: [u8; 4]) -> RgbaImage {
    RgbaImage::from_fn(width, height, |_, _| color)
}

/// Smooth gradients on every channel, including alpha.
pub fn gradient(width: u32, height: u32) -> RgbaImage {
    let wx = (width - 1).max(1);
    let hy = (height - 1).max(1);
    RgbaImage::from_fn(width, height, |x, y| {
        [
            (x * 255 / wx) as u8,
            (y * 255 / hy) as u8,
            ((x + y) * 255 / (wx + hy)) as u8,
            (255 - (y * 255 / hy)) as u8,
        ]
    })
}

pub fn noise(width: u32, height: u32, seed: u64) -> RgbaImage {
    let mut rng = Rng::new(seed);
    let pixels = (0..width * height)
        .map(|_| [rng.next_u8(), rng.next_u8(), rng.next_u8(), rng.next_u8()])
        .collect();
    RgbaImage {
        width,
        height,
        pixels,
    }
}

pub fn checker(width: u32, height: u32, cell: u32) -> RgbaImage {
    RgbaImage::from_fn(width, height, |x, y| {
        if ((x / cell) + (y / cell)).is_multiple_of(2) {
            [230, 40, 20, 255]
        } else {
            [20, 60, 220, 0]
        }
    })
}

/// A "natural looking" image: smooth low-frequency color variation with a few hard edges
/// and some fine detail, loosely resembling photographic content.
pub fn natural(width: u32, height: u32) -> RgbaImage {
    let mut rng = Rng::new(0xC0FFEE);
    let grain: Vec<i32> = (0..width * height)
        .map(|_| (rng.next_u8() % 9) as i32 - 4)
        .collect();
    RgbaImage::from_fn(width, height, |x, y| {
        let fx = x as f32 / width as f32;
        let fy = y as f32 / height as f32;
        let wave = |a: f32, b: f32, c: f32| {
            0.5 + 0.25 * (a * fx * TAU + c).sin() + 0.25 * (b * fy * TAU + c * 0.5).cos()
        };
        let mut rgb = [
            wave(1.3, 2.1, 0.0),
            wave(2.7, 0.9, 1.0),
            wave(0.7, 3.3, 2.0),
        ];
        // A hard-edged disc in the middle.
        let (dx, dy) = (fx - 0.5, fy - 0.5);
        if dx * dx + dy * dy < 0.04 {
            rgb = [0.95, 0.85, 0.2];
        }
        let g = grain[(y * width + x) as usize];
        let to_u8 = |v: f32| ((v * 255.0) as i32 + g).clamp(0, 255) as u8;
        let alpha = (wave(0.5, 0.5, 3.0) * 255.0) as u8;
        [to_u8(rgb[0]), to_u8(rgb[1]), to_u8(rgb[2]), alpha]
    })
}

/// Pixel formats accepted by the encoders.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Input {
    /// 1 byte per pixel (red).
    R8,
    /// 2 bytes per pixel (red, green).
    Rg8,
    /// 4 bytes per pixel.
    Rgba8,
    /// 8 bytes per pixel, four little-endian IEEE half floats.
    Rgba16F,
}

impl Input {
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            Input::R8 => 1,
            Input::Rg8 => 2,
            Input::Rgba8 => 4,
            Input::Rgba16F => 8,
        }
    }

    /// Converts an RGBA8 image into a tightly packed buffer of this format.
    pub fn convert(self, image: &RgbaImage) -> Vec<u8> {
        let mut out = Vec::with_capacity(image.pixels.len() * self.bytes_per_pixel());
        for p in &image.pixels {
            match self {
                Input::R8 => out.push(p[0]),
                Input::Rg8 => out.extend_from_slice(&p[..2]),
                Input::Rgba8 => out.extend_from_slice(p),
                Input::Rgba16F => {
                    for c in p {
                        let h = half::f16::from_f32(*c as f32 / 255.0);
                        out.extend_from_slice(&h.to_le_bytes());
                    }
                }
            }
        }
        out
    }
}

/// Uniform description of one encoder + preset, so API-level properties can be tested for
/// every format with the same code.
#[derive(Copy, Clone)]
pub struct Codec {
    pub name: &'static str,
    pub input: Input,
    pub block_bytes: usize,
    pub output_size: fn(u32, u32) -> usize,
    pub compress: fn(&[u8], u32, u32, u32) -> Vec<u8>,
    pub compress_into: fn(&[u8], u32, u32, u32, &mut [u8]),
    /// Decodes to RGBA8 using an independent reference decoder.
    pub decode: fn(&[u8], u32, u32) -> Vec<[u8; 4]>,
    /// Channels of the RGBA8 source that this codec is expected to reproduce.
    pub channels: &'static [usize],
}

impl Codec {
    pub fn compress_image(&self, image: &RgbaImage) -> Vec<u8> {
        let data = self.input.convert(image);
        let stride = image.width * self.input.bytes_per_pixel() as u32;
        (self.compress)(&data, image.width, image.height, stride)
    }

    pub fn roundtrip(&self, image: &RgbaImage) -> Vec<[u8; 4]> {
        let blocks = self.compress_image(image);
        (self.decode)(&blocks, image.width, image.height)
    }

    pub fn psnr(&self, image: &RgbaImage) -> f64 {
        psnr(&image.pixels, &self.roundtrip(image), self.channels)
    }
}

fn r_surface(data: &[u8], width: u32, height: u32, stride: u32) -> RSurface<'_> {
    RSurface {
        data,
        width,
        height,
        stride,
    }
}

fn rg_surface(data: &[u8], width: u32, height: u32, stride: u32) -> RgSurface<'_> {
    RgSurface {
        data,
        width,
        height,
        stride,
    }
}

fn rgba_surface(data: &[u8], width: u32, height: u32, stride: u32) -> RgbaSurface<'_> {
    RgbaSurface {
        data,
        width,
        height,
        stride,
    }
}

type BlockDecoder = fn(&[u8], usize, usize, &mut [u32]) -> Result<(), &'static str>;

fn decode_with(decoder: BlockDecoder, blocks: &[u8], width: u32, height: u32) -> Vec<[u8; 4]> {
    let mut out = vec![0u32; (width * height) as usize];
    decoder(blocks, width as usize, height as usize, &mut out).expect("reference decode failed");
    out.into_iter()
        .map(|p| {
            // texture2ddecoder packs pixels as little-endian BGRA.
            let [b, g, r, a] = p.to_le_bytes();
            [r, g, b, a]
        })
        .collect()
}

/// Decodes unsigned BC6H blocks to linear RGB floats.
///
/// Uses `bcdec_rs` since `texture2ddecoder`'s BC6H decoder overflows in debug builds.
pub fn decode_bc6h_f32(blocks: &[u8], width: u32, height: u32) -> Vec<[f32; 3]> {
    let (width, height) = (width as usize, height as usize);
    let mut out = vec![[0.0f32; 3]; width * height];
    let mut block_rgb = [0.0f32; 4 * 4 * 3];
    for (i, block) in blocks.as_chunks::<16>().0.iter().enumerate() {
        let (bx, by) = (i % (width / 4), i / (width / 4));
        bcdec_rs::bc6h_float(block, &mut block_rgb, 4 * 3, false);
        for y in 0..4 {
            for x in 0..4 {
                let src = (y * 4 + x) * 3;
                out[(by * 4 + y) * width + bx * 4 + x] =
                    [block_rgb[src], block_rgb[src + 1], block_rgb[src + 2]];
            }
        }
    }
    out
}

fn decode_bc6h_unorm8(blocks: &[u8], width: u32, height: u32) -> Vec<[u8; 4]> {
    let to_u8 = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    decode_bc6h_f32(blocks, width, height)
        .into_iter()
        .map(|[r, g, b]| [to_u8(r), to_u8(g), to_u8(b), 255])
        .collect()
}

pub const BC1: Codec = Codec {
    name: "bc1",
    input: Input::Rgba8,
    block_bytes: 8,
    output_size: bc1::calc_output_size,
    compress: |d, w, h, s| bc1::compress_blocks(&rgba_surface(d, w, h, s)),
    compress_into: |d, w, h, s, o| bc1::compress_blocks_into(&rgba_surface(d, w, h, s), o),
    decode: |b, w, h| decode_with(texture2ddecoder::decode_bc1, b, w, h),
    channels: &[0, 1, 2],
};

pub const BC3: Codec = Codec {
    name: "bc3",
    input: Input::Rgba8,
    block_bytes: 16,
    output_size: bc3::calc_output_size,
    compress: |d, w, h, s| bc3::compress_blocks(&rgba_surface(d, w, h, s)),
    compress_into: |d, w, h, s, o| bc3::compress_blocks_into(&rgba_surface(d, w, h, s), o),
    decode: |b, w, h| decode_with(texture2ddecoder::decode_bc3, b, w, h),
    channels: &[0, 1, 2, 3],
};

pub const BC4: Codec = Codec {
    name: "bc4",
    input: Input::R8,
    block_bytes: 8,
    output_size: bc4::calc_output_size,
    compress: |d, w, h, s| bc4::compress_blocks(&r_surface(d, w, h, s)),
    compress_into: |d, w, h, s, o| bc4::compress_blocks_into(&r_surface(d, w, h, s), o),
    decode: |b, w, h| decode_with(texture2ddecoder::decode_bc4, b, w, h),
    channels: &[0],
};

pub const BC5: Codec = Codec {
    name: "bc5",
    input: Input::Rg8,
    block_bytes: 16,
    output_size: bc5::calc_output_size,
    compress: |d, w, h, s| bc5::compress_blocks(&rg_surface(d, w, h, s)),
    compress_into: |d, w, h, s, o| bc5::compress_blocks_into(&rg_surface(d, w, h, s), o),
    decode: |b, w, h| decode_with(texture2ddecoder::decode_bc5, b, w, h),
    channels: &[0, 1],
};

pub const BC6H: Codec = Codec {
    name: "bc6h",
    input: Input::Rgba16F,
    block_bytes: 16,
    output_size: bc6h::calc_output_size,
    compress: |d, w, h, s| {
        bc6h::compress_blocks(&bc6h::basic_settings(), &rgba_surface(d, w, h, s))
    },
    compress_into: |d, w, h, s, o| {
        bc6h::compress_blocks_into(&bc6h::basic_settings(), &rgba_surface(d, w, h, s), o)
    },
    decode: decode_bc6h_unorm8,
    channels: &[0, 1, 2],
};

pub const BC7_OPAQUE: Codec = Codec {
    name: "bc7_opaque",
    input: Input::Rgba8,
    block_bytes: 16,
    output_size: bc7::calc_output_size,
    compress: |d, w, h, s| {
        bc7::compress_blocks(&bc7::opaque_basic_settings(), &rgba_surface(d, w, h, s))
    },
    compress_into: |d, w, h, s, o| {
        bc7::compress_blocks_into(&bc7::opaque_basic_settings(), &rgba_surface(d, w, h, s), o)
    },
    decode: |b, w, h| decode_with(texture2ddecoder::decode_bc7, b, w, h),
    channels: &[0, 1, 2],
};

pub const BC7_ALPHA: Codec = Codec {
    name: "bc7_alpha",
    input: Input::Rgba8,
    block_bytes: 16,
    output_size: bc7::calc_output_size,
    compress: |d, w, h, s| {
        bc7::compress_blocks(&bc7::alpha_basic_settings(), &rgba_surface(d, w, h, s))
    },
    compress_into: |d, w, h, s, o| {
        bc7::compress_blocks_into(&bc7::alpha_basic_settings(), &rgba_surface(d, w, h, s), o)
    },
    decode: |b, w, h| decode_with(texture2ddecoder::decode_bc7, b, w, h),
    channels: &[0, 1, 2, 3],
};

pub const ETC1: Codec = Codec {
    name: "etc1",
    input: Input::Rgba8,
    block_bytes: 8,
    output_size: etc1::calc_output_size,
    compress: |d, w, h, s| etc1::compress_blocks(&etc1::slow_settings(), &rgba_surface(d, w, h, s)),
    compress_into: |d, w, h, s, o| {
        etc1::compress_blocks_into(&etc1::slow_settings(), &rgba_surface(d, w, h, s), o)
    },
    decode: |b, w, h| decode_with(texture2ddecoder::decode_etc1, b, w, h),
    channels: &[0, 1, 2],
};

/// A named encoder preset.
pub type Preset<S> = (&'static str, fn() -> S);

pub const ALL_CODECS: &[Codec] = &[BC1, BC3, BC4, BC5, BC6H, BC7_OPAQUE, BC7_ALPHA, ETC1];

/// Peak signal-to-noise ratio in dB over the given channels. Returns `f64::INFINITY` for a
/// lossless match.
pub fn psnr(a: &[[u8; 4]], b: &[[u8; 4]], channels: &[usize]) -> f64 {
    assert_eq!(a.len(), b.len());
    let mut sum = 0.0f64;
    for (pa, pb) in a.iter().zip(b) {
        for &c in channels {
            let d = pa[c] as f64 - pb[c] as f64;
            sum += d * d;
        }
    }
    let mse = sum / (a.len() * channels.len()) as f64;
    if mse == 0.0 {
        f64::INFINITY
    } else {
        10.0 * (255.0 * 255.0 / mse).log10()
    }
}

/// Largest absolute per-channel difference over the given channels.
pub fn max_abs_error(a: &[[u8; 4]], b: &[[u8; 4]], channels: &[usize]) -> u8 {
    a.iter()
        .zip(b)
        .flat_map(|(pa, pb)| channels.iter().map(move |&c| pa[c].abs_diff(pb[c])))
        .max()
        .unwrap_or(0)
}
