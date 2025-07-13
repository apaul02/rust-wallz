use image::{GenericImageView, Rgb, RgbImage};
use palette::{
    color_difference::Ciede2000,
    IntoColor, Lab, Srgb,
};

fn main() {
    let img = image::open("Makima.jpg")
        .expect("Failed to open image.png. Make sure the file is in the correct directory.");

    let (width, height) = img.dimensions();
    println!("Image dimensions: {}x{}", width, height);

    let rgb_img = img.to_rgb8();
    println!("Converted image to RGB8 format.");

let hex_palette = [
        // Base Colors
        "#191724", // Base
        "#1f1d2e", // Surface
        "#26233a", // Overlay
        "#393552", // Muted
        "#6e6a86", // Subtle
        "#e0def4", // Text
        // Highlight Colors
        "#eb6f92", // Rose
        "#f6c177", // Gold
        "#ebbcba", // Red
        "#31748f", // Pine
        "#9ccfd8", // Foam
        "#c4a7e7", // Iris
        // Accent Colors
        "#e5e0ce", // Highlight Low
        "#403d52", // Highlight Med
        "#524f67", // Highlight High
    ];
    let lab_palette: Vec<Lab> = hex_palette.iter().map(|&h| hex_to_lab(h)).collect();
    println!("Created Lab color palette for comparison.");

    
    let rgb_u8_palette: Vec<Rgb<u8>> = hex_palette.iter().map(|h| hex_to_rgb_u8(h)).collect();
    println!("Created Rgb<u8> pixel palette for output.");

    
    let mut output_image: RgbImage = RgbImage::new(width, height);
    println!("\nProcessing pixels...");

    
    for y in 0..height {
        for x in 0..width {
            let original_pixel = rgb_img.get_pixel(x, y);
            let [r, g, b] = original_pixel.0;

        
            let srgb_pixel = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
            let lab_pixel: Lab = srgb_pixel.into_linear().into_color();

            let closest_color_index = find_closest_palette_color(lab_pixel, &lab_palette);
            let final_pixel = rgb_u8_palette[closest_color_index];
            output_image.put_pixel(x, y, final_pixel);
        }
    }

    output_image.save("final.png").expect("Failed to save image");
    println!("\nImage processing complete. Saved as final.png");
}

fn find_closest_palette_color(pixel_lab: Lab, palette: &[Lab]) -> usize {
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

fn parse_hex(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
    (r, g, b)
}

fn hex_to_lab(hex: &str) -> Lab {
    let (r, g, b) = parse_hex(hex);
    let srgb = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    srgb.into_linear().into_color()
}
fn hex_to_rgb_u8(hex: &str) -> Rgb<u8> {
    let (r, g, b) = parse_hex(hex);
    Rgb([r, g, b])
}
