# Courses Backend

This is the new backend repository for CMU Courses.

## Development

### Environment Variables

Copy the `.env.example` file to `.env`.

* To get a `CANVAS_ACCESS_TOKEN`, navigate to the [Canvas settings page](https://canvas.cmu.edu/profile/settings) and press `+ New Access Token`.
* The `OIDC_ISSUER_URL` and `DATABASE_URL` come from Authentik and Railway, respectively. 

## Running

In this directory, use `cargo run --bin <name>`, where name is one of `datafetcher | server`. Before running the server, make sure to check its [README](./crates/server/README.md).

## Database

You should install `sea-orm-cli` using `cargo install sea-orm-cli`. The following instructions assume you are in the project root:

### Generating a new migration

```bash
sea-orm-cli migrate generate --migration-dir ./crates/migration
```

### Applying migrations

```bash
sea-orm-cli migrate up --migration-dir ./crates/migration
```

### Generating entities from database

```bash
sea-orm-cli generate entity -o ./crates/database/src/entities
```
