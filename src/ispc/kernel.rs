#[allow(non_camel_case_types,dead_code,non_upper_case_globals,non_snake_case,improper_ctypes)]
pub mod kernel {
/* automatically generated by rust-bindgen 0.69.1 */

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rgba_surface {
    pub ptr: *mut u8,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
}
#[test]
fn bindgen_test_layout_rgba_surface() {
    const UNINIT: ::std::mem::MaybeUninit<rgba_surface> = ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<rgba_surface>(),
        24usize,
        concat!("Size of: ", stringify!(rgba_surface))
    );
    assert_eq!(
        ::std::mem::align_of::<rgba_surface>(),
        8usize,
        concat!("Alignment of ", stringify!(rgba_surface))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).ptr) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(rgba_surface),
            "::",
            stringify!(ptr)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).width) as usize - ptr as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(rgba_surface),
            "::",
            stringify!(width)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).height) as usize - ptr as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(rgba_surface),
            "::",
            stringify!(height)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).stride) as usize - ptr as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(rgba_surface),
            "::",
            stringify!(stride)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct bc6h_enc_settings {
    pub slow_mode: bool,
    pub fast_mode: bool,
    pub refineIterations_1p: i32,
    pub refineIterations_2p: i32,
    pub fastSkipTreshold: i32,
}
#[test]
fn bindgen_test_layout_bc6h_enc_settings() {
    const UNINIT: ::std::mem::MaybeUninit<bc6h_enc_settings> = ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<bc6h_enc_settings>(),
        16usize,
        concat!("Size of: ", stringify!(bc6h_enc_settings))
    );
    assert_eq!(
        ::std::mem::align_of::<bc6h_enc_settings>(),
        4usize,
        concat!("Alignment of ", stringify!(bc6h_enc_settings))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).slow_mode) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(bc6h_enc_settings),
            "::",
            stringify!(slow_mode)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).fast_mode) as usize - ptr as usize },
        1usize,
        concat!(
            "Offset of field: ",
            stringify!(bc6h_enc_settings),
            "::",
            stringify!(fast_mode)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).refineIterations_1p) as usize - ptr as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(bc6h_enc_settings),
            "::",
            stringify!(refineIterations_1p)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).refineIterations_2p) as usize - ptr as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(bc6h_enc_settings),
            "::",
            stringify!(refineIterations_2p)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).fastSkipTreshold) as usize - ptr as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(bc6h_enc_settings),
            "::",
            stringify!(fastSkipTreshold)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct bc7_enc_settings {
    pub mode_selection: [bool; 4usize],
    pub refineIterations: [i32; 8usize],
    pub skip_mode2: bool,
    pub fastSkipTreshold_mode1: i32,
    pub fastSkipTreshold_mode3: i32,
    pub fastSkipTreshold_mode7: i32,
    pub mode45_channel0: i32,
    pub refineIterations_channel: i32,
    pub channels: i32,
}
#[test]
fn bindgen_test_layout_bc7_enc_settings() {
    const UNINIT: ::std::mem::MaybeUninit<bc7_enc_settings> = ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<bc7_enc_settings>(),
        64usize,
        concat!("Size of: ", stringify!(bc7_enc_settings))
    );
    assert_eq!(
        ::std::mem::align_of::<bc7_enc_settings>(),
        4usize,
        concat!("Alignment of ", stringify!(bc7_enc_settings))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).mode_selection) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(bc7_enc_settings),
            "::",
            stringify!(mode_selection)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).refineIterations) as usize - ptr as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(bc7_enc_settings),
            "::",
            stringify!(refineIterations)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).skip_mode2) as usize - ptr as usize },
        36usize,
        concat!(
            "Offset of field: ",
            stringify!(bc7_enc_settings),
            "::",
            stringify!(skip_mode2)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).fastSkipTreshold_mode1) as usize - ptr as usize },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(bc7_enc_settings),
            "::",
            stringify!(fastSkipTreshold_mode1)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).fastSkipTreshold_mode3) as usize - ptr as usize },
        44usize,
        concat!(
            "Offset of field: ",
            stringify!(bc7_enc_settings),
            "::",
            stringify!(fastSkipTreshold_mode3)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).fastSkipTreshold_mode7) as usize - ptr as usize },
        48usize,
        concat!(
            "Offset of field: ",
            stringify!(bc7_enc_settings),
            "::",
            stringify!(fastSkipTreshold_mode7)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).mode45_channel0) as usize - ptr as usize },
        52usize,
        concat!(
            "Offset of field: ",
            stringify!(bc7_enc_settings),
            "::",
            stringify!(mode45_channel0)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).refineIterations_channel) as usize - ptr as usize },
        56usize,
        concat!(
            "Offset of field: ",
            stringify!(bc7_enc_settings),
            "::",
            stringify!(refineIterations_channel)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).channels) as usize - ptr as usize },
        60usize,
        concat!(
            "Offset of field: ",
            stringify!(bc7_enc_settings),
            "::",
            stringify!(channels)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct etc_enc_settings {
    pub fastSkipTreshold: i32,
}
#[test]
fn bindgen_test_layout_etc_enc_settings() {
    const UNINIT: ::std::mem::MaybeUninit<etc_enc_settings> = ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<etc_enc_settings>(),
        4usize,
        concat!("Size of: ", stringify!(etc_enc_settings))
    );
    assert_eq!(
        ::std::mem::align_of::<etc_enc_settings>(),
        4usize,
        concat!("Alignment of ", stringify!(etc_enc_settings))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).fastSkipTreshold) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(etc_enc_settings),
            "::",
            stringify!(fastSkipTreshold)
        )
    );
}
extern "C" {
    pub fn CompressBlocksBC1_ispc(src: *mut rgba_surface, dst: *mut u8);
}
extern "C" {
    pub fn CompressBlocksBC3_ispc(src: *mut rgba_surface, dst: *mut u8);
}
extern "C" {
    pub fn CompressBlocksBC4_ispc(src: *mut rgba_surface, dst: *mut u8);
}
extern "C" {
    pub fn CompressBlocksBC5_ispc(src: *mut rgba_surface, dst: *mut u8);
}
extern "C" {
    pub fn CompressBlocksBC6H_ispc(
        src: *mut rgba_surface,
        dst: *mut u8,
        settings: *mut bc6h_enc_settings,
    );
}
extern "C" {
    pub fn CompressBlocksBC7_ispc(
        src: *mut rgba_surface,
        dst: *mut u8,
        settings: *mut bc7_enc_settings,
    );
}
extern "C" {
    pub fn CompressBlocksETC1_ispc(
        src: *mut rgba_surface,
        dst: *mut u8,
        settings: *mut etc_enc_settings,
    );
}
}