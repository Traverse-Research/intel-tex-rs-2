extern crate ispc_rt;

#[cfg(feature = "ispc")]

/*
ISPC project file builds the kernels as such:
<Command Condition="'$(Configuration)|$(Platform)'=='Release|x64'">ispc -O2 "%(Filename).ispc" -o "$(TargetDir)%(Filename).obj" -h "$(ProjectDir)%(Filename)_ispc.h" --target=sse2,sse4,avx,avx2 --opt=fast-math</Command>
<Outputs Condition="'$(Configuration)|$(Platform)'=='Release|x64'">$(TargetDir)%(Filename).obj;$(TargetDir)%(Filename)_sse2.obj;$(TargetDir)%(Filename)_sse4.obj;$(TargetDir)%(Filename)_avx.obj;$(TargetDir)%(Filename)_avx2.obj;</Outputs>
*/

#[cfg(feature = "ispc")]
use ispc_compile::{TargetISA, TargetOS};

#[cfg(feature = "ispc")]
#[cfg(target_os = "linux")]
fn get_target_os() -> TargetOS  {
    TargetOS::Linux
}

#[cfg(feature = "ispc")]
#[cfg(target_os = "windows")]
fn get_target_os() -> TargetOS {
    TargetOS::Windows
}

#[cfg(feature = "ispc")]
#[cfg(target_os = "macos")]
fn get_target_os() -> TargetOS {
    TargetOS::Macos
}

#[cfg(feature = "ispc")]
fn compile_kernel() {

    ispc_compile::Config::new()
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
        .target_os(get_target_os())
        .out_dir("src/ispc")
        .compile("kernel");

    ispc_compile::Config::new()
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
        .target_os(get_target_os())
        .out_dir("src/ispc")
        .compile("kernel_astc");

    // ASTC encoder `extern "C"`'s some code, so we need to make sure to link
    // and compile that in.
    let out_dir = std::env::var("OUT_DIR").unwrap();

    cc::Build::new()
        .include(out_dir)
        .file("vendor/ispc_texcomp/ispc_texcomp_astc.cpp")
        .out_dir("src/ispc")
        // format the output name such that ispc_rt::PackagedModule can just pick it up easily
        .compile(&format!("ispc_texcomp_astc{}", std::env::var("TARGET").unwrap()));

}

#[cfg(not(feature = "ispc"))]
fn compile_kernel() {
    ispc_rt::PackagedModule::new("kernel")
        .lib_path("src/ispc")
        .link();

    ispc_rt::PackagedModule::new("kernel_astc")
        .lib_path("src/ispc")
        .link();

    // slightly re-use the PackagedModule logic here since it's just linking in libs
    ispc_rt::PackagedModule::new("ispc_texcomp_astc")
        .lib_path("src/ispc")
        .link();
}

fn main() {
    compile_kernel();
}
