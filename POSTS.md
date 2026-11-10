# Posts and their pipelines

What each post uses from the producers, and the command that regenerates it. Commands run from the repo root. The rules behind this are in the [README](README.md).

| Post | Data (`data/`, committed) | WASM (generated) | Regenerate |
| --- | --- | --- | --- |
| `mapping-the-milky-way` | Gaia star densities, parquet, from `owlmanac-explore/mapping-the-milky-way.py` | — | `uv run owlmanac-explore/mapping-the-milky-way.py` |
| `from-brute-force-to-cdcl` | — (puzzles are in `engine/puzzles.ts`) | `sat-lab-wasm`, into the post's `wasm/` | `cd owlmanac-web && npm run wasm:sat-lab` |
| `making-puzzles-with-a-sat-oracle` | Strategy comparison, `strategies.json`, from `star-battle`'s `strategies` binary | `star-battle-wasm`, shared, in `src/wasm/star-battle/` | `cd owlmanac-wasm && cargo run --release -p star-battle --bin strategies -- ../owlmanac-web/src/content/posts/making-puzzles-with-a-sat-oracle/data` |
| `star-battle` | The puzzle pool, `easy.json` and `hard.json`, from `star-battle`'s `pool` binary | `star-battle-wasm`, shared, in `src/wasm/star-battle/` | `cd owlmanac-wasm && cargo run --release -p star-battle --bin pool -- 2000 ../owlmanac-web/src/content/posts/star-battle/data` |

The covers of the three SAT posts are drawn from their own puzzles: `cd owlmanac-web && npm run covers`, after the shared WASM is built.

Shared WASM is built by `cd owlmanac-web && npm run wasm:star-battle`. Every WASM package is rebuilt by `npm run wasm`, which `npm run dev` and `npm run build` run first.
