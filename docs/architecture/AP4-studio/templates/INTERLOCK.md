---
name: ap4-interlock-template
description: "AP4 interlock contract template — one per cross-department interface, versioned, frozen at P0."
type: Template
plan_id: AP4
---

# IC-<n> — <name>

```yaml
provider:
consumer:
frozen_at:            # date + commit sha at P0
version: 1.0.0        # minor=compatible addition; major=breaking (change-order)
change_orders: []     # ids of approved mid-cycle changes
```

## Purpose
One paragraph: what flows across this boundary and why it exists.

## Surface (the actual contract)
Exact types / schemas / file formats / command signatures — copy the real definitions.
Prose summaries are not contracts.

## Guarantees
- What the provider promises (stability windows, determinism, performance bounds).

## Obligations
- What the consumer must not assume; error-handling expectations.

## Tests guarding this contract
Named tests or probe paths on both sides.

## Changelog

| ver | change | approved |
|---|---|---|
