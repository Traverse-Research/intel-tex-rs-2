/*
ISPC project file builds the kernels as such:
<Command Condition="'$(Configuration)|$(Platform)'=='Release|x64'">ispc -O2 "%(Filename).ispc" -o "$(TargetDir)%(Filename).obj" -h "$(ProjectDir)%(Filename)_ispc.h" --target=sse2,sse4,avx,avx2 --opt=fast-math</Command>
<Outputs Condition="'$(Configuration)|$(Platform)'=='Release|x64'">$(TargetDir)%(Filename).obj;$(TargetDir)%(Filename)_sse2.obj;$(TargetDir)%(Filename)_sse4.obj;$(TargetDir)%(Filename)_avx.obj;$(TargetDir)%(Filename)_avx2.obj;</Outputs>
*/

#[cfg(feature = "ispc")]
fn compile_kernel() {
    use ispc_compile::{bindgen::builder, Config, TargetISA};

    Config::new()
        .file("vendor/ispc_texcomp/kernel.ispc")
        .opt_level(2)
        .woff()
        .target_isas(vec![
            TargetISA::SSE2i32x4,
            TargetISA::SSE4i32x4,
            TargetISA::AVX1i32x8,
            TargetISA::AVX2i32x8,
            TargetISA::AVX512KNLi32x16,
            TargetISA::AVX512SKXi32x16,
        ])
        .bindgen_builder(builder().allowlist_function(r#"CompressBlocks(BC\dH?|ETC1)_ispc"#))
        .out_dir("src/ispc")
        .compile("kernel");

    Config::new()
        .file("vendor/ispc_texcomp/kernel_astc.ispc")
        .opt_level(2)
        .woff()
        .target_isas(vec![
            TargetISA::SSE2i32x4,
            TargetISA::SSE4i32x4,
            TargetISA::AVX1i32x8,
            TargetISA::AVX2i32x8,
            TargetISA::AVX512KNLi32x16,
            TargetISA::AVX512SKXi32x16,
        ])
        .bindgen_builder(
            builder()
                .allowlist_function("astc_rank_ispc")
                .allowlist_function("astc_encode_ispc")
                .allowlist_function("get_programCount"),
        )
        .out_dir("src/ispc")
        .compile("kernel_astc");

    // ASTC encoder `extern "C"`'s some code, so we need to make sure to link
    // and compile that in. The relevant codepath using this functionality is
    // completely commented out and only results in linker errors on MSVC which
    // is unable to deadstrip it and the requirement for a single symbol.
    let out_dir = std::env::var("OUT_DIR").unwrap();

    cc::Build::new()
        .include(out_dir)
        .file("vendor/ispc_texcomp/ispc_texcomp_astc.cpp")
        .out_dir("src/ispc")
        // Append the target triple since we'll be checking this file in, just like
        // the compiled kernels above.
        .compile(&format!(
            "ispc_texcomp_astc{}",
            std::env::var("TARGET").unwrap()
        ));
}

#[cfg(not(feature = "ispc"))]
fn compile_kernel() {
    use std::path::Path;

    ispc_rt::PackagedModule::new("kernel")
        .lib_path("src/ispc")
        .link();

    ispc_rt::PackagedModule::new("kernel_astc")
        .lib_path("src/ispc")
        .link();

    // Manually link ispc_texcomp_astc since it's not an ISPC module but C++.

    let libname = format!("ispc_texcomp_astc{}", std::env::var("TARGET").unwrap());
    let libpath = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ispc");
    let libfile = match std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap().as_str() {
        "unix" => format!("lib{}.a", libname),
        "windows" => format!("lib{}.a", libname),
        x => panic!("Unknown target family {}", x),
    };

    println!("cargo:rustc-link-lib=static={}", libname);
    println!("cargo:rerun-if-changed={}", libpath.join(libfile).display());
    println!("cargo:rustc-link-search=native={}", libpath.display());
}

fn main() {
    compile_kernel();
}
