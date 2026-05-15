# tester crate

**THIS IS NOT A TEST SUITE.** It is a sandbox / scratchpad for manually exercising `db` crate features. Binary name: `muoxi_sandbox`.

## FILES

```
tester/
├── Cargo.toml         # bin = muoxi_sandbox; reads workspace deps (db, redis, diesel, tokio, serde)
└── src/
    └── main.rs        # ~40 lines; round-trips a CacheSocket through Redis
```

## CURRENT BEHAVIOR

`main()` does ONE thing: round-trips a `CacheSocket` through Redis (`set_ip → set_port → dump`, sleep 30s, `load`, print port). The 30-second sleep is intentional - it gives time to inspect Redis manually with `redis-cli` between dump and load.

The 100+ lines of commented-out experiments in the original (Postgres CRUD scratch, plain redis SET/GET, JSON serialization, transactions, BSON ObjectId) were DROPPED during the modernization port - most referenced types that no longer exist (`db::clients::Client`, `bson::ObjectId`, `db::templates::ClientDB`).

## CONVENTIONS

- File-top: `#![allow(unused_imports, dead_code, unused_variables)]` - INTENTIONAL. Do not remove.

## RUN

```bash
redis-server                        # required, port 6379
cargo run --bin muoxi_sandbox       # then in another shell:
redis-cli KEYS 'Socket:*'           # inspect what was written
```

Sandbox needs Redis running. Postgres NOT required for the current code path.

## ANTI-PATTERNS

- DO NOT add automated assertions here - this is for manual exploration only. Promote validated experiments into a proper `[dev-dependencies]` test setup elsewhere.
- DO NOT depend on the `tester` crate from another crate. It is a leaf binary.

## SYSTEM REQUIREMENT

This binary transitively links `db` (Diesel/Postgres). Building requires `libpq-dev`. `cargo check` works without it.
