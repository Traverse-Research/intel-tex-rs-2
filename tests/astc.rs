// The ASTC kernel calls into `ispc_texcomp_astc`, which is C++, so this test only links if the
// C++ standard library is linked too. A program that never calls the ASTC kernel can still link
// it, depending on how its code is split into codegen units.

#[test]
fn astc_kernel_links() {
    let program_count = unsafe { intel_tex_2::bindings::kernel_astc::get_programCount() };

    assert!(program_count > 0);
}
