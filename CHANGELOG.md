# Changelog

## Unreleased

### Added
- SKU inventory model with bounded identifiers and names.
- Non-negative quantity invariant with overflow-safe stock adjustments.
- Deterministic item listing and total-unit accounting.
- HTTP endpoints for upsert, retrieval, listing, adjustment, health, and readiness.
- Rustfmt, Clippy, tests, dependency audit, release build, Docker build, and non-root CI gates.
- Security and product-status documentation.

### Changed
- Replaced the unrelated generic time-series store behavior with an actual inventory-management domain.
