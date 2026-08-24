# Sky Inventory — Rust Inventory Manager

**Status: engineering beta.** This repository now implements a focused in-memory SKU inventory service in Rust/Actix Web. CI verifies formatting, compilation, Clippy, tests, dependency audit, release build, Docker build, and non-root image execution. Production deployment and durable storage are not verified here.

## Implemented behavior

- Create or replace an item by SKU with validated name and starting quantity.
- Retrieve one item or list all items in deterministic SKU order.
- Atomically adjust stock while preventing negative inventory and integer overflow.
- Report total units for readiness checks.
- Return explicit 400/404/5xx responses instead of silently accepting invalid operations.
- Run as a non-root container user.

## API

- `PUT /v1/items`
- `GET /v1/items`
- `GET /v1/items/{sku}`
- `POST /v1/items/{sku}/adjust`
- `GET /healthz`
- `GET /readyz`

Create an item:

```json
{
  "sku": "SKU-1001",
  "name": "Example product",
  "quantity": 12
}
```

Adjust stock:

```json
{
  "delta": -2
}
```

## Run locally

```bash
cargo run
```

Set `BIND_ADDR` to override the default `0.0.0.0:8080` bind address.

## Verification

Because this repository does not yet commit `Cargo.lock`, CI first generates one and then performs locked verification:

```bash
cargo generate-lockfile
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --locked
cargo audit
cargo build --release --locked
docker build -t sky-inventory .
docker run --rm --entrypoint=id sky-inventory -u
```

The container is expected to run as UID `10001`, not root.

## Architecture

`src/lib.rs` owns inventory invariants and synchronization. The current store is an in-memory `HashMap` protected by a mutex and exposed through a small reusable domain API. `src/main.rs` maps that domain into HTTP endpoints.

This is intentionally a compact engineering product. It is not a warehouse-management system, ERP, distributed inventory ledger, or durable database.

## SKYCOIN4444 integration

Keep this repository independently deployable. A SKYCOIN4444 marketplace, shop, or fulfillment adapter can call the HTTP API for development/demo inventory operations. Durable ecosystem inventory should add an explicit persistence adapter, authentication/authorization, reconciliation, audit history, and idempotency rather than copying this code into a flagship application.

## Security and limitations

The service does **not** currently provide persistence, authentication, authorization, tenant isolation, TLS termination, rate limiting, durable audit logs, reservation semantics, purchase-order workflows, or backup/restore. See [`SECURITY.md`](SECURITY.md) for boundaries and [`CHANGELOG.md`](CHANGELOG.md) for changes.

## License

See `LICENSE`.
