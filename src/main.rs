use image::{GenericImageView, Rgb, RgbImage};
use palette::{
    color_difference::Ciede2000,
    IntoColor, Lab, Srgb,
};

fn main() {
    let img = image::open("image4.png").expect("Failed to open image");
    let (width, height) = img.dimensions();
    println!("Image dimensions: {}x{}", width, height);

    let hex_palette = [
        "#303446", "#292c3c", "#232634", "#626880", "#51576d", "#414559", "#c6d0f5",
        "#b5bfe2", "#a5adce", "#949cbb", "#838ba7", "#737994", "#f2d5cf", "#eebebe",
        "#e78284", "#ef9f76", "#e5c890", "#a6d189", "#81c8be", "#99d1db", "#8caaee",
        "#c6a0f6", "#f4b8e4",
    ];

    let lab_palette: Vec<Lab> = hex_palette.iter().map(|&h| hex_to_lab(h)).collect();
    let rgb_palette: Vec<Rgb<u8>> = hex_palette.iter().map(|&h| hex_to_rgb(h)).collect();

    let mut float_pixels: Vec<Srgb<f32>> = img
        .to_rgb32f()
        .chunks_exact(3)
        .map(|p| Srgb::new(p[0], p[1], p[2]))
        .collect();
    
    let dither_strength = 0.7;

    let mut output_image: RgbImage = RgbImage::new(width, height);
    println!("Processing image with Hybrid Dithering (strength: {})...", dither_strength);

    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) as usize;
            let original_srgb = float_pixels[index];
            
            let original_lab: Lab = original_srgb.into_linear().into_color();
            let closest_index = find_closest_color(original_lab, &lab_palette);
            let final_rgb_pixel = rgb_palette[closest_index];

            output_image.put_pixel(x, y, final_rgb_pixel);

            let [r, g, b] = final_rgb_pixel.0;
            let final_srgb = Srgb::new(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
            );
            
            let error_r = (original_srgb.red - final_srgb.red) * dither_strength;
            let error_g = (original_srgb.green - final_srgb.green) * dither_strength;
            let error_b = (original_srgb.blue - final_srgb.blue) * dither_strength;

            let weights = [
                ((1, 0), 7.0/48.0), ((2, 0), 5.0/48.0),
                ((-2, 1), 3.0/48.0), ((-1, 1), 5.0/48.0), ((0, 1), 7.0/48.0), ((1, 1), 5.0/48.0), ((2, 1), 3.0/48.0),
                ((-2, 2), 1.0/48.0), ((-1, 2), 3.0/48.0), ((0, 2), 5.0/48.0), ((1, 2), 3.0/48.0), ((2, 2), 1.0/48.0),
            ]; 

            for &((dx, dy), weight) in &weights {
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0 && nx < width as isize && ny < height as isize {
                    let neighbor_index = (ny as u32 * width + nx as u32) as usize;
                    let mut neighbor = float_pixels[neighbor_index];
                    neighbor.red = (neighbor.red + error_r * weight).clamp(0.0, 1.0);
                    neighbor.green = (neighbor.green + error_g * weight).clamp(0.0, 1.0);
                    neighbor.blue = (neighbor.blue + error_b * weight).clamp(0.0, 1.0);
                    float_pixels[neighbor_index] = neighbor;
                }
            }
        }
    }
    
    output_image.save("output.png").expect("Failed to save image");
    println!("Image processing complete. Saved as output.png");
}


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
        .min_by(|a, b| {
            let color_a = *a.1;
            let color_b = *b.1;
            let diff_a = Ciede2000::difference(pixel_lab, color_a);
            let diff_b = Ciede2000::difference(pixel_lab, color_b);
            diff_a.partial_cmp(&diff_b).unwrap()
        })
        .map(|(index, _)| index)
        .unwrap()
}
