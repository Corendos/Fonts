use std::boxed::Box;
use std::fmt::{Debug};
use freetype::{GlyphSlot};

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

/// A struct representing a glyph in the font atlas.
#[derive(Debug)]
pub struct Glyph {
    pub metrics: GlyphMetrics,
    pub atlas_position: Rectangle,
    pub atlas_index: usize
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
