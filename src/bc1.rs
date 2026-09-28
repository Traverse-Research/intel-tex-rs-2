use crate::bindings::kernel;
use crate::RgbaSurface;

#[inline(always)]
pub fn calc_output_size(width: u32, height: u32) -> usize {
    // BC1 uses 8 bytes to store each 4×4 block, giving it an average data rate of 0.5 bytes per pixel.
    let block_count = (width as usize * height as usize).div_ceil(16);
    block_count * 8
}

pub fn compress_blocks(surface: &RgbaSurface) -> Vec<u8> {
    let output_size = calc_output_size(surface.width, surface.height);
    let mut output = vec![0u8; output_size];
    compress_blocks_into(surface, &mut output);
    output
}

pub fn compress_blocks_into(surface: &RgbaSurface, blocks: &mut [u8]) {
    assert_eq!(
        blocks.len(),
        calc_output_size(surface.width, surface.height)
    );
    assert!(surface.data.len() >= surface.height as usize * surface.stride as usize);

    let mut surface = kernel::rgba_surface {
        width: surface.width as i32,
        height: surface.height as i32,
        stride: surface.stride as i32,
        ptr: surface.data.as_ptr() as *mut u8,
    };

    unsafe {
        kernel::CompressBlocksBC1_ispc(&mut surface, blocks.as_mut_ptr());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_size() {
        assert_eq!(calc_output_size(0, 0), 0);
        assert_eq!(calc_output_size(4, 4), 8);
        assert_eq!(calc_output_size(8, 4), 2 * 8);
        assert_eq!(calc_output_size(256, 256), 64 * 64 * 8);
        assert_eq!(calc_output_size(4096, 4096), 1024 * 1024 * 8);
        // Partial blocks are rounded up by texel count.
        assert_eq!(calc_output_size(1, 1), 8);
        assert_eq!(calc_output_size(4, 5), 2 * 8);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn output_size_does_not_overflow_u32() {
        assert_eq!(calc_output_size(65536, 65536), 16384 * 16384 * 8);
    }
}
