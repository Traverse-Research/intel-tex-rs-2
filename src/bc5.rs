use crate::bindings::kernel;
use crate::RgSurface;

#[inline(always)]
pub fn calc_output_size(width: u32, height: u32) -> usize {
    // BC5 uses 16 bytes to store each 4×4 block, giving it an average data rate of 1 byte per pixel.
    let block_count = (width as usize * height as usize).div_ceil(16);
    block_count * 16
}

pub fn compress_blocks(surface: &RgSurface) -> Vec<u8> {
    let output_size = calc_output_size(surface.width, surface.height);
    let mut output = vec![0u8; output_size];
    compress_blocks_into(surface, &mut output);
    output
}

pub fn compress_blocks_into(surface: &RgSurface, blocks: &mut [u8]) {
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
        kernel::CompressBlocksBC5_ispc(&mut surface, blocks.as_mut_ptr());
    }
}
