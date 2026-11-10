//! The rules of a puzzle, each one something a reader can be told was broken.

use crate::puzzle::Cell;
use serde::Serialize;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize)]
#[serde(tag = "kind", content = "at", rename_all = "camelCase")]
pub enum Rule {
    Row(usize),
    Column(usize),
    Region(usize),
    /// A star here can't touch a star to its right or in the row below.
    NoTouching(Cell),
}
