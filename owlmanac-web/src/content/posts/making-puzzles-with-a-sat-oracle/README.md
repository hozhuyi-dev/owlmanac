# making-puzzles-with-a-sat-oracle: how it's made

- **Data:** the strategy comparison, `data/strategies.json`, from `star-battle`'s `strategies` binary: `cd owlmanac-wasm && cargo run --release -p star-battle --bin strategies -- ../owlmanac-web/src/content/posts/making-puzzles-with-a-sat-oracle/data`.
- **WASM:** `star-battle-wasm`, shared, in `src/wasm/star-battle/` (generated): `cd owlmanac-web && npm run wasm -- star-battle-wasm`.
- **Cover:** drawn by `cover.mjs` from a real attempt, after the WASM is built: `cd owlmanac-web && npm run covers -- making-puzzles-with-a-sat-oracle`.
