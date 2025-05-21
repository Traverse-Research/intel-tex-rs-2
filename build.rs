/*
ISPC project file builds the kernels as such:
<Command Condition="'$(Configuration)|$(Platform)'=='Release|x64'">ispc -O2 "%(Filename).ispc" -o "$(TargetDir)%(Filename).obj" -h "$(ProjectDir)%(Filename)_ispc.h" --target=sse2,sse4,avx,avx2 --opt=fast-math</Command>
<Outputs Condition="'$(Configuration)|$(Platform)'=='Release|x64'">$(TargetDir)%(Filename).obj;$(TargetDir)%(Filename)_sse2.obj;$(TargetDir)%(Filename)_sse4.obj;$(TargetDir)%(Filename)_avx.obj;$(TargetDir)%(Filename)_avx2.obj;</Outputs>
*/

fn windows_mt_suffix() -> &'static str {
    let target_features = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap();
    if target_features.contains("crt-static") {
        "-MT"
    } else {
        "-MD"
    }
}

#[cfg(feature = "ispc")]
fn main() {
    use ispc_compile::{bindgen::builder, Config, TargetISA};

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    #[allow(deprecated, reason = "Pending ISPC update")]
    let target_isas = match target_arch.as_str() {
        "x86" | "x86_64" => vec![
            TargetISA::SSE2i32x4,
            TargetISA::SSE4i32x4,
            TargetISA::AVX1i32x8,
            TargetISA::AVX2i32x8,
            // TargetISA::AVX512KNLx16,
            TargetISA::AVX512SKXx16,
        ],
        "arm" | "aarch64" => vec![
            // TargetISA::Neoni32x4,
            TargetISA::Neoni32x8,
        ],
        x => panic!("Unsupported target architecture {x}"),
    };

    Config::new()
        .opt_level(2)
        .woff()
        .target_isas(target_isas.clone())
        .out_dir("src/ispc")
        .file("vendor/ispc_texcomp/kernel.ispc")
        .bindgen_builder(builder().allowlist_function(r#"CompressBlocks(BC\dH?|ETC1)_ispc"#))
        .compile("kernel");

    Config::new()
        .opt_level(2)
        .woff()
        .target_isas(target_isas)
        .out_dir("src/ispc")
        .file("vendor/ispc_texcomp/kernel_astc.ispc")
        .bindgen_builder(
            builder()
                .allowlist_function("astc_rank_ispc")
                .allowlist_function("astc_encode_ispc")
                .allowlist_function("get_programCount"),
        )
        .compile("kernel_astc");

    // ASTC encoder `extern "C"`'s some code, so we need to make sure to link
    // and compile that in. The relevant codepath using this functionality is
    // completely commented out and only results in linker errors on MSVC which
    // is unable to deadstrip it and the requirement for a single symbol.
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let target_family = std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap();
    let target_mt = if target_family == "windows" {
        windows_mt_suffix()
    } else {
        ""
    };

    cc::Build::new()
        .include(out_dir)
        .file("vendor/ispc_texcomp/ispc_texcomp_astc.cpp")
        .out_dir("src/ispc")
        // Append the target triple since we'll be checking this file in, just like
        // the compiled kernels above.
        // On Windows, also append whether we were compiled against the dynamic or static
        // multithread library, so that we can copy and link against this correctly when using
        // pregenerated libraries.
        .compile(&format!(
            "ispc_texcomp_astc{}{target_mt}",
            std::env::var("TARGET").unwrap()
        ));
}

#[cfg(not(feature = "ispc"))]
fn main() {
    use std::path::Path;

    ispc_rt::PackagedModule::new("kernel")
        .lib_path("src/ispc")
        .link();

    ispc_rt::PackagedModule::new("kernel_astc")
        .lib_path("src/ispc")
        .link();

    // Manually link ispc_texcomp_astc since it's not an ISPC module but C++.

    let mut libname = format!("ispc_texcomp_astc{}", std::env::var("TARGET").unwrap());
    let libfile = match std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap().as_str() {
        "unix" => format!("lib{libname}.a"),
        "windows" => {
            let windows_mt = windows_mt_suffix();
            libname += windows_mt;
            format!("{libname}.lib")
        }
        x => panic!("Unknown target family {x}"),
    };

    println!("cargo:rustc-link-lib=static={libname}");

    let libpath = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ispc");
    println!("cargo:rustc-link-search=native={}", libpath.display());
    println!("cargo:rerun-if-changed={}", libpath.join(libfile).display());
}
