//! What the player has marked so far.

use crate::puzzle::Cell;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mark {
    Star,
    /// The player has ruled out a star here.
    Empty,
}

/// A mark on a particular cell, whether the player's or deduced.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize)]
pub struct Placed {
    pub cell: Cell,
    pub mark: Mark,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Board {
    marks: BTreeMap<Cell, Mark>,
}

impl Board {
    pub fn new() -> Self {
        Board::default()
    }

    /// Places a mark, replacing any already on that cell.
    pub fn place(&mut self, placed: Placed) {
        self.marks.insert(placed.cell, placed.mark);
    }

    pub fn clear(&mut self, cell: Cell) {
        self.marks.remove(&cell);
    }

    pub fn mark_at(&self, cell: Cell) -> Option<Mark> {
        self.marks.get(&cell).copied()
    }

    pub fn placed(&self) -> impl Iterator<Item = Placed> + '_ {
        self.marks
            .iter()
            .map(|(&cell, &mark)| Placed { cell, mark })
    }

    /// A copy of this board with one more mark.
    pub fn with(&self, placed: Placed) -> Board {
        let mut board = self.clone();
        board.place(placed);
        board
    }
}
