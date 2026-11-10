# /// script
# requires-python = ">=3.10,<3.13"
# dependencies = ["python-sat"]
# ///
"""Checks sat-cli against tools it shares no code with: answers against CaDiCaL, and UNSAT proofs
with drat-trim. Build first with `cargo build --release -p sat-cli`, then run:

    uv run scripts/check_external.py <path to drat-trim>
"""

import subprocess
import sys
import tempfile
from pathlib import Path

from pysat.formula import CNF
from pysat.solvers import Cadical153

SAT_CLI = Path(__file__).parent.parent / "target" / "release" / "sat-cli"
SIZES = [(50, 218), (100, 426), (150, 639), (200, 852)]
SEEDS = range(50)
EXIT_SAT, EXIT_UNSAT = 10, 20


def model_from(output: str) -> list[int]:
    line = next(line for line in output.splitlines() if line.startswith("v "))
    return [int(token) for token in line.split()[1:-1]]


def check_one(folder: Path, drat_trim: str, num_vars: int, num_clauses: int, seed: int) -> bool:
    formula = folder / "formula.cnf"
    proof = folder / "proof.drat"
    text = subprocess.run(
        [SAT_CLI, "random", str(num_vars), str(num_clauses), "3", str(seed)],
        capture_output=True, text=True, check=True,
    ).stdout
    formula.write_text(text)

    cnf = CNF(from_file=str(formula))
    with Cadical153(bootstrap_with=cnf.clauses) as cadical:
        expected_sat = cadical.solve()

    solved = subprocess.run(
        [SAT_CLI, "solve", formula, "--proof", proof], capture_output=True, text=True,
    )
    name = f"{num_vars} vars, seed {seed}"
    assert solved.returncode in (EXIT_SAT, EXIT_UNSAT), f"{name}: {solved.stderr}"
    assert (solved.returncode == EXIT_SAT) == expected_sat, f"{name}: disagrees with CaDiCaL"

    if expected_sat:
        truths = set(model_from(solved.stdout))
        assert all(truths & set(clause) for clause in cnf.clauses), f"{name}: bad model"
    else:
        checked = subprocess.run(
            [drat_trim, formula, proof, "-t", "600"], capture_output=True, text=True,
        )
        assert "s VERIFIED" in checked.stdout, f"{name}: drat-trim said\n{checked.stdout}"
    return expected_sat


def main() -> None:
    drat_trim = sys.argv[1]
    if not Path(drat_trim).is_file():
        sys.exit(f"drat-trim not found at {drat_trim}")
    with tempfile.TemporaryDirectory() as temporary:
        folder = Path(temporary)
        for num_vars, num_clauses in SIZES:
            answers = [check_one(folder, drat_trim, num_vars, num_clauses, seed) for seed in SEEDS]
            sat = sum(answers)
            print(f"{num_vars} vars: {sat} SAT, {len(answers) - sat} UNSAT, all agreed and verified")


if __name__ == "__main__":
    main()
