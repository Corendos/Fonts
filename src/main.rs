use std::ffi::OsStr;

use freetype::{face::LoadFlag};

use font::{atlas::{AtlasGenerator, AtlasGeneratorOption, Padding, AtlasLoadMode}, FontLoader};

const FONT_DIRECTORY: &str = "/home/corendos/dev/rust/font/resources/fonts";
const FONT_SIZE: u32 = 200 * 64;

fn get_fonts() -> Vec<String> {

    let mut font_paths = Vec::<String>::new();

    std::fs::read_dir(FONT_DIRECTORY).unwrap().for_each(|dir| {
	if let Ok(dir_entry) = dir {
	    if dir_entry.path().as_path().extension() == Some(OsStr::new("ttf")) {
		font_paths.push(String::from(dir_entry.path().to_str().unwrap()));
	    }
	}
    });

    font_paths
}

fn main() {
    let fonts = get_fonts();

    let generator = AtlasGenerator::new(
	&fonts[1],
	AtlasGeneratorOption::new(2048, 2048, 72, Padding::new(1, 1, 1, 1)),
	AtlasLoadMode::LCD
    ).unwrap();

    let font_atlas = generator.generate(FONT_SIZE).unwrap();

    let font_loader = FontLoader::new(&fonts[1]).unwrap();

    let start = std::time::Instant::now();
    font_loader.load_glyph('W', 72, 300 * 64, LoadFlag::RENDER | LoadFlag::TARGET_LCD).unwrap();
    let end = std::time::Instant::now();

    println!("Took {} ns", (end - start).as_nanos());

    font_atlas.buffer.save("output/glyphs.png").unwrap();

    /*
    let text = "Hello dlrow !";

    let buffer = generate_buffers_from_text(&text, &font_atlas, 0, 0);
    generate_text_img(text, &font_atlas, "output/text.png");
    println!("{:#?}", buffer);
     */
}
