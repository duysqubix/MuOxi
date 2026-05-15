# benchmarks crate

Custom benchmark harness. NOT Criterion. NOT `cargo bench`. Standalone binary `muoxi_benchmarks`.

## FILES

```
benchmarks/
├── Cargo.toml              # bin = muoxi_benchmarks; reads workspace deps
├── src/
│   ├── main.rs             # calls benchmark_io_json_100_000()
│   ├── io.rs               # the workload(s)
│   └── report.rs           # ReportBuilder + Report writer
└── results/
    └── io_benchmarks.txt   # last run output
```

## REPORT PATTERN

`ReportBuilder::new().with_title(..).with_body(..).with_footnotes(..).build_report().write_report(path)`. Builder takes `&mut self` (chainable mutable refs); `build_report()` clones internal `Option<String>` fields into a final `Report`.

## REQUIRED FIXTURE (NOT IN REPO)

`benchmarks/db_100_000.json` - `HashMap<usize, Person>` with 100k entries. `Person` schema:

```json
{ "1": {"id":1,"name":"...","email":"...","hp":0,"mana":0,"vit":0}, ... }
```

Generate one before running. The workload:
1. `read_json_file("benchmarks/db_100_000.json")`
2. `json_to_object` → `HashMap<usize, Person>`
3. mutate entry `1`'s name
4. `write_json_file("benchmarks/db_100_000_altered.json")`
5. `remove_file("benchmarks/db_100_000_altered.json")` (cleanup at end)

## RUN

```bash
cargo run --bin muoxi_benchmarks       # writes benchmarks/results/io_benchmarks.txt
```

CWD MUST be repo root - all paths are relative.

## ANTI-PATTERNS

- DO NOT add `cargo bench` integration without first vendoring `criterion` - the workspace has zero `[dev-dependencies]`.
- DO NOT remove the `remove_file` cleanup step at the end of `benchmark_io_json_100_000` - benchmark is destructive on its own output.
- DO NOT write benchmark output anywhere other than `benchmarks/results/` - `report.rs` does no path normalization.
- DO NOT remove `benchmarks/results/.init` (zero-byte marker) - keeps the dir tracked in git.

## SYSTEM REQUIREMENT

Default `db-sqlite` build: **none**. The benchmark binary transitively links
`db`, which now defaults to SQLite via the bundled `libsqlite3-sys`.

If you opt into Postgres (`cargo build --no-default-features --features db-postgres`),
you'll need `libpq-dev` (or your platform's equivalent) installed on the host.
