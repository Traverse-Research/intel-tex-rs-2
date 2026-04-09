use image::GenericImageView;
use image::ImageBuffer;
use image::Luma;
use image::LumaA;
use image::Pixel;
use intel_tex_2::bc4;
use intel_tex_2::bc5;
use intel_tex_2::bc7;
use std::fs::File;
use std::path::Path;

use ddsfile::{AlphaMode, Caps2, D3D10ResourceDimension, Dds, DxgiFormat, NewDxgiParams};

fn main() {
    let rgb_img = image::open(Path::new("examples/lambertian.jpg")).unwrap();

    let (width, height) = rgb_img.dimensions();
    println!("Width is {}", width);
    println!("Height is {}", height);
    println!("ColorType is {:?}", rgb_img.color());

    let mut rgba_img = ImageBuffer::new(width, height);
    let mut rg_img = ImageBuffer::new(width, height);
    let mut r_img = ImageBuffer::new(width, height);

    println!("Converting RGB -> RGBA/RG/R"); // could be optimized
    for x in 0u32..width {
        for y in 0u32..height {
            let pixel = rgb_img.get_pixel(x, y);
            let pixel_rgba = pixel.to_rgba();
            let pixel_rg = LumaA::from([pixel_rgba[0], pixel_rgba[1]]);
            let pixel_r = Luma::from([pixel_rgba[0]]);
            rgba_img.put_pixel(x, y, pixel_rgba);
            rg_img.put_pixel(x, y, pixel_rg);
            r_img.put_pixel(x, y, pixel_r);
        }
    }

    let block_count = (width as usize * height as usize).div_ceil(16);
    println!("Block count: {}", block_count);
    let dds_defaults = NewDxgiParams {
        height,
        width,
        depth: Some(1),
        format: DxgiFormat::BC7_UNorm,
        mipmap_levels: Some(1),
        array_layers: Some(1),
        caps2: Some(Caps2::empty()),
        is_cubemap: false,
        resource_dimension: D3D10ResourceDimension::Texture2D,
        alpha_mode: AlphaMode::Opaque,
    };
    // BC4
    {
        let mut dds = Dds::new_dxgi(NewDxgiParams {
            format: DxgiFormat::BC4_UNorm,
            ..dds_defaults
        })
        .unwrap();

        let surface = intel_tex_2::RSurface {
            width,
            height,
            stride: width,
            data: &r_img,
        };

        println!("Compressing to BC4...");
        bc4::compress_blocks_into(&surface, dds.get_mut_data(0 /* layer */).unwrap());
        println!("  Done!");
        println!("Saving lambertian_bc4.dds file");
        let mut dds_file = File::create("examples/lambertian_bc4.dds").unwrap();
        dds.write(&mut dds_file).expect("Failed to write dds file");
    }
    // BC5
    {
        let mut dds = Dds::new_dxgi(NewDxgiParams {
            format: DxgiFormat::BC5_UNorm,
            ..dds_defaults
        })
        .unwrap();
        let surface = intel_tex_2::RgSurface {
            width,
            height,
            stride: width * 2,
            data: &rg_img,
        };

        println!("Compressing to BC5...");
        bc5::compress_blocks_into(&surface, dds.get_mut_data(0 /* layer */).unwrap());
        println!("  Done!");
        println!("Saving lambertian_bc5.dds file");
        let mut dds_file = File::create("examples/lambertian_bc5.dds").unwrap();
        dds.write(&mut dds_file).expect("Failed to write dds file");
    }
    // BC7
    {
        let mut dds = Dds::new_dxgi(NewDxgiParams {
            format: DxgiFormat::BC7_UNorm,
            ..dds_defaults
        })
        .unwrap();
        let surface = intel_tex_2::RgbaSurface {
            width,
            height,
            stride: width * 4,
            data: &rgba_img,
        };

        println!("Compressing to BC7...");
        bc7::compress_blocks_into(
            &bc7::opaque_ultra_fast_settings(),
            &surface,
            dds.get_mut_data(0 /* layer */).unwrap(),
        );
        println!("  Done!");
        println!("Saving lambertian_bc7.dds file");
        let mut dds_file = File::create("examples/lambertian_bc7.dds").unwrap();
        dds.write(&mut dds_file).expect("Failed to write dds file");
    }
}
