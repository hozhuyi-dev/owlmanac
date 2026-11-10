//! Every clause, stored in one flat array of literals so propagation stays cache-friendly.
//!
//! A `ClauseRef` indexes a table of headers and each header says where its literals are. Compacting
//! the literals only rewrites headers, so a `ClauseRef` held by a watcher or a reason stays valid.
//!
//! Solver invariants on the literal order: positions 0 and 1 are the watched literals, and a clause
//! that is the reason for an assignment keeps the implied literal at position 0.

use super::lit::Lit;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ClauseRef(u32);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Original,
    Learnt,
    Deleted,
}

#[derive(Clone, Copy, Debug)]
struct Header {
    start: u32,
    len: u32,
    /// How many decision levels the clause spanned when learnt: lower is more valuable.
    lbd: u32,
    kind: Kind,
}

#[derive(Default)]
pub struct ClauseDb {
    literals: Vec<Lit>,
    headers: Vec<Header>,
    free_headers: Vec<ClauseRef>,
    learnt: Vec<ClauseRef>,
}

impl ClauseDb {
    pub fn add_original(&mut self, lits: &[Lit]) -> ClauseRef {
        self.add(lits, 0, Kind::Original)
    }

    pub fn add_learnt(&mut self, lits: &[Lit], lbd: u32) -> ClauseRef {
        let clause = self.add(lits, lbd, Kind::Learnt);
        self.learnt.push(clause);
        clause
    }

    fn add(&mut self, lits: &[Lit], lbd: u32, kind: Kind) -> ClauseRef {
        debug_assert!(lits.len() >= 2, "units are assigned, not stored");
        let header = Header {
            start: self.literals.len() as u32,
            len: lits.len() as u32,
            lbd,
            kind,
        };
        self.literals.extend_from_slice(lits);
        match self.free_headers.pop() {
            Some(clause) => {
                self.headers[clause.0 as usize] = header;
                clause
            }
            None => {
                self.headers.push(header);
                ClauseRef(self.headers.len() as u32 - 1)
            }
        }
    }

    fn header(&self, clause: ClauseRef) -> &Header {
        &self.headers[clause.0 as usize]
    }

    pub fn lits(&self, clause: ClauseRef) -> &[Lit] {
        let Header { start, len, .. } = *self.header(clause);
        &self.literals[start as usize..(start + len) as usize]
    }

    pub fn lits_mut(&mut self, clause: ClauseRef) -> &mut [Lit] {
        let Header { start, len, .. } = *self.header(clause);
        &mut self.literals[start as usize..(start + len) as usize]
    }

    pub fn lbd(&self, clause: ClauseRef) -> u32 {
        self.header(clause).lbd
    }

    pub fn is_deleted(&self, clause: ClauseRef) -> bool {
        self.header(clause).kind == Kind::Deleted
    }

    pub fn learnt(&self) -> &[ClauseRef] {
        &self.learnt
    }

    /// Marks a clause as gone. Its space comes back at the next `compact`, once no watcher lists it.
    pub fn delete(&mut self, clause: ClauseRef) {
        self.headers[clause.0 as usize].kind = Kind::Deleted;
    }

    /// Reclaims the space of deleted clauses. Live clauses keep their `ClauseRef`.
    pub fn compact(&mut self) {
        let mut literals = Vec::with_capacity(self.literals.len());
        self.free_headers.clear();
        for (index, header) in self.headers.iter_mut().enumerate() {
            if header.kind == Kind::Deleted {
                self.free_headers.push(ClauseRef(index as u32));
                continue;
            }
            let (start, end) = (header.start as usize, (header.start + header.len) as usize);
            header.start = literals.len() as u32;
            literals.extend_from_slice(&self.literals[start..end]);
        }
        self.literals = literals;

        let headers = &self.headers;
        self.learnt
            .retain(|clause| headers[clause.0 as usize].kind != Kind::Deleted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::lit::Var;

    fn clause(numbers: &[u32]) -> Vec<Lit> {
        numbers.iter().map(|&n| Var::new(n).positive()).collect()
    }

    #[test]
    fn compacting_keeps_live_clauses_at_their_refs() {
        let mut db = ClauseDb::default();
        let first = db.add_original(&clause(&[0, 1]));
        let gone = db.add_learnt(&clause(&[2, 3, 4]), 3);
        let last = db.add_learnt(&clause(&[5, 6]), 2);

        db.delete(gone);
        db.compact();

        assert_eq!(db.lits(first), clause(&[0, 1]));
        assert_eq!(db.lits(last), clause(&[5, 6]));
        assert_eq!(db.learnt(), &[last]);
        assert_eq!(db.literals.len(), 4);
    }

    #[test]
    fn a_deleted_header_is_reused_after_compacting() {
        let mut db = ClauseDb::default();
        let gone = db.add_learnt(&clause(&[0, 1]), 2);
        db.delete(gone);
        db.compact();

        let next = db.add_learnt(&clause(&[2, 3]), 2);

        assert_eq!(next, gone);
        assert_eq!(db.lits(next), clause(&[2, 3]));
    }
}
