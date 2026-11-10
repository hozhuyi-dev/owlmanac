//! Restarts: abandons the current guesses, keeping everything learnt, to escape a bad early choice.

use crate::observer::Observer;
use crate::solver::Solver;

impl<O: Observer> Solver<O> {
    pub(crate) fn restart_due(&self) -> bool {
        self.next_restart
            .is_some_and(|at| self.stats.conflicts >= at)
    }

    pub(crate) fn restart(&mut self) {
        self.backjump(0);
        self.stats.restarts += 1;
        self.observer.on_restart();
        self.next_restart = self
            .config
            .restart_base
            .map(|base| self.stats.conflicts + base * luby(self.stats.restarts + 1));
    }
}

/// The Luby sequence, counting from 1: 1, 1, 2, 1, 1, 2, 4, 1, 1, 2, ...
fn luby(mut index: u64) -> u64 {
    loop {
        let bits = u64::BITS - index.leading_zeros();
        if index == (1 << bits) - 1 {
            return 1 << (bits - 1);
        }
        index -= (1 << (bits - 1)) - 1;
    }
}

#[cfg(test)]
mod tests {
    use super::luby;

    #[test]
    fn luby_matches_the_published_sequence() {
        let sequence: Vec<u64> = (1..=15).map(luby).collect();
        assert_eq!(sequence, [1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8]);
    }
}
