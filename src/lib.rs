use std::boxed::Box;
use std::fmt::{Debug, Display};
use std::path::Path;
use image::{ImageBuffer, Rgb};

use freetype::{face::{Face, LoadFlag}, LcdFilter, Library, Bitmap};

pub mod atlas;

/// A rectangle constrained by corner position and sizes
#[derive(Default, Debug, Copy, Clone)]
pub struct Rectangle {
    pub top: u32,
    pub left: u32,
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    /// Creates a new rectangle at (top, left) with (width, height) dimensions.
    pub fn new(top: u32, left: u32, width: u32, height: u32) -> Self {
	Self { top, left, width, height }
    }

    /// Returns true if the current rectangle fits in the `other` rectangle.
    pub fn fit_in(&self, other: &Rectangle) -> bool {
	other.width >= self.width && other.height >= self.height
    }

    /// Returns true if the two rectangles hase the same sizes.
    pub fn same_size(&self, other: &Rectangle) -> bool {
	return self.width == other.width && self.height == other.height
    }
}

/// A binary tree node containing rectangles.
#[derive(Debug)]
pub struct Node {
    pub rectangle: Rectangle,
    pub children: [Option<Box<Node>>; 2],
    pub occupied: bool
}

impl Node {
    /// Creates a node containing the given node.
    pub fn new(rectangle: Rectangle) -> Self {
	Self {
	    rectangle,
	    children: [None, None],
	    occupied: false
	}
   }

    /// Returns true if the given node is a leaf,
    pub fn is_leaf(&self) -> bool {
	self.children[0].is_none() && self.children[1].is_none()
    }

    /// Returns a result indicating if the given rectangle were sucessfully inserted in the tree.
    pub fn insert(&mut self, rectangle: &Rectangle) -> Result<Rectangle, NodeInsertError> {
	// If we are in a leaf
	if self.is_leaf() {
	    // If the node is already occupied, we can't insert the new rectangle
	    if self.occupied {
		return Err(NodeInsertError(rectangle.clone()));
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
	    return Err(NodeInsertError(rectangle.clone()));
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

#[derive(Debug)]
pub struct NodeInsertError(Rectangle);

impl Display for NodeInsertError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	write!(f, "Can't insert rectangle of size ({},{})", self.0.width, self.0.height)
    }
}

/// A struct representing a glyph in the font atlas.
#[derive(Debug)]
pub struct Glyph {
    pub metrics: GlyphMetrics,
    pub bitmap: ImageBuffer<Rgb<u8>, Vec<u8>>
}

impl Glyph {
    /// Creates a glyph from its metrics and its associated bitmap.
    pub fn new(metrics: GlyphMetrics, bitmap: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Self {
	Self {
	    metrics,
	    bitmap,
	}
    }
}

/// A struct representing various metrics about a glyph.
#[derive(Debug)]
pub struct GlyphMetrics {
    pub width: u32,
    pub height: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    pub advance: i32,

}

impl GlyphMetrics {
    /// Creates a GlyphMetrics from the given metrics values.
    pub fn new(width: u32, height: u32, bearing_x: i32, bearing_y: i32, advance: i32) -> Self {
	Self {
	    width,
	    height,
	    bearing_x,
	    bearing_y,
	    advance
	}
    }
}


pub struct FontLoader {
    ft_font_face: Face,
}

impl FontLoader {
    pub fn new<P: AsRef<Path>>(font_filepath: P) -> Result<FontLoader, FontLoaderError> {
	let library = if let Ok(library) = Library::init() {
	    library
	} else {
	    return Err(FontLoaderError::LibraryLoadError);
	};

	if let Err(_) = library.set_lcd_filter(LcdFilter::LcdFilterDefault) {
	    return Err(FontLoaderError::LcdFilterError);
	}

	let face = if let Ok(face) = library.new_face(font_filepath.as_ref(), 0) {
	    face
	} else {
	    let font_name = font_filepath.as_ref().file_name().unwrap().to_str().unwrap();
	    return Err(FontLoaderError::FontLoadError(String::from(font_name)));
	};

	Ok(FontLoader {
	    ft_font_face: face
	})
    }

    /// Loads a glyph from the associated font file.
    pub fn load_glyph(&self, c: char, dpi: u32, size: u32, load_flags: LoadFlag) -> Result<Glyph, FontLoaderError> {
	self.ft_font_face.set_char_size(0, size as isize, 0, dpi).unwrap();

	if let Err(e) = self.ft_font_face.load_char(c as usize, load_flags) {
	    return Err(FontLoaderError::GlyphLoadError(c));
	}

	let ft_glyph = self.ft_font_face.glyph();
	let raw_bitmap = ft_glyph.bitmap();

	let bitmap = self.convert_bitmap(&raw_bitmap, load_flags);

	let metrics = GlyphMetrics::new(
	    ft_glyph.metrics().width as u32 / 64,
	    ft_glyph.metrics().height as u32 / 64,
	    ft_glyph.metrics().horiBearingX as i32 / 64,
	    ft_glyph.metrics().horiBearingY as i32 / 64,
	    ft_glyph.metrics().horiAdvance as i32 / 64
	);


	Ok(Glyph::new(metrics, bitmap))
    }

    fn convert_bitmap(&self, bitmap: &Bitmap, load_flags: LoadFlag) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
	let (width, height, pitch) = match load_flags.contains(LoadFlag::TARGET_LCD) {
	    true => (bitmap.width() as u32 / 3, bitmap.rows() as u32, bitmap.pitch()),
	    false => (bitmap.width() as u32, bitmap.rows() as u32, bitmap.pitch()),
	};

	let pixel_count = (width * height) as usize;

	let mut vec_buffer = Vec::<u8>::with_capacity(pixel_count * 3);
	vec_buffer.resize(pixel_count * 3, 0);

	for y in 0..height as usize {
	    for x in 0..width as usize {
		match load_flags.contains(LoadFlag::TARGET_LCD) {
		    false => {
			let src = y * pitch as usize + x;
			let dst = y * (width * 3) as usize + x * 3;
			let gray = bitmap.buffer()[src];
			vec_buffer[dst] = gray;
			vec_buffer[dst + 1] = gray;
			vec_buffer[dst + 2] = gray;
		    },
		    true => {
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
}

pub enum FontLoaderError {
    LibraryLoadError,
    LcdFilterError,
    FontLoadError(String),
    GlyphLoadError(char)
}

impl Display for FontLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	match self {
	    FontLoaderError::LibraryLoadError => write!(f, "Failed to load freetype library !"),
	    FontLoaderError::LcdFilterError => write!(f, "Failed to set LCD filter"),
	    FontLoaderError::FontLoadError(s) => write!(f, "Failed to load font {}", s),
	    FontLoaderError::GlyphLoadError(c) => write!(f, "Can't load character {}", c),
	}
    }
}

impl Debug for FontLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
	match self {
	    FontLoaderError::LibraryLoadError => write!(f, "Failed to load freetype library !"),
	    FontLoaderError::LcdFilterError => write!(f, "Failed to set LCD filter"),
	    FontLoaderError::FontLoadError(s) => write!(f, "Failed to load font {}", s),
	    FontLoaderError::GlyphLoadError(c) => write!(f, "Can't load character {}", c),
	}
    }
}
