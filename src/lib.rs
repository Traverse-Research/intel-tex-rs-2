#[allow(deref_nullptr)]
pub mod bindings {
    use ispc_rt::ispc_module;
    ispc_module!(kernel);
    ispc_module!(kernel_astc);
}

pub mod astc;
pub mod bc1;
pub mod bc3;
pub mod bc4;
pub mod bc5;
pub mod bc6h;
pub mod bc7;
pub mod etc1;

/// Describes a 2D image to block-compress.
#[derive(Debug, Copy, Clone)]
pub struct Surface<'a, const COMPONENTS: usize> {
    /// The pixel data for the image.
    /// The data does not need to be tightly packed, but if it isn't, stride must be different from `width * 4`.
    ///
    /// Expected to be at least `stride * height`.
    pub data: &'a [u8],
    /// The width of the image in texels.
    pub width: u32,
    /// The height of the image in texels.
    pub height: u32,
    /// The stride between the rows of the image, in bytes.
    /// If `data` is tightly packed, this is expected to be `width * 4`.
    pub stride: u32,
}

pub type RgbaSurface<'a> = Surface<'a, 4>;
pub type RgSurface<'a> = Surface<'a, 2>;
pub type RSurface<'a> = Surface<'a, 1>;
