This directory is for **import / export** payloads only — not runtime canonical state.
The canonical store is the SQLite database at `data/world.db` (or Postgres when the
`db-postgres` feature is enabled).

Place sample worlds here for tests, demos, or migration scripts to consume. They are
not loaded automatically.
