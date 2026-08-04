# C ABI limitations

Oxiland tip **0.12** ships a Redland-shaped C surface (`oxiland-capi`) and a
librdf-compatible packaging path (demonstrated in milestone 0.11). The following
limits remain part of the product contract.

## Compatibility boundary

- **Source compatibility** is proven for the frozen C corpus and selected
  downstream consumers under `-Werror` against both Redland and Oxiland headers.
- **Binary interchange** is proven via the librdf-compat shared library and
  ABI-swap smoke on supported Unix targets in the qualification matrix.
- Symbols outside the frozen inventory remain omitted; programs that call them
  will not link.

## Behavioral gaps (fail closed)

These exported APIs return an error / NULL instead of silently succeeding:

- `librdf_stream_add_map` (iterator maps are supported; stream maps are not)
- `librdf_parser_get_namespaces_seen_*` (namespace tracking is not implemented)
- `librdf_serializer_set_error` / `librdf_serializer_set_warning`
- Non-baseline `librdf_*_register_factory` names and any non-NULL factory
  callback (ADR-025: callbacks are never executed)

## Supported with notes

- `librdf_world_set_logger` uses a flattened logger callback (code/level/facility/
  message/locator strings), not Redland's `librdf_log_message*` object form.
  `librdf_log` is variadic and forwards into `librdf_log_simple`.
- iostream arguments are Oxiland tagged handles (`oxiland_new_iostream*`), not
  Raptor objects.
- Optional durable backends open when compiled; otherwise they remain
  known-but-not-compiled.
- Double-free after allocator reuse remains undefined.

## Ownership

- Mix of system `free` with Oxiland allocations is unsupported.
- Stream and query-result handles are bound to their model lifetime.
- Thread-safety is per handle type—see [design/0.8-cabi.md](../design/0.8-cabi.md).

## Evidence

See [`docs/reports/0.11.md`](../reports/0.11.md) and
`scripts/check-0.11-release.py`. Claims must not exceed the verified matrix.
