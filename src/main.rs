use image::{GenericImageView, Rgb, RgbImage, imageops};
use palette::{
    color_difference::{DeltaE, ImprovedCiede2000},
    IntoColor, Lab, Srgb,
};



fn main() {
    let img = image::open("image4.png").expect("Failed to open image");
    let (width, height) = img.dimensions();
    println!("Original dimensions: {}x{}", width, height);

    // Upscale to 2x resolution
    let hires_width = width * 2;
    let hires_height = height * 2;
    println!("Upscaling to {}x{}...", hires_width, hires_height);
    let hires_img = imageops::resize(
        &img,
        hires_width,
        hires_height,
        imageops::FilterType::CatmullRom,
    );

  let hex_palette = [
  // Backgrounds
  "#282a36", // Background
  "#21222c", // Current Line / Selection
  "#3a3f51", // Comment

  // Foreground / Text
  "#f8f8f2", // Foreground (Default Text)
  "#6272a4", // Selection Background (Similar to Comment)

  // Accents
  "#ff79c6", // Pink (Functions, Keywords, Strings)
  "#bd93f9", // Purple (Variables, Operators)
  "#ffb86c", // Orange (Numbers, Booleans)
  "#50fa7b", // Green (Strings, RegEx)
  "#8be9fd", // Cyan (Classes, Types)
  "#f1fa8c"  // Yellow (Parameters, Special Keywords)
];
    let lab_palette: Vec<Lab> = hex_palette.iter().map(|&h| hex_to_lab(h)).collect();
    let rgb_palette: Vec<Rgb<u8>> = hex_palette.iter().map(|&h| hex_to_rgb(h)).collect();

    // Convert the HI-RES image to float pixels for processing
    let mut hires_float_pixels: Vec<Srgb<f32>> = hires_img
        .pixels()
        .map(|p| {
            let [r, g, b, _] = p.0;
            Srgb::new(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
            )
        })
        .collect();

    let dither_strength = 1.0; // tune this
    println!("Processing {}x{} image with dithering (strength: {})...", hires_width, hires_height, dither_strength);

    // --- The main dithering loop ---
    let mut dithered_hires_image: RgbImage = RgbImage::new(hires_width, hires_height);
    
    let start_timer = std::time::Instant::now();

    for y in 0..hires_height {
        for x in 0..hires_width {
            let index = (y * hires_width + x) as usize;
            let original_srgb = hires_float_pixels[index];

            let original_lab: Lab = original_srgb.into_linear().into_color();
            let closest_index = find_closest_color(original_lab, &lab_palette);
            let final_rgb_pixel = rgb_palette[closest_index];

            dithered_hires_image.put_pixel(x, y, final_rgb_pixel);

            let [r, g, b] = final_rgb_pixel.0;
            let final_srgb = Srgb::new(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
            );

            let error_r = (original_srgb.red - final_srgb.red) * dither_strength;
            let error_g = (original_srgb.green - final_srgb.green) * dither_strength;
            let error_b = (original_srgb.blue - final_srgb.blue) * dither_strength;

            // Jarvis, Judice, and Ninke kernel
            let weights = [
                ((1, 0), 7.0 / 48.0), ((2, 0), 5.0 / 48.0),
                ((-2, 1), 3.0 / 48.0), ((-1, 1), 5.0 / 48.0), ((0, 1), 7.0 / 48.0), ((1, 1), 5.0 / 48.0), ((2, 1), 3.0 / 48.0),
                ((-2, 2), 1.0 / 48.0), ((-1, 2), 3.0 / 48.0), ((0, 2), 5.0 / 48.0), ((1, 2), 3.0 / 48.0), ((2, 2), 1.0 / 48.0),
            ];

            for &((dx, dy), weight) in &weights {
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0 && nx < hires_width as isize && ny < hires_height as isize {
                    let neighbor_index = (ny as u32 * hires_width + nx as u32) as usize;
                    let mut neighbor = hires_float_pixels[neighbor_index];
                    neighbor.red = (neighbor.red + error_r * weight).clamp(0.0, 1.0);
                    neighbor.green = (neighbor.green + error_g * weight).clamp(0.0, 1.0);
                    neighbor.blue = (neighbor.blue + error_b * weight).clamp(0.0, 1.0);
                    hires_float_pixels[neighbor_index] = neighbor;
                }
            }
        }
    }
    let elapsed = start_timer.elapsed();
    println!("Dithering completed in {:.2?}", elapsed);
    
    // ---  Downscale the dithered image back to original size ---
    println!("Downscaling dithered image back to {}x{} using Lanczos3...", width, height);
    let downscaled_image = imageops::resize(
        &dithered_hires_image,
        width,
        height,
        imageops::FilterType::Lanczos3,
    );


    
    downscaled_image.save("output.png").expect("Failed to save image");
    println!("Image processing complete. Saved as output.png");
}

// --- Helper functions ---

fn parse_hex(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
    (r, g, b)
}

fn hex_to_rgb(hex: &str) -> Rgb<u8> {
    let (r, g, b) = parse_hex(hex);
    Rgb([r, g, b])
}

fn hex_to_lab(hex: &str) -> Lab {
    let (r, g, b) = parse_hex(hex);
    let srgb = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    srgb.into_linear().into_color()
}

fn find_closest_color(pixel_lab: Lab, palette: &[Lab]) -> usize {
    palette
        .iter()
        .enumerate()
        .min_by(|(_, color_a), (_, color_b)| {
            let diff_a: f32 = ImprovedCiede2000::improved_difference(pixel_lab, **color_a);
            let diff_b: f32 = ImprovedCiede2000::improved_difference(pixel_lab, **color_b);
            diff_a.partial_cmp(&diff_b).unwrap()
        })
        .map(|(index, _)| index)
        .unwrap()
}