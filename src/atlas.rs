use std::collections::HashMap;
use std::path::Path;
use std::fmt::{Debug, Display};

use freetype::face::{Face, LoadFlag};
use freetype::{Bitmap, Library, LcdFilter};
use image::{ImageBuffer, Rgb, GenericImage};

use super::{Glyph, GlyphMetrics, Node, Rectangle, NodeInsertError};

const GLYPHS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789\\|/?.>,<`!@#$%^&*()_-=+[]{};:'\" ";

/// An atlas containing glyphs of a given font.
pub struct FontAtlas {
    pub map: HashMap<char, FontAtlasEntry>,
    pub buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

impl FontAtlas {
    /// Create a font atlas of given `atlas_size`.
    pub fn new(atlas_size: (u32, u32)) -> Self {
	Self {
	    map: HashMap::new(),
	    buffer: ImageBuffer::new(atlas_size.0, atlas_size.1),
	    width: atlas_size.0,
	    height: atlas_size.1,
	}
    }


}

/// An entry to the font atlas. It contains the glyph metrics and its position in the atlas.
pub struct FontAtlasEntry {
    metrics: GlyphMetrics,
    position: Rectangle
}

impl FontAtlasEntry {
    /// Creates an entry from the glyph metrics and position in an atlas.
    pub fn new(position: Rectangle, metrics: GlyphMetrics) -> Self {
	Self {
	    position,
	    metrics
	}
    }
}

// @Temporary
#[derive(Debug)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

impl TextVertex {
    pub fn new(x: f32, y: f32, u: f32, v: f32) -> TextVertex {
	TextVertex {
	    position: [x, y],
	    uv: [u, v],
	}
    }
}

pub fn generate_buffers_from_text(text: &str, font_atlas: &FontAtlas, x: i32, y: i32) -> Vec<TextVertex> {
    let mut advance = 0i32;

    let mut vertex_buffer = Vec::<TextVertex>::with_capacity(text.len() * 4 * 6);

    for c in text.chars() {
	let glyph = font_atlas.map.get(&c).unwrap_or_else(|| {
	    font_atlas.map.get(&' ').unwrap()
	});

	let left = (x + advance + glyph.metrics.bearing_x) as f32;
	let right = (x + advance + glyph.metrics.bearing_x + glyph.metrics.width as i32) as f32;
	let top = (y + glyph.metrics.bearing_y) as f32;
	let bottom = (y + glyph.metrics.bearing_y - glyph.metrics.height as i32) as f32;

	let uv_left = glyph.position.left as f32 / font_atlas.width as f32;
	let uv_right = (glyph.position.left + glyph.position.width) as f32 / font_atlas.width as f32;
	let uv_top = (font_atlas.height -  glyph.position.top) as f32 / font_atlas.height as f32;
	let uv_bottom = (font_atlas.height - (glyph.position.top + glyph.position.height)) as f32 / font_atlas.height as f32;

	let v1 = TextVertex::new(left, bottom, uv_left, uv_bottom);
	let v2 = TextVertex::new(right, bottom, uv_right, uv_bottom);
	let v3 = TextVertex::new(left, top, uv_left, uv_top);
	let v4 = TextVertex::new(right, bottom, uv_right, uv_bottom);
	let v5 = TextVertex::new(right, top, uv_right, uv_top);
	let v6 = TextVertex::new(left, top, uv_left, uv_top);

	vertex_buffer.push(v1);
	vertex_buffer.push(v2);
	vertex_buffer.push(v3);
	vertex_buffer.push(v4);
	vertex_buffer.push(v5);
	vertex_buffer.push(v6);

	advance += glyph.metrics.advance;
    }

    vertex_buffer
}

pub fn generate_text_img<P>(s: &str, font_atlas: &FontAtlas, save_path: P) where P: AsRef<Path> {
    let mut advance = 0i32;
    let mut top = 0i32;
    let mut left = 0i32;
    let mut right = 0i32;
    let mut bottom = 0i32;

    for c in s.chars() {
	let glyph = font_atlas.map.get(&c).unwrap();

	top = std::cmp::max(top, glyph.metrics.bearing_y);
	bottom = std::cmp::max(bottom, glyph.metrics.height as i32 - glyph.metrics.bearing_y);
	left = std::cmp::max(left, -(advance + glyph.metrics.bearing_x));
	right = std::cmp::max(right, advance + glyph.metrics.bearing_x + glyph.metrics.width as i32);

	advance += glyph.metrics.advance;
    }

    let buffer_width = right + left + 1;
    let buffer_height = top + bottom + 1;

    let mut buffer: ImageBuffer<Rgb<u8>, _> = ImageBuffer::new(buffer_width as u32, buffer_height as u32);

    advance = 0;
    for c in s.chars() {
	let glyph = font_atlas.map.get(&c).unwrap();

	for x in 0..glyph.position.width {
	    for y in 0..glyph.position.height {
		let source_x = x + glyph.position.left;
		let source_y = y + glyph.position.top;

		let dest_x = x as i32 + left + advance + glyph.metrics.bearing_x;
		let dest_y = y as i32 + top - glyph.metrics.bearing_y;

		buffer.put_pixel(dest_x as u32, dest_y as u32, *font_atlas.buffer.get_pixel(source_x, source_y));
	    }
	}


	advance += glyph.metrics.advance;
    }

    buffer.save(save_path).unwrap();
}

/// A struct representing a padding area around a rectangle.
#[allow(dead_code)]
pub struct Padding {
    left: u32,
    right: u32,
    top: u32,
    bottom: u32,
    horizontal: u32,
    vertical: u32,
}

impl Padding {
    /// Creates a Padding object from the padding on all sides
    pub fn new(left: u32, right: u32, top: u32, bottom: u32) -> Self {
	Self {
	    left, right, top, bottom,
	    horizontal: left + right, vertical: top + bottom
	}
    }
}

/// An enum telling the AtlasGenerator how to load the glyphs.
pub enum AtlasLoadMode {
    Gray,
    LCD,
}

impl AtlasLoadMode {
    pub fn default() -> AtlasLoadMode {
	AtlasLoadMode::LCD
    }
}

/// A struct representing the AtlasGenerator options.
pub struct AtlasGeneratorOption {
    pub dpi: u32,
    pub size: (u32, u32),
    pub padding: Padding,
}

impl AtlasGeneratorOption {
    /// Creates an AtlasGeneratorOption object from its components.
    pub fn new(width: u32, height: u32, dpi: u32, padding: Padding) -> Self {
	Self {
	    dpi,
	    size: (width, height),
	    padding,
	}
    }
}

/// A struct representing a FontAtlas generator
pub struct AtlasGenerator {
    ft_font_face: Face,
    load_mode: AtlasLoadMode,
    options: AtlasGeneratorOption
}

impl AtlasGenerator {
    /// Creates a generator from the given font filepath, options and load mode.
    pub fn new<P>(font_filepath: P, options: AtlasGeneratorOption, load_mode: AtlasLoadMode) -> AtlasGenerator where P: AsRef<Path> {
	let library = Library::init().expect("Failed to init freetype library");

	library.set_lcd_filter(LcdFilter::LcdFilterDefault).expect("Failed to set LCD Filter");

	let face = library.new_face(font_filepath.as_ref(), 0).expect("Failed to load font");

	AtlasGenerator {
	    ft_font_face: face,
	    load_mode,
	    options,
	}
    }

    /// Generate an atlas with the associated font of size `size`.
    pub fn generate(&self, size: u32) -> Result<FontAtlas, AtlasGeneratorError>{
	self.ft_font_face.set_char_size(0, size as isize, 0, self.options.dpi).unwrap();

	let mut atlas = FontAtlas::new(self.options.size);

	let mut node = Node::new(Rectangle::new(0, 0, atlas.width, atlas.height));

	for c in GLYPHS.chars() {
	    let load_flags = match self.load_mode {
		AtlasLoadMode::Gray => LoadFlag::RENDER,
		AtlasLoadMode::LCD => LoadFlag::RENDER | LoadFlag::TARGET_LCD
	    };

	    let glyph = self.load_glyph(c, load_flags)?;

	    let bitmap_rectangle = Rectangle::new(
		0,
		0,
		glyph.bitmap.width() + self.options.padding.horizontal,
		glyph.bitmap.height() + self.options.padding.vertical
	    );

	    let inserted = node.insert(&bitmap_rectangle)?;

	    let inserted_without_padding = Rectangle::new(
		inserted.top + self.options.padding.top,
		inserted.left + self.options.padding.left,
		inserted.width - self.options.padding.horizontal,
		inserted.height - self.options.padding.vertical
	    );

	    let entry = FontAtlasEntry::new(inserted_without_padding, glyph.metrics);

	    atlas.map.insert(c, entry);

	    let mut atlas_view = atlas.buffer.sub_image(
		inserted_without_padding.left,
		inserted_without_padding.top,
		inserted_without_padding.width,
		inserted_without_padding.height
	    );
	    atlas_view.copy_from(&glyph.bitmap, 0, 0);
	}

	Ok(atlas)
    }

    fn convert_bitmap(&self, bitmap: &Bitmap) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
	let (width, height, pitch) = match self.load_mode {
	    AtlasLoadMode::Gray => (bitmap.width() as u32, bitmap.rows() as u32, bitmap.pitch()),
	    AtlasLoadMode::LCD => (bitmap.width() as u32 / 3, bitmap.rows() as u32, bitmap.pitch()),
	};

	let pixel_count = (width * height) as usize;

	let mut vec_buffer = Vec::<u8>::with_capacity(pixel_count * 3);
	vec_buffer.resize(pixel_count * 3, 0);

	for y in 0..height as usize {
	    for x in 0..width as usize {
		match self.load_mode {
		    AtlasLoadMode::Gray => {
			let src = y * pitch as usize + x;
			let dst = y * (width * 3) as usize + x * 3;
			let gray = bitmap.buffer()[src];
			vec_buffer[dst] = gray;
			vec_buffer[dst + 1] = gray;
			vec_buffer[dst + 2] = gray;
		    },
		    AtlasLoadMode::LCD => {
			let src = y * pitch as usize + x * 3;
			let dst = y * (width * 3) as usize + x * 3;

			let r = bitmap.buffer()[src];
			let g = bitmap.buffer()[src + 1];
			let b = bitmap.buffer()[src + 2];
			vec_buffer[dst] = r;
			vec_buffer[dst + 1] = g;
			vec_buffer[dst + 2] = b;
		    },
		}

	    }
	}

	ImageBuffer::from_vec(width, height, vec_buffer).unwrap()
    }

    /// Loads a glyph from the associated font file.
    pub fn load_glyph(&self, c: char, load_flags: LoadFlag) -> Result<Glyph, AtlasGeneratorError> {
	if let Err(_) = self.ft_font_face.load_char(c as usize, load_flags) {
	    return Err(AtlasGeneratorError::LoadError(c));
	}

	let ft_glyph = self.ft_font_face.glyph();
	let raw_bitmap = ft_glyph.bitmap();

	let bitmap = self.convert_bitmap(&raw_bitmap);


	let metrics = GlyphMetrics::new(
	    ft_glyph.metrics().width as u32 / 64,
	    ft_glyph.metrics().height as u32 / 64,
	    ft_glyph.metrics().horiBearingX as i32 / 64,
	    ft_glyph.metrics().horiBearingY as i32 / 64,
	    ft_glyph.metrics().horiAdvance as i32 / 64
	);


	Ok(Glyph::new(metrics, bitmap))
    }
}

/// An enum representing all the error that could happen using the generator.
pub enum AtlasGeneratorError{
    InsertError(NodeInsertError),
    LoadError(char)
}

impl From<NodeInsertError> for AtlasGeneratorError {
    fn from(e: NodeInsertError) -> Self {
	AtlasGeneratorError::InsertError(e)
    }
}

impl Display for AtlasGeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	match self {
	    AtlasGeneratorError::InsertError(node_error) => write!(f, "{}", node_error),
	    AtlasGeneratorError::LoadError(c) => write!(f, "Can't load character {}", c),
	}
    }
}

impl Debug for AtlasGeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	match self {
	    AtlasGeneratorError::InsertError(node_error) => write!(f, "{}", node_error),
	    AtlasGeneratorError::LoadError(c) => write!(f, "Can't load character {}", c),
	}
    }
}
