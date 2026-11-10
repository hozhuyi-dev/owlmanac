//! Variables and literals. A literal packs its variable and sign into one `u32` (`2v + sign`), so
//! negating is a bit flip and a literal can index an array.

use std::ops::Not;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Var(u32);

impl Var {
    pub fn new(index: u32) -> Self {
        Var(index)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn positive(self) -> Lit {
        Lit(self.0 << 1)
    }

    pub fn negative(self) -> Lit {
        Lit(self.0 << 1 | 1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Lit(u32);

impl Lit {
    pub fn var(self) -> Var {
        Var(self.0 >> 1)
    }

    pub fn is_positive(self) -> bool {
        self.0 & 1 == 0
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// Writes DIMACS's numbering, the reverse of `from_dimacs`.
    pub fn to_dimacs(self) -> i32 {
        let number = self.var().index() as i32 + 1;
        if self.is_positive() { number } else { -number }
    }

    /// Reads DIMACS's numbering: `3` is variable 2 (counting from 0), `-3` its negation. `0` is not a literal.
    pub fn from_dimacs(number: i32) -> Self {
        debug_assert!(number != 0);
        let var = Var(number.unsigned_abs() - 1);
        if number > 0 {
            var.positive()
        } else {
            var.negative()
        }
    }
}

impl Not for Lit {
    type Output = Lit;

    fn not(self) -> Lit {
        Lit(self.0 ^ 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_literal_knows_its_variable_and_sign() {
        let var = Var::new(7);
        assert_eq!(var.positive().var(), var);
        assert_eq!(var.negative().var(), var);
        assert!(var.positive().is_positive());
        assert!(!var.negative().is_positive());
    }

    #[test]
    fn negating_twice_is_the_same_literal() {
        let lit = Var::new(3).negative();
        assert_eq!(!lit, Var::new(3).positive());
        assert_eq!(!!lit, lit);
    }

    #[test]
    fn dimacs_numbers_count_from_one() {
        assert_eq!(Lit::from_dimacs(1), Var::new(0).positive());
        assert_eq!(Lit::from_dimacs(-4), Var::new(3).negative());
        assert_eq!(Lit::from_dimacs(-4).to_dimacs(), -4);
    }
}
