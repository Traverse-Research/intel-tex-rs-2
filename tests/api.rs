//! Format-agnostic API contract tests, instantiated once for every codec.
//!
//! These check the behavior of `compress_blocks` / `compress_blocks_into` that callers rely
//! on regardless of the block format: output sizing, block layout, stride handling,
//! determinism and input validation.

mod common;

use common::*;

/// Surface data with every row padded by `pad` bytes of garbage.
fn padded(codec: &Codec, image: &RgbaImage, pad: usize) -> (Vec<u8>, u32) {
    let bpp = codec.input.bytes_per_pixel();
    let tight = codec.input.convert(image);
    let row = image.width as usize * bpp;
    let stride = row + pad;
    let mut data = vec![0xABu8; stride * image.height as usize];
    for y in 0..image.height as usize {
        data[y * stride..y * stride + row].copy_from_slice(&tight[y * row..(y + 1) * row]);
    }
    (data, stride as u32)
}

fn tight_stride(codec: &Codec, width: u32) -> u32 {
    width * codec.input.bytes_per_pixel() as u32
}

fn output_size_matches_block_count(codec: Codec) {
    for (w, h) in [
        (4, 4),
        (8, 4),
        (4, 8),
        (16, 16),
        (64, 32),
        (128, 256),
        (1024, 4),
    ] {
        let blocks = (w as usize / 4) * (h as usize / 4);
        assert_eq!(
            (codec.output_size)(w, h),
            blocks * codec.block_bytes,
            "{} {w}x{h}",
            codec.name
        );
    }
}

fn compress_returns_output_size_bytes(codec: Codec) {
    for (w, h) in [(4, 4), (12, 8), (32, 32)] {
        let out = codec.compress_image(&natural(w, h));
        assert_eq!(
            out.len(),
            (codec.output_size)(w, h),
            "{} {w}x{h}",
            codec.name
        );
    }
}

fn compress_matches_compress_into(codec: Codec) {
    let image = natural(32, 24);
    let data = codec.input.convert(&image);
    let stride = tight_stride(&codec, image.width);
    let expected = (codec.compress)(&data, image.width, image.height, stride);

    // Pre-fill with garbage to make sure every byte is written.
    let mut out = vec![0x5Au8; (codec.output_size)(image.width, image.height)];
    (codec.compress_into)(&data, image.width, image.height, stride, &mut out);
    assert_eq!(out, expected);
}

fn compress_into_overwrites_every_byte(codec: Codec) {
    let image = gradient(16, 16);
    let data = codec.input.convert(&image);
    let stride = tight_stride(&codec, image.width);
    let size = (codec.output_size)(image.width, image.height);

    let mut a = vec![0x00u8; size];
    let mut b = vec![0xFFu8; size];
    (codec.compress_into)(&data, image.width, image.height, stride, &mut a);
    (codec.compress_into)(&data, image.width, image.height, stride, &mut b);
    assert_eq!(a, b);
}

fn is_deterministic(codec: Codec) {
    let image = noise(32, 32, 7);
    let first = codec.compress_image(&image);
    for _ in 0..3 {
        assert_eq!(codec.compress_image(&image), first);
    }
}

fn padded_stride_matches_tight(codec: Codec) {
    let image = natural(20, 12);
    let expected = codec.compress_image(&image);
    for pad in [1, 3, 8, 64] {
        let (data, stride) = padded(&codec, &image, pad);
        let out = (codec.compress)(&data, image.width, image.height, stride);
        assert_eq!(out, expected, "{} pad={pad}", codec.name);
    }
}

fn subrect_via_stride_matches_crop(codec: Codec) {
    let full = natural(32, 32);
    let bpp = codec.input.bytes_per_pixel();
    let data = codec.input.convert(&full);
    let stride = tight_stride(&codec, full.width);

    // Compress the 16x12 sub-rectangle at (8, 4) by offsetting into the parent buffer.
    let (x0, y0, w, h) = (8u32, 4u32, 16u32, 12u32);
    let offset = y0 as usize * stride as usize + x0 as usize * bpp;
    // The surface must cover `stride * height` bytes, so hand it the rest of the buffer.
    let out = (codec.compress)(&data[offset..], w, h, stride);

    assert_eq!(out, codec.compress_image(&full.crop(x0, y0, w, h)));
}

fn blocks_are_row_major(codec: Codec) {
    let image = natural(16, 12);
    let whole = codec.compress_image(&image);
    let bb = codec.block_bytes;
    for by in 0..3 {
        for bx in 0..4 {
            let block = codec.compress_image(&image.crop(bx * 4, by * 4, 4, 4));
            let index = (by * 4 + bx) as usize;
            assert_eq!(
                &whole[index * bb..(index + 1) * bb],
                &block[..],
                "{} block ({bx}, {by})",
                codec.name
            );
        }
    }
}

fn strips_match_whole(codec: Codec) {
    // Encoding horizontal strips into sub-slices of the output (e.g. one per thread) must
    // give the same result as encoding the whole surface at once.
    let image = natural(24, 32);
    let expected = codec.compress_image(&image);
    let data = codec.input.convert(&image);
    let stride = tight_stride(&codec, image.width);

    let strip_rows = 8;
    let out_strip = (codec.output_size)(image.width, strip_rows);
    let mut out = vec![0u8; expected.len()];
    for (i, output) in out.chunks_mut(out_strip).enumerate() {
        let input = &data[i * strip_rows as usize * stride as usize..];
        (codec.compress_into)(input, image.width, strip_rows, stride, output);
    }
    assert_eq!(out, expected);
}

fn blocks_are_independent(codec: Codec) {
    let base = natural(16, 16);
    let before = codec.compress_image(&base);

    // Scribble over block (2, 1) only.
    let mut modified = base.clone();
    let mut rng = Rng::new(99);
    for y in 4..8 {
        for x in 8..12 {
            modified.pixels[(y * 16 + x) as usize] =
                [rng.next_u8(), rng.next_u8(), rng.next_u8(), rng.next_u8()];
        }
    }
    let after = codec.compress_image(&modified);

    let bb = codec.block_bytes;
    let changed = 4 + 2;
    for i in 0..16 {
        let (a, b) = (&before[i * bb..(i + 1) * bb], &after[i * bb..(i + 1) * bb]);
        if i == changed {
            assert_ne!(a, b, "{}: modified block should change", codec.name);
        } else {
            assert_eq!(a, b, "{}: block {i} changed unexpectedly", codec.name);
        }
    }
}

fn identical_blocks_encode_identically(codec: Codec) {
    let tile = natural(4, 4);
    let tiled = RgbaImage::from_fn(32, 16, |x, y| tile.pixel(x % 4, y % 4));
    let out = codec.compress_image(&tiled);
    let first = &out[..codec.block_bytes];
    for chunk in out.chunks_exact(codec.block_bytes) {
        assert_eq!(chunk, first);
    }
}

fn empty_surface_produces_no_output(codec: Codec) {
    assert_eq!((codec.output_size)(0, 0), 0);
    assert_eq!((codec.output_size)(0, 64), 0);
    assert_eq!((codec.output_size)(64, 0), 0);
    let out = (codec.compress)(&[], 0, 0, 0);
    assert!(out.is_empty());
}

fn full_blocks_match_aligned_crop(codec: Codec, width: u32, height: u32) {
    // Only whole 4x4 blocks are encoded; they must match an encode of the aligned crop.
    let image = natural(width, height);
    let out = codec.compress_image(&image);
    assert_eq!(out.len(), (codec.output_size)(width, height));
    let (blocks_x, blocks_y) = (width as usize / 4, height as usize / 4);
    let aligned = codec.compress_image(&image.crop(0, 0, blocks_x as u32 * 4, blocks_y as u32 * 4));

    let row_bytes = blocks_x * codec.block_bytes;
    for by in 0..blocks_y {
        assert_eq!(
            &out[by * row_bytes..(by + 1) * row_bytes],
            &aligned[by * row_bytes..(by + 1) * row_bytes],
            "{} {width}x{height} block row {by}",
            codec.name
        );
    }
}

fn wrong_output_len(codec: Codec, delta: isize) {
    let image = gradient(8, 8);
    let data = codec.input.convert(&image);
    let len = (codec.output_size)(8, 8) as isize + delta;
    let mut out = vec![0u8; len as usize];
    (codec.compress_into)(&data, 8, 8, tight_stride(&codec, 8), &mut out);
}

fn too_little_input_data(codec: Codec) {
    let image = gradient(8, 8);
    let data = codec.input.convert(&image);
    (codec.compress)(&data[..data.len() - 1], 8, 8, tight_stride(&codec, 8));
}

fn stride_larger_than_data(codec: Codec) {
    let image = gradient(8, 8);
    let data = codec.input.convert(&image);
    (codec.compress)(&data, 8, 8, tight_stride(&codec, 8) + 1);
}

macro_rules! api_tests {
    ($($module:ident => $codec:expr),* $(,)?) => {$(
        mod $module {
            use super::*;

            #[test]
            fn output_size_matches_block_count() {
                super::output_size_matches_block_count($codec);
            }

            #[test]
            fn compress_returns_output_size_bytes() {
                super::compress_returns_output_size_bytes($codec);
            }

            #[test]
            fn compress_matches_compress_into() {
                super::compress_matches_compress_into($codec);
            }

            #[test]
            fn compress_into_overwrites_every_byte() {
                super::compress_into_overwrites_every_byte($codec);
            }

            #[test]
            fn is_deterministic() {
                super::is_deterministic($codec);
            }

            #[test]
            fn padded_stride_matches_tight() {
                super::padded_stride_matches_tight($codec);
            }

            #[test]
            fn subrect_via_stride_matches_crop() {
                super::subrect_via_stride_matches_crop($codec);
            }

            #[test]
            fn blocks_are_row_major() {
                super::blocks_are_row_major($codec);
            }

            #[test]
            fn strips_match_whole() {
                super::strips_match_whole($codec);
            }

            #[test]
            fn blocks_are_independent() {
                super::blocks_are_independent($codec);
            }

            #[test]
            fn identical_blocks_encode_identically() {
                super::identical_blocks_encode_identically($codec);
            }

            #[test]
            fn empty_surface_produces_no_output() {
                super::empty_surface_produces_no_output($codec);
            }

            #[test]
            fn unaligned_height_encodes_full_blocks() {
                super::full_blocks_match_aligned_crop($codec, 16, 13);
            }

            #[test]
            #[ignore = "the ISPC kernels advance output rows by `width * block_bytes / 16` bytes, \
                        so widths that aren't a multiple of 4 produce a skewed block layout"]
            fn unaligned_width_encodes_full_blocks() {
                super::full_blocks_match_aligned_crop($codec, 18, 12);
            }

            #[test]
            #[should_panic]
            fn output_buffer_too_small_panics() {
                super::wrong_output_len($codec, -1);
            }

            #[test]
            #[should_panic]
            fn output_buffer_too_large_panics() {
                super::wrong_output_len($codec, 1);
            }

            #[test]
            #[should_panic]
            fn too_little_input_data_panics() {
                super::too_little_input_data($codec);
            }

            #[test]
            #[should_panic]
            fn stride_larger_than_data_panics() {
                super::stride_larger_than_data($codec);
            }
        }
    )*};
}

api_tests! {
    bc1 => BC1,
    bc3 => BC3,
    bc4 => BC4,
    bc5 => BC5,
    bc6h => BC6H,
    bc7_opaque => BC7_OPAQUE,
    bc7_alpha => BC7_ALPHA,
    etc1 => ETC1,
}
