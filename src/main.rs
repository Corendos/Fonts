use std::ffi::OsStr;
use std::io::{Write, Read};

use freetype::{Library, LcdFilter};

use font::{atlas::{FontAtlas}, Glyph};

const FONT_DIRECTORY: &str = "/home/corendos/dev/rust/font/resources/fonts";
const FONT_SIZE: isize = 30 * 64;
const GLYPHS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789\\|/?.>,<`!@#$%^&*()_-=+[]{};:'\" é";

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

fn _test_serialize_deserialize(c: &Glyph) {
    unsafe {
	let buffer = std::slice::from_raw_parts(c as *const Glyph as *const u8, std::mem::size_of_val(c));

	let mut f = std::fs::File::create("test.data").unwrap();

	f.write(buffer).unwrap();
    }

    let mut read_glyph: Glyph = unsafe { std::mem::zeroed() };
    unsafe {
	let mut buffer = std::slice::from_raw_parts_mut(&mut read_glyph as *mut Glyph as *mut u8, std::mem::size_of::<Glyph>());

	let mut f = std::fs::File::open("test.data").unwrap();

	f.read(&mut buffer).unwrap();
    }
}

fn main() {
    let fonts = get_fonts();
    let library = Library::init().expect("Failed to init freetype library");

    library.set_lcd_filter(LcdFilter::LcdFilterDefault).unwrap();

    let font_face = library.new_face(&fonts[7], 0).expect("Failed to load font");

    font_face.set_char_size(0, FONT_SIZE, 0, 72).unwrap();

    let font_atlas = FontAtlas::from_str(GLYPHS, &font_face, (256, 256)).unwrap();

    let _text = "test the text generator";

    for (i, buffer) in font_atlas.buffers.iter().enumerate() {
	buffer.save(format!("output/glyphs_{}.png", i)).unwrap();
    }
    //generate_text(&text, &font_atlas, Path::new(&"text.png"));
/*
    let c = font_atlas.map.get(&'a').unwrap();
     */
}
