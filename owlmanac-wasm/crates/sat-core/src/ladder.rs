//! The solver built up one idea at a time, for measuring what each idea adds. Learning and
//! backjumping share a rung because both come out of the same conflict analysis.

use crate::solver::Config;

pub fn ladder() -> [(&'static str, Config); 6] {
    let basic = Config::basic_cdcl();
    [
        ("DPLL", Config::dpll()),
        ("Clause learning", basic),
        (
            "+ VSIDS",
            Config {
                vsids: true,
                ..basic
            },
        ),
        (
            "+ phase saving",
            Config {
                vsids: true,
                phase_saving: true,
                ..basic
            },
        ),
        (
            "+ restarts",
            Config {
                reduce_interval: None,
                ..Config::default()
            },
        ),
        ("+ clause deletion", Config::default()),
    ]
}
