# plato-route

> Message routing for PLATO tiles — pattern-based routing with priority and statistics

## What This Does

plato-route provides a routing table that directs tiles to destinations based on source patterns, tile types, and priority levels. Routes are matched in order (first match wins), with a configurable default destination. Statistics track routing decisions for observability.

## The Key Idea

Tiles come from many sources (sensors, agents, alerts) and need to go to the right place. A temperature tile from "kitchen-temp" goes to the analytics pipeline. An alert tile goes to the notification system. plato-route matches source patterns against routing rules and dispatches accordingly.

## Install

```bash
cargo add plato-route
```

## Quick Start

```rust
use plato_route::*;

let mut table = RouteTable::new();
table.add_route(Route {
    source_pattern: "temp-*".into(),
    tile_type: Some("alert".into()),
    destination: "notification-service".into(),
    priority: Priority::High,
});
table.set_default("analytics-pipeline");

let decision = table.route("temp-kitchen", Some("alert"));
// → destination: "notification-service", priority: High
```

## API Reference

| Type | Description |
|---|---|
| `Priority` | `Low` / `Normal` / `High` / `Critical`. Convertible to/from u8. |
| `Route { source_pattern, tile_type, destination, priority }` | A routing rule |
| `RoutingDecision { destination, priority, matched_route }` | Result of routing |
| `RouteTable` | `add_route()`, `remove_route()`, `route()`, `set_default()`, `stats()` |
| `RouteStats { total_routed, by_destination, by_priority, dropped }` | Routing statistics |

## Testing

20 tests: route matching, pattern matching, priority mapping, default destination, statistics, route removal, edge cases.

## License

Apache-2.0
