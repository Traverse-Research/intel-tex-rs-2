#[macro_use]
extern crate ispc_rt;

pub mod bindings {
    ispc_module!(kernel);
    ispc_module!(kernel_astc);
}

pub mod astc;
pub mod bc1;
pub mod bc3;
pub mod bc6h;
pub mod bc7;
pub mod etc1;

/// Describes a 2D uncompressed little endian RGBA image.
#[derive(Debug, Copy, Clone)]
pub struct RgbaSurface<'a> {
    /// The pixel data for the image.
    /// The data does not need to be tightly packed, but if it isn't, stride must be different from `width * 4`.
    /// 
    /// Expected to be at least `stride * height` bytes long.
    pub data: &'a [u8],
    pub width: u32,
    pub height: u32,
    /// The stride between the rows of pixels, in bytes.
    pub stride_in_bytes: u32,
}

#[inline(always)]
pub fn divide_up_by_multiple(val: u32, align: u32) -> u32 {
    let mask: u32 = align - 1;
    (val + mask) / align
}
