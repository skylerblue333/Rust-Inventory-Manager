# Security Policy

## Status

Sky Inventory is an **engineering beta**. CI verifies Rust formatting, compilation, Clippy, tests, a generated dependency-lock audit, release build, Docker build, and non-root image execution. Production infrastructure is not verified here.

## Boundaries

The service validates SKU/name lengths, forbids negative quantities, rejects zero adjustments, prevents underflow/overflow, and fails closed if the in-memory store lock is poisoned. It does not provide authentication, authorization, persistence, tenant isolation, TLS termination, rate limiting, audit-log durability, or encryption at rest.

Operators are responsible for access control, durable storage, backups, reconciliation, TLS, request limits, observability, and authorization before using this service outside a trusted development environment.

## Reporting

Use GitHub private vulnerability reporting when available. Do not place credentials, private inventory data, or exploitable vulnerability details in public issues.
