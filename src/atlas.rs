use std::collections::HashMap;
use std::path::Path;
use std::fmt::{Debug, Display};

use freetype::face::{Face, LoadFlag};
use freetype::Bitmap;
use image::{ImageBuffer, Rgb, GenericImage};

use super::{Glyph, Node, Rectangle};

/// An atlas containing glyphs of a given font.
pub struct FontAtlas {
    pub map: HashMap<char, Glyph>,
    pub buffers: Vec<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    pub width: u32,
    pub height: u32,
}

impl FontAtlas {
    /// Create a font atlas of given `atlas_size`.
    pub fn new(atlas_size: (u32, u32)) -> Self {
	Self {
	    map: HashMap::new(),
	    buffers: vec![ImageBuffer::new(atlas_size.0, atlas_size.1)],
	    width: atlas_size.0,
	    height: atlas_size.1,
	}
    }

    fn raw_bitmap_to_vec(bitmap: &Bitmap) -> Vec<u8> {
	let width = bitmap.width() as u32 / 3;
	let height = bitmap.rows() as u32;
	let pitch = bitmap.pitch();

	let mut vec_buffer = Vec::<u8>::with_capacity((width * height * 3) as usize);
	vec_buffer.resize((width * height * 3) as usize, 0);

	for y in 0..height as usize {
	    for x in 0..(width * 3) as usize {
		let src = y * pitch as usize + x;
		let dst = y * (width * 3) as usize + x;
		vec_buffer[dst] = bitmap.buffer()[src];
	    }
	}

	vec_buffer
    }

    /// Creates a font atlas from the given string and the given font face.
    pub fn from_str(glyphs: &str, font_face: &Face, atlas_size: (u32, u32)) -> Result<FontAtlas, FontAtlasError> {
	let mut atlas = FontAtlas::new(atlas_size);

	let mut node = Node::new(Rectangle::new(0, 0, atlas.width, atlas.height));

	let mut current_atlas_index = 0usize;

	for c in glyphs.chars() {
	    if let Err(_) = font_face.load_char(c as usize, LoadFlag::RENDER | LoadFlag::TARGET_LCD) {
		return Err(FontAtlasError::LoadError(c));
	    }

	    let glyph = font_face.glyph();
	    let raw_bitmap = glyph.bitmap();
	    let vec_buffer = FontAtlas::raw_bitmap_to_vec(&raw_bitmap);
	    let bitmap: ImageBuffer<Rgb<u8>, Vec<_>> = ImageBuffer::from_vec(
		raw_bitmap.width() as u32 / 3,
		raw_bitmap.rows() as u32,
		vec_buffer
	    ).unwrap();

	    let padding_left = 0;
	    let padding_right = 1;
	    let padding_top = 0;
	    let padding_bottom = 1;
	    let padding_h = padding_left + padding_right;
	    let padding_v = padding_top + padding_bottom;

	    let bitmap_rectangle = Rectangle::new(0, 0, bitmap.width() + padding_h, bitmap.height() + padding_v);

	    let inserted = node.insert(&bitmap_rectangle).or_else(|_| {
		let new_buffer = ImageBuffer::new(atlas.width, atlas.height);
		atlas.buffers.push(new_buffer);
		current_atlas_index += 1;

		node = Node::new(Rectangle::new(0, 0, atlas.width, atlas.height));
		return node.insert(&bitmap_rectangle).or_else(|_| {
		    return Err(FontAtlasError::InsertError(bitmap_rectangle))
		});
	    })?;

	    let inserted_without_padding = Rectangle::new(
		inserted.top + padding_top,
		inserted.left + padding_left,
		inserted.width - padding_h,
		inserted.height - padding_v
	    );

	    let g = Glyph {
		atlas_position: inserted_without_padding,
		atlas_index: current_atlas_index,
		metrics: glyph.into()
	    };
	    atlas.map.insert(c, g);

	    let mut atlas_view = atlas.buffers[current_atlas_index].sub_image(
		inserted_without_padding.left,
		inserted_without_padding.top,
		inserted_without_padding.width,
		inserted_without_padding.height
	    );
	    atlas_view.copy_from(&bitmap, 0, 0);
	}

	Ok(atlas)
    }
}

pub enum FontAtlasError{
    InsertError(Rectangle),
    LoadError(char)
}

impl Display for FontAtlasError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	match self {
	    FontAtlasError::InsertError(size) => write!(f, "Can't insert rectangle of size ({},{})", size.width, size.width),
	    FontAtlasError::LoadError(c) => write!(f, "Can't load character {}", c),
	}
    }
}

impl Debug for FontAtlasError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	match self {
	    FontAtlasError::InsertError(size) => write!(f, "Can't insert rectangle of size ({},{})", size.width, size.width),
	    FontAtlasError::LoadError(c) => write!(f, "Can't load character {}", c),
	}
    }
}

pub struct TextVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
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

	let uv_left = glyph.atlas_position.left as f32;
	let uv_right = (glyph.atlas_position.left + glyph.atlas_position.width) as f32;
	let uv_top = glyph.atlas_position.top as f32;
	let uv_bottom = (glyph.atlas_position.top + glyph.atlas_position.height) as f32;

	let v1 = TextVertex {
	    position: [left, bottom],
	    uv: [uv_left, uv_bottom]
	};

	let v2 = TextVertex {
	    position: [right, bottom],
	    uv: [uv_right, uv_bottom]
	};

	let v3 = TextVertex {
	    position: [left, top],
	    uv: [uv_left, uv_top]
	};

	let v4 = TextVertex {
	    position: [right, bottom],
	    uv: [uv_right, uv_bottom]
	};

	let v5 = TextVertex {
	    position: [right, top],
	    uv: [uv_right, uv_top]
	};

	let v6 = TextVertex {
	    position: [left, top],
	    uv: [uv_left, uv_top]
	};

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

pub fn generate_text_img(s: &str, font_atlas: &FontAtlas, save_path: &Path) {
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

	for x in 0..glyph.atlas_position.width {
	    for y in 0..glyph.atlas_position.height {
		let source_x = x + glyph.atlas_position.left;
		let source_y = y + glyph.atlas_position.top;

		let dest_x = x as i32 + left + advance + glyph.metrics.bearing_x;
		let dest_y = y as i32 + top - glyph.metrics.bearing_y;

		buffer.put_pixel(dest_x as u32, dest_y as u32, *font_atlas.buffers[glyph.atlas_index].get_pixel(source_x, source_y));
	    }
	}


	advance += glyph.metrics.advance;
    }

    buffer.save(save_path).unwrap();
}
