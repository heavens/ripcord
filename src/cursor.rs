use crate::text::Position;

/// An abstraction for Cursor-like types. This uniform api, even in its current naive state, provides a level of
/// convenience for navigating over a collection of items.
pub trait Cursor {
    /// The value type for the items being navigated over.
    type Value;

    /// Seek to a position within the collection of items. It is up to the implementor to ensure that
    /// the state of the cursor is maintained such that the position is updated to reflect each call of `seek` and so forth.
    ///
    ///
    /// ## Example
    /// ```rust,norun
    ///  impl<'node> Cursor for TextCursor<'node> {
    ///    type Value = &'node [u16];
    ///
    ///    fn seek(&mut self, to: &Position) -> Option<Self::Value> {
    ///       let src = self.position.column;
    ///       let slice = &self.node.code_points[src..to.column];
    ///       self.position = *to;
    ///       Some(slice)
    ///    }
    ///
    ///    fn position(&self) -> &Position {
    ///       &self.position
    ///    }
    /// }
    /// ```
    fn seek(&mut self, to: &Position) -> Option<Self::Value>;

    /// The current position of the cursor relative to the collection of items its navigating over. 
    fn position(&self) -> &Position;
}
