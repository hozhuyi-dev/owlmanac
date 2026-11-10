//! Each thing a reader can ask about a puzzle, one file per question, each an `impl Session` block.

pub mod blame;
pub mod check;
pub mod count;
pub mod difficulty;
pub mod forced;
pub mod learned;
pub mod why_not;
