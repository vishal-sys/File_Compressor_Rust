use std::{
    env,
    fs::{self, File},
    io::{BufReader, BufWriter, copy},
    path::Path,
    process::Command,
    time::Instant,
};

use webp::Encoder;
use image::GenericImageView;
use flate2::write::GzEncoder;
use image::codecs::webp::WebPEncoder;
use flate2::Compression;
use image::{ImageFormat, DynamicImage};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{PngEncoder, CompressionType, FilterType};
use image::{ImageEncoder, ColorType}; // ✅ For .encode and ColorType

use image::{self,ImageBuffer};





fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: cargo run -- <file_path>");
        return;
    }

    let input_path = Path::new(&args[1]);

    if !input_path.exists() {
        eprintln!("File does not exist.");
        return;
    }

    let extension = input_path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    let start = Instant::now();

   match extension.as_str() {
    "png" => compress_png(&input_path),
    "jpg" | "jpeg" => compress_jpeg(&input_path),
    "pdf" => compress_pdf(&input_path),
    "txt" => compress_txt(&input_path),
    "webp" => compress_webp(&input_path), // ✅ Added WebP support
    _ => eprintln!("Unsupported file type."),
}


    println!("✅ Done in {:?}", start.elapsed());
}

fn compress_png(path: &Path) {
    let img = image::open(path).expect("Failed to open PNG image");
    let rgba8 = img.to_rgba8(); // Convert to RGBA8

    let output_name = format!(
        "{}_compressed.png",
        path.file_stem().unwrap().to_string_lossy()
    );
    let output_path = path.with_file_name(output_name);

    let file = File::create(&output_path).expect("Failed to create output PNG file");
    let writer = BufWriter::new(file);

    let encoder = PngEncoder::new_with_quality(writer, CompressionType::Best, FilterType::Adaptive);

    encoder
        .write_image(
            &rgba8,
            rgba8.width(),
            rgba8.height(),
            image::ColorType::Rgba8.into(),
        )
        .expect("Failed to encode PNG");

    print_size(path, &output_path);
}



fn compress_jpeg(path: &Path) {
    let img = image::open(path).expect("Failed to open JPEG image");

    let output_name = format!(
        "{}_compressed.jpg",
        path.file_stem().unwrap().to_string_lossy()
    );
    let output_path = path.with_file_name(output_name);

    let file = File::create(&output_path).expect("Failed to create output JPEG file");
    let writer = BufWriter::new(file);

    let mut encoder = JpegEncoder::new_with_quality(writer, 70); // adjust quality (50-90)
    encoder
        .encode_image(&img)
        .expect("Failed to encode JPEG");

    print_size(path, &output_path);
}

fn compress_txt(path: &Path) {
    let file = File::open(path).expect("Failed to open text file");
    let mut input = BufReader::new(file);

    let output_name = format!(
        "{}.gz",
        path.file_name().unwrap().to_string_lossy()
    );
    let output_path = path.with_file_name(output_name);

    let output_file = File::create(&output_path).expect("Failed to create .gz file");
    let mut encoder = GzEncoder::new(output_file, Compression::default());
    copy(&mut input, &mut encoder).expect("Failed to compress .txt");
    let output = encoder.finish().expect("Failed to finalize .gz");

    print_size(path, &output_path);
}

fn compress_pdf(path: &Path) {
    let output_name = format!(
        "{}_compressed.pdf",
        path.file_stem().unwrap().to_string_lossy()
    );
    let output_path = path.with_file_name(&output_name);

    let status = Command::new("gs")
        .args([
            "-sDEVICE=pdfwrite",
            "-dCompatibilityLevel=1.4",
            "-dPDFSETTINGS=/ebook", // You can change this to /screen, /printer etc.
            "-dNOPAUSE",
            "-dQUIET",
            "-dBATCH",
            &format!("-sOutputFile={}", output_path.to_string_lossy()), // ✅ Correct way to pass output path
            path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run Ghostscript");

    if status.success() {
        print_size(path, &output_path);
    } else {
        eprintln!("❌ Ghostscript compression failed.");
    }
}

fn print_size(original: &Path, compressed: &Path) {
    let original_size = std::fs::metadata(original).unwrap().len();
    let compressed_size = std::fs::metadata(compressed).unwrap().len();
    println!("📦 Original:   {} bytes", original_size);
    println!("📉 Compressed: {} bytes", compressed_size);
}


fn compress_webp(path: &Path) {
    let img = image::open(path).expect("Failed to open WebP image");
    let rgb_image = img.to_rgb8();

    // ✅ Create WebP encoder from RGB
    let encoder = Encoder::from_rgb(&rgb_image, rgb_image.width(), rgb_image.height());

    // ✅ Compress with 75.0% quality
    let webp_data = encoder.encode(75.0); 

    let output_name = format!(
        "{}_compressed.webp",
        path.file_stem().unwrap().to_string_lossy()
    );
    let output_path = path.with_file_name(&output_name);

    std::fs::write(&output_path, &*webp_data).expect("Failed to write compressed WebP");

    print_size(path, &output_path);
}
