use std::{
    fmt::Debug,
    hash::Hash,
    ops::{Add, AddAssign, RangeInclusive},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{buffer::Boundary, cursor::Cursor};

/// A value used to uniquely identify a node. The current default provider generates new ids statically, incrementing by 1 on each call, in order to avoid collision. There is no api in place,
/// as of yet, to allow for user-defined providers however, this may change in the future.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NodeId(usize);

impl NodeId {
    pub fn new() -> Self {
        static ID_PROVIDER: AtomicUsize = AtomicUsize::new(1);
        Self(ID_PROVIDER.fetch_add(1, Ordering::SeqCst))
    }
}

/// The smallest unit of representation that text may take the form of. A text node can be either a single character or a block
/// of characters spanning multiple lines.
#[derive(PartialEq, Eq, Ord)]
pub struct TextNode {
    id: NodeId,
    code_points: Vec<u16>,
    line_endings: Vec<TextRange>,
    pub(crate) dimensions: Boundary,
}

impl TextNode {
    /// The standardized, and currently recommended, approach for constructing a new node. The current expectations are
    /// the following:
    /// - The passed-in text is delimitered with a [Newline](https://en.wikipedia.org/wiki/Newline) character sequence either in
    /// `CRLF` (Windows-style) or `LF` (Unix-like) form.
    pub fn new_delimitered(text: impl AsRef<str>) -> Self {
        let mut width_descriminator = 0;
        let code_points: Vec<u16> = text.as_ref().encode_utf16().collect();

        let line_endings: Vec<TextRange> =
            code_points
                .iter()
                .enumerate()
                .fold(Vec::new(), |mut acc, (index, code)| {
                    if *code == b'\n'.into() {
                        let prev_lf = if acc.is_empty() {
                            0
                        } else {
                            acc.last().unwrap().end + 1
                        };

                        let range: TextRange = (prev_lf..=index).into();
                        let width = range.units();
                        if width > width_descriminator {
                            width_descriminator = width;
                        }
                        acc.push(range);
                    }
                    acc
                });

        let dimensions = Boundary {
            origin: Position::default(),
            height: line_endings.len(),
            width: width_descriminator,
        };
        Self {
            id: NodeId::new(),
            code_points,
            dimensions,
            line_endings,
        }
    }

    pub fn lines(&self) -> Vec<String> {
        self.line_endings
            .iter()
            .flat_map(|range| String::from_utf16(&self.code_points[range.start..=range.end]))
            .collect()
    }
}

impl PartialOrd for TextNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Debug for TextNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Line")
            .field("graphemes", &self.code_points)
            .field("dimensions", &self.dimensions)
            .field(
                "text",
                &String::from_utf16(&self.code_points).unwrap_or_default(),
            )
            .finish()
    }
}

pub struct TextCursor<'node> {
    position: Position,
    node: &'node TextNode,
}

impl<'node> TextCursor<'node> {
    pub fn new(node: &'node TextNode) -> Self {
        Self {
            position: Position::default(),
            node,
        }
    }
}

impl<'node> Cursor for TextCursor<'node> {
    type Value = &'node [u16];

    fn seek(&mut self, to: &Position) -> Option<Self::Value> {
        let src = self.position.column;
        let slice = &self.node.code_points[src..to.column];
        self.position = *to;
        Some(slice)
    }

    fn position(&self) -> &Position {
        &self.position
    }
}

/// The line and column values of a [TextNode] within a buffer. These values are analogous
/// to a pair of x and y coordinates on a 2d grid.
#[derive(Clone, Copy, Debug, Default, Eq, Ord)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub(crate) const MAX_COLUMN: usize = 255;

    pub fn clamp(Self { line, column }: Self) -> Self {
        Self {
            line,
            column: std::cmp::min(column, Self::MAX_COLUMN),
        }
    }

    /// A hash in the form of a 32-bit integer where the upper 24 bits store the line
    /// value and remaining lower bits store the column value. The composition may change in
    /// the future as this imposes a rather small limitation on the column value.
    pub fn hash(&self) -> u32 {
        (self.line << 24 | self.column & Self::MAX_COLUMN) as u32
    }
}

impl Hash for Position {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.hash());
        state.finish();
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.hash().partial_cmp(&other.hash())
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.hash().eq(&other.hash())
    }
}

impl Add<(usize, usize)> for Position {
    type Output = Self;

    fn add(self, rhs: (usize, usize)) -> Self::Output {
        let (delta_row, delta_col) = rhs;
        let Self { line, column } = self;
        Position {
            line: line + delta_row,
            column: delta_col + column,
        }
    }
}

impl AddAssign<(usize, usize)> for Position {
    fn add_assign(&mut self, rhs: (usize, usize)) {
        let (delta_row, delta_col) = rhs;
        self.line += delta_row;
        self.column += delta_col;
    }
}

impl From<(usize, usize)> for Position {
    fn from(value: (usize, usize)) -> Self {
        Position {
            line: value.0,
            column: value.1,
        }
    }
}

/// A range within a provided text. Currently, this type shouldn't be used directly but instead be used
/// through the current apis in place that make use of it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TextRange {
    pub start: usize,
    pub end: usize,
}

impl TextRange {
    /// The total amount of units this range covers. A single unit, in the current application usage, can be seen as
    /// a [`grapheme`](https://unicode.org/glossary/#grapheme).
    pub fn units(&self) -> usize {
        self.end - self.start
    }
}

impl From<RangeInclusive<usize>> for TextRange {
    fn from(value: RangeInclusive<usize>) -> Self {
        Self {
            start: *value.start(),
            end: *value.end(),
        }
    }
}

pub fn assert_utf8_empty(text: impl AsRef<[u8]>) -> bool {
    text.as_ref()
        .into_iter()
        .map(|byte| char::from(*byte))
        .all(char::is_whitespace)
}

/// A [Newline](https://en.wikipedia.org/wiki/Newline) type supported by the current text processors set in place.
#[derive(Default, Debug)]
pub enum LineEnding {
    #[default]
    Crlf,
    Lf,
}

impl LineEnding {
    pub fn as_utf16(&self) -> [u16; 2] {
        let _bytes: [u16; 2] = [0u16; 2];
        let encoded = match self {
            Self::Crlf => [13u16, 10u16],
            Self::Lf => [0u16, 10u16],
        };
        encoded
    }
}

impl ToString for LineEnding {
    fn to_string(&self) -> String {
        match self {
            Self::Crlf => "\r\n".into(),
            Self::Lf => "\n".into(),
        }
    }
}

impl PartialEq for LineEnding {
    fn eq(&self, other: &Self) -> bool {
        self.as_utf16().eq(&other.as_utf16())
    }
}

impl PartialEq<[u16]> for LineEnding {
    
    fn eq(&self, other: &[u16]) -> bool {
        match self {
            Self::Crlf => other[0] == 13u16 && other[1] == 10u16,
            Self::Lf => other[1] == 10u16,
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_line_break() {
        let text = r"hello i am


something interesting.
";
        let node= crate::text::TextNode::new_delimitered(text);
        let mut buffer = crate::buffer::TextBuffer::new();
        buffer.push(node);
        println!("{}", buffer);
    }
}
