//! Criterion benchmarks for every encoder and preset.
//!
//! Throughput is reported in texels, so results are comparable across formats and sizes.
//! Run a subset with e.g. `cargo bench --bench compress -- bc7_alpha`.

use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use intel_tex_2::{bc1, bc3, bc4, bc5, bc6h, bc7, etc1, RSurface, RgSurface, RgbaSurface};

#[path = "../tests/common/mod.rs"]
mod common;

use common::{gradient, natural, noise, solid, Input, Preset, RgbaImage, ALL_CODECS};

/// Owned pixel data for one image in every input layout the encoders accept.
struct Source {
    width: u32,
    height: u32,
    r8: Vec<u8>,
    rg8: Vec<u8>,
    rgba8: Vec<u8>,
    rgba16f: Vec<u8>,
}

impl Source {
    fn new(image: &RgbaImage) -> Self {
        Self {
            width: image.width,
            height: image.height,
            r8: Input::R8.convert(image),
            rg8: Input::Rg8.convert(image),
            rgba8: Input::Rgba8.convert(image),
            rgba16f: Input::Rgba16F.convert(image),
        }
    }

    fn texels(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    fn r(&self) -> RSurface<'_> {
        RSurface {
            data: &self.r8,
            width: self.width,
            height: self.height,
            stride: self.width,
        }
    }

    fn rg(&self) -> RgSurface<'_> {
        RgSurface {
            data: &self.rg8,
            width: self.width,
            height: self.height,
            stride: self.width * 2,
        }
    }

    fn rgba(&self) -> RgbaSurface<'_> {
        RgbaSurface {
            data: &self.rgba8,
            width: self.width,
            height: self.height,
            stride: self.width * 4,
        }
    }

    fn rgba16f(&self) -> RgbaSurface<'_> {
        RgbaSurface {
            data: &self.rgba16f,
            width: self.width,
            height: self.height,
            stride: self.width * 8,
        }
    }
}

const SIZES: [u32; 3] = [64, 256, 1024];

const BC6H_PRESETS: [Preset<bc6h::EncodeSettings>; 5] = [
    ("very_fast", bc6h::very_fast_settings),
    ("fast", bc6h::fast_settings),
    ("basic", bc6h::basic_settings),
    ("slow", bc6h::slow_settings),
    ("very_slow", bc6h::very_slow_settings),
];

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

/// Fixed-rate formats: every size, natural content, reusing the output buffer.
///
/// This group must stay first: it is the only one that runs ETC1, and ETC1 has to run
/// before any BC6H/BC7 call (see [`bench_content`]).
fn bench_fixed_formats(c: &mut Criterion) {
    let sources: Vec<Source> = SIZES.iter().map(|&s| Source::new(&natural(s, s))).collect();

    macro_rules! group {
        ($name:literal, $size_fn:path, |$src:ident, $out:ident| $body:expr) => {{
            let mut group = c.benchmark_group($name);
            for $src in &sources {
                group.throughput(Throughput::Elements($src.texels()));
                let mut $out = vec![0u8; $size_fn($src.width, $src.height)];
                group.bench_function(BenchmarkId::from_parameter($src.width), |b| {
                    b.iter(|| $body)
                });
            }
            group.finish();
        }};
    }

    group!("bc1", bc1::calc_output_size, |src, out| {
        bc1::compress_blocks_into(black_box(&src.rgba()), &mut out)
    });
    group!("bc3", bc3::calc_output_size, |src, out| {
        bc3::compress_blocks_into(black_box(&src.rgba()), &mut out)
    });
    group!("bc4", bc4::calc_output_size, |src, out| {
        bc4::compress_blocks_into(black_box(&src.r()), &mut out)
    });
    group!("bc5", bc5::calc_output_size, |src, out| {
        bc5::compress_blocks_into(black_box(&src.rg()), &mut out)
    });
    group!("etc1", etc1::calc_output_size, |src, out| {
        etc1::compress_blocks_into(&etc1::slow_settings(), black_box(&src.rgba()), &mut out)
    });
}

/// Every BC6H / BC7 preset at 256x256. The slow presets are expensive, so use few samples.
fn bench_presets(c: &mut Criterion) {
    let src = Source::new(&natural(256, 256));

    let mut group = c.benchmark_group("bc6h");
    group.throughput(Throughput::Elements(src.texels()));
    group.sample_size(10);
    let mut out = vec![0u8; bc6h::calc_output_size(src.width, src.height)];
    for (name, preset) in BC6H_PRESETS {
        let settings = preset();
        group.bench_function(name, |b| {
            b.iter(|| bc6h::compress_blocks_into(&settings, black_box(&src.rgba16f()), &mut out))
        });
    }
    group.finish();

    for (group_name, presets) in [
        ("bc7_opaque", BC7_OPAQUE_PRESETS),
        ("bc7_alpha", BC7_ALPHA_PRESETS),
    ] {
        let mut group = c.benchmark_group(group_name);
        group.throughput(Throughput::Elements(src.texels()));
        group.sample_size(10);
        let mut out = vec![0u8; bc7::calc_output_size(src.width, src.height)];
        for (name, preset) in presets {
            let settings = preset();
            group.bench_function(name, |b| {
                b.iter(|| bc7::compress_blocks_into(&settings, black_box(&src.rgba()), &mut out))
            });
        }
        group.finish();
    }
}

/// How the fast presets scale with image size (per-call overhead vs. steady state).
fn bench_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");
    group.sample_size(10);
    for size in [16, 64, 256, 1024] {
        let src = Source::new(&natural(size, size));
        group.throughput(Throughput::Elements(src.texels()));

        let mut out = vec![0u8; bc7::calc_output_size(size, size)];
        let settings = bc7::opaque_very_fast_settings();
        group.bench_with_input(
            BenchmarkId::new("bc7_opaque_very_fast", size),
            &src,
            |b, src| {
                b.iter(|| bc7::compress_blocks_into(&settings, black_box(&src.rgba()), &mut out))
            },
        );

        let mut out = vec![0u8; bc6h::calc_output_size(size, size)];
        let settings = bc6h::very_fast_settings();
        group.bench_with_input(BenchmarkId::new("bc6h_very_fast", size), &src, |b, src| {
            b.iter(|| bc6h::compress_blocks_into(&settings, black_box(&src.rgba16f()), &mut out))
        });
    }
    group.finish();
}

/// Encoder speed depends on content: early-outs make flat blocks cheap and noise expensive.
///
/// ETC1 is deliberately left out: running it right after the BC6H/BC7 kernels here trips
/// `assert(v<pow2(bits))` in the ISPC ETC1 packer (`compress_etc1_half_7` can read
/// uninitialized `colors[q][7..10]` for empty clusters), which aborts the process.
fn bench_content(c: &mut Criterion) {
    let images = [
        ("solid", solid(256, 256, [90, 140, 200, 255])),
        ("gradient", gradient(256, 256)),
        ("natural", natural(256, 256)),
        ("noise", noise(256, 256, 42)),
    ];
    let sources: Vec<(&str, Source)> = images
        .iter()
        .map(|(name, img)| (*name, Source::new(img)))
        .collect();

    let mut group = c.benchmark_group("content");
    group.sample_size(20);
    for (name, src) in &sources {
        group.throughput(Throughput::Elements(src.texels()));

        let mut out = vec![0u8; bc1::calc_output_size(src.width, src.height)];
        group.bench_function(BenchmarkId::new("bc1", name), |b| {
            b.iter(|| bc1::compress_blocks_into(black_box(&src.rgba()), &mut out))
        });

        let mut out = vec![0u8; bc7::calc_output_size(src.width, src.height)];
        let settings = bc7::alpha_basic_settings();
        group.bench_function(BenchmarkId::new("bc7_alpha_basic", name), |b| {
            b.iter(|| bc7::compress_blocks_into(&settings, black_box(&src.rgba()), &mut out))
        });

        let mut out = vec![0u8; bc6h::calc_output_size(src.width, src.height)];
        let settings = bc6h::basic_settings();
        group.bench_function(BenchmarkId::new("bc6h_basic", name), |b| {
            b.iter(|| bc6h::compress_blocks_into(&settings, black_box(&src.rgba16f()), &mut out))
        });
    }
    group.finish();
}

/// API-level overheads: allocating vs. reusing the output, and padded row strides.
fn bench_api(c: &mut Criterion) {
    let image = natural(512, 512);
    let src = Source::new(&image);

    let mut group = c.benchmark_group("api");
    group.throughput(Throughput::Elements(src.texels()));

    group.bench_function("bc1_compress_blocks_alloc", |b| {
        b.iter(|| bc1::compress_blocks(black_box(&src.rgba())))
    });
    let mut out = vec![0u8; bc1::calc_output_size(src.width, src.height)];
    group.bench_function("bc1_compress_blocks_into", |b| {
        b.iter(|| bc1::compress_blocks_into(black_box(&src.rgba()), &mut out))
    });

    // Rows padded to a 256-byte pitch, as GPU readback buffers commonly are.
    let pitch = (src.width as usize * 4).next_multiple_of(256) + 256;
    let mut padded = vec![0u8; pitch * src.height as usize];
    for (dst, row) in padded
        .chunks_exact_mut(pitch)
        .zip(src.rgba8.chunks_exact(src.width as usize * 4))
    {
        dst[..row.len()].copy_from_slice(row);
    }
    let padded_surface = RgbaSurface {
        data: &padded,
        width: src.width,
        height: src.height,
        stride: pitch as u32,
    };
    group.bench_function("bc1_padded_stride", |b| {
        b.iter(|| bc1::compress_blocks_into(black_box(&padded_surface), &mut out))
    });
    group.finish();
}

/// Splitting a surface into horizontal strips and encoding them on scoped threads, the
/// usual way to parallelize these single-threaded encoders.
fn bench_threaded_strips(c: &mut Criterion) {
    let src = Source::new(&natural(1024, 1024));
    let max_threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let settings = bc7::alpha_very_fast_settings();

    let mut group = c.benchmark_group("threaded_strips");
    group.throughput(Throughput::Elements(src.texels()));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));

    let mut out = vec![0u8; bc7::calc_output_size(src.width, src.height)];
    let mut threads = vec![1];
    while threads.last().unwrap() * 2 <= max_threads {
        threads.push(threads.last().unwrap() * 2);
    }
    for n in threads {
        group.bench_function(BenchmarkId::new("bc7_alpha_very_fast", n), |b| {
            b.iter(|| encode_strips(&settings, &src, &mut out, n))
        });
    }
    group.finish();
}

fn encode_strips(settings: &bc7::EncodeSettings, src: &Source, out: &mut [u8], threads: usize) {
    let block_rows = src.height as usize / 4;
    let rows_per_strip = block_rows.div_ceil(threads);
    let in_strip = rows_per_strip * 4 * src.width as usize * 4;
    let out_strip = bc7::calc_output_size(src.width, rows_per_strip as u32 * 4);
    std::thread::scope(|s| {
        for (input, output) in src.rgba8.chunks(in_strip).zip(out.chunks_mut(out_strip)) {
            s.spawn(move || {
                let height = (input.len() / (src.width as usize * 4)) as u32;
                let surface = RgbaSurface {
                    data: input,
                    width: src.width,
                    height,
                    stride: src.width * 4,
                };
                bc7::compress_blocks_into(settings, &surface, output);
            });
        }
    });
}

/// A real photograph (the example image) cropped to 1024x1024, through every codec's
/// default preset. ETC1 is skipped for the same reason as in [`bench_content`].
fn bench_photo(c: &mut Criterion) {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/lambertian.jpg");
    let photo = image::open(path)
        .expect("failed to load examples/lambertian.jpg")
        .to_rgba8();
    let crop = RgbaImage::from_fn(1024, 1024, |x, y| photo.get_pixel(x + 1024, y + 1024).0);

    let mut group = c.benchmark_group("photo_1024");
    group.throughput(Throughput::Elements(1024 * 1024));
    group.sample_size(10);
    for codec in ALL_CODECS.iter().filter(|c| c.name != "etc1") {
        let data = codec.input.convert(&crop);
        let stride = crop.width * codec.input.bytes_per_pixel() as u32;
        let mut out = vec![0u8; (codec.output_size)(crop.width, crop.height)];
        group.bench_function(codec.name, |b| {
            b.iter(|| {
                (codec.compress_into)(black_box(&data), crop.width, crop.height, stride, &mut out)
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_fixed_formats,
    bench_presets,
    bench_size_scaling,
    bench_content,
    bench_api,
    bench_threaded_strips,
    bench_photo,
);
criterion_main!(benches);
