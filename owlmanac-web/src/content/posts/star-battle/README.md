# star-battle: how it's made

- **Data:** the puzzle pool, `data/easy.json` and `data/hard.json`, from `star-battle`'s `pool` binary: `cd owlmanac-wasm && cargo run --release -p star-battle --bin pool -- 2000 ../owlmanac-web/src/content/posts/star-battle/data`.
- **WASM:** `star-battle-wasm`, shared, in `src/wasm/star-battle/` (generated): `cd owlmanac-web && npm run wasm -- star-battle-wasm`.
- **Cover:** drawn by `cover.mjs` from the first Hard puzzle, after the WASM is built: `cd owlmanac-web && npm run covers -- star-battle`.
