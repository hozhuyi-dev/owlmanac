import type { AsyncDuckDB } from "@duckdb/duckdb-wasm";
import wasmUrl from "@duckdb/duckdb-wasm/dist/duckdb-eh.wasm?url";
import workerUrl from "@duckdb/duckdb-wasm/dist/duckdb-browser-eh.worker.js?url";

// One database per page, opened by the first query, so pages without charts never download it.
let db: Promise<AsyncDuckDB> | undefined;
const files = new Map<string, Promise<string>>();

function database(): Promise<AsyncDuckDB> {
  db ??= (async () => {
    const duckdb = await import("@duckdb/duckdb-wasm");
    const instance = new duckdb.AsyncDuckDB(new duckdb.VoidLogger(), new Worker(workerUrl));
    await instance.instantiate(wasmUrl);
    return instance;
  })();
  return db;
}

/** Registers a parquet file by URL and returns the SQL that reads it, for a FROM clause. */
export function parquet(url: string): Promise<string> {
  let table = files.get(url);
  if (!table) {
    table = (async () => {
      const duckdb = await import("@duckdb/duckdb-wasm");
      const name = `${url.replace(/\W/g, "_")}.parquet`;
      const href = new URL(url, location.href).href;
      await (await database()).registerFileURL(name, href, duckdb.DuckDBDataProtocol.HTTP, false);
      return `read_parquet('${name}')`;
    })();
    files.set(url, table);
  }
  return table;
}

/** Runs a query and returns its rows as plain objects. Cast int64 columns in SQL: they come back as BigInt. */
export async function query<T>(sql: string): Promise<T[]> {
  const conn = await (await database()).connect();
  try {
    const result = await conn.query(sql);
    return result.toArray().map((row) => row.toJSON() as T);
  } finally {
    await conn.close();
  }
}
