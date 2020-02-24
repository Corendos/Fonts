use std::path::Path;
use std::boxed::Box;
use std::collections::HashMap;
use std::fmt::{Debug, Display};

use freetype::face::{Face, LoadFlag};
use freetype::{GlyphSlot, Bitmap};
use image::{ImageBuffer, Rgb, GenericImage};


#[derive(Default, Debug, Copy, Clone)]
pub struct Rectangle {
    pub top: u32,
    pub left: u32,
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    pub fn new(top: u32, left: u32, width: u32, height: u32) -> Self {
	Self { top, left, width, height }
    }

    pub fn fit_in(&self, other: &Rectangle) -> bool {
	other.width >= self.width && other.height >= self.height
    }

    pub fn same_size(&self, other: &Rectangle) -> bool {
	return self.width == other.width && self.height == other.height
    }

    pub fn merge_max(&self, other: &Rectangle) -> Rectangle {
	let new_left = if other.left < self.left {
	    other.left
	} else {
	    self.left
	};

	let other_right = other.left + other.width;
	let self_right = self.left + self.width;
	let new_right = std::cmp::max(other_right, self_right);

	let new_width = new_right - new_left;

	let new_top = if other.top < self.top {
	    other.top
	} else {
	    self.top
	};

	let other_bottom = other.top + other.height;
	let self_bottom = self.top + self.height;
	let new_bottom = std::cmp::max(other_bottom, self_bottom);

	let new_height = new_bottom - new_top;

	Rectangle::new(new_left, new_top, new_width, new_height)
    }
}

#[derive(Debug)]
pub struct Node {
    pub rectangle: Rectangle,
    pub children: [Option<Box<Node>>; 2],
    pub occupied: bool
}

impl Node {
    pub fn new(rectangle: Rectangle) -> Self {
	Self {
	    rectangle,
	    children: [None, None],
	    occupied: false
	}
    }

    pub fn is_leaf(&self) -> bool {
	self.children[0].is_none() && self.children[1].is_none()
    }

    pub fn insert(&mut self, rectangle: &Rectangle) -> Result<Rectangle, ()> {
	// If we are in a leaf
	if self.is_leaf() {
	    // If the node is already occupied, we can't insert the new rectangle
	    if self.occupied {
		return Err(());
	    }

	    // If the rectangle fit
	    if rectangle.fit_in(&self.rectangle) {
		// If it fits perfectly
		if rectangle.same_size(&self.rectangle) {
		    self.occupied = true;
		    return Ok(self.rectangle.clone());
		}
		// Otherwise
		let delta_width = self.rectangle.width - rectangle.width;
		let delta_height = self.rectangle.height - rectangle.height;

		if delta_width > delta_height {
		    self.children[0] = Some(
			Box::new(
			    Node::new(
				Rectangle::new(
				    self.rectangle.top, self.rectangle.left,
				    rectangle.width, self.rectangle.height)
			    )));
		    self.children[1] = Some(
			Box::new(
			    Node::new(
				Rectangle::new(
				    self.rectangle.top, self.rectangle.left + rectangle.width,
				    self.rectangle.width - rectangle.width, self.rectangle.height)
			    )));
		} else {
		    self.children[0] = Some(
			Box::new(
			    Node::new(
				Rectangle::new(
				    self.rectangle.top, self.rectangle.left,
				    self.rectangle.width, rectangle.height)
			    )));
		    self.children[1] = Some(
			Box::new(
			    Node::new(
				Rectangle::new(
				    self.rectangle.top + rectangle.height, self.rectangle.left,
				    self.rectangle.width, self.rectangle.height - rectangle.height)
			    )));
		}

		return self.children[0].as_mut().unwrap().insert(rectangle);
	    }

	    // The rectangle does not fit
	    return Err(());
	} else {    // We are not in a leaf
	    // We try to insert it in the first children
	    match self.children[0].as_mut().unwrap().insert(rectangle) {
		Ok(rect) => Ok(rect),
		Err(_) => {
		    self.children[1].as_mut().unwrap().insert(rectangle)
		}
	    }
	}
    }
}

impl From<&GlyphSlot> for GlyphMetrics {
    fn from(glyph_slot: &GlyphSlot) -> GlyphMetrics {
	GlyphMetrics {
	    width: glyph_slot.metrics().width as u32 / 64,
	    height: glyph_slot.metrics().height as u32 / 64,
	    bearing_x: glyph_slot.metrics().horiBearingX as i32 / 64,
	    bearing_y: glyph_slot.metrics().horiBearingY as i32 / 64,
	    advance: glyph_slot.metrics().horiAdvance as i32 / 64
	}
    }
}


#[derive(Debug)]
pub struct Glyph {
    pub metrics: GlyphMetrics,
    pub atlas_position: Rectangle,
    pub atlas_index: usize
}

#[derive(Debug)]
pub struct GlyphMetrics {
    pub width: u32,
    pub height: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    pub advance: i32,

}

pub struct FontAtlas {
    pub map: HashMap<char, Glyph>,
    pub buffers: Vec<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    pub width: u32,
    pub height: u32,
}

impl FontAtlas {
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

	    let padding_left = 5;
	    let padding_right = 5;
	    let padding_top = 5;
	    let padding_bottom = 5;
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

	    {
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


pub fn generate_text(s: &str, font_atlas: &FontAtlas, save_path: &Path) {
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

    let buffer_width = right + left;
    let buffer_height = top + bottom;

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
