use std::{cmp, collections::BTreeSet, fmt::Display};

use crate::text::{Position, TextNode};

/// A heap-allocated buffer designed for efficient insertion, deletion and edit operations on containing [TextNode] values.
#[derive(Debug)]
pub(crate) struct TextBuffer {
    // The containing nodes for this buffer making up the entire text it has governance over.
    nodes: BTreeSet<TextNode>,

    // The virtual boundary this buffer takes up. For example, there could be multiple buffers pooled for a single document
    // with each taking up a certain amount of space depending on the total amount of containing nodes.
    boundary: Boundary,
}

impl TextBuffer {
    /// Constructs a new [TextBuffer] from the provided string value.
    pub fn new() -> Self {
        Self {
            nodes: BTreeSet::new(),
            boundary: Boundary::default(),
        }
    }

    /// Pushes a [TextNode] into this buffer, adjusting the boundary if needed.
    pub fn push(&mut self, node: TextNode) {
        self.boundary = self.boundary.union(node.dimensions);
        self.nodes.insert(node);
    }


}

#[doc(hidden)]
impl Display for TextBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for node in self.nodes.iter() {
            for line in node.lines() {
                let _ = f.write_str(&line);
            }
        }
        Ok(())
    }
}

/// A virtual bounding box comprised of a position, denoting its origin, as well as a width & height value used to calculate its span.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Boundary {
    pub(crate) origin: Position,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl Boundary {
    /// Compares `self` with the boundary passed in and returns a new boundary representing the intersection of the two, that being the smallest
    /// set of values between the two.
    pub fn intersect(self, Self { width, height, .. }: Self) -> Self {
        Self {
            origin: self.origin,
            width: cmp::min(self.width, width),
            height: cmp::min(self.height, height),
        }
    }

    /// Compares `self` with the boundary passed in and returns a new boundary representing the union of the two, that being the largest pair of
    /// values between the two.
    pub fn union(self, Self { width, height, .. }: Self) -> Self {
        Self {
            origin: self.origin,
            width: cmp::max(self.width, width),
            height: cmp::max(self.height, height),
        }
    }

    /// Compares the passed-in [Boundary] with `this` returning true if `this` boundary completely covers the opposing boundary.
    pub fn contains(self, Self { width, height, .. }: Self) -> bool {
        width <= self.width && height <= self.height
    }

    /// The start, or top-left, position where this boundary originates from.
    pub fn origin(&self) -> &Position {
        &self.origin
    }
}
