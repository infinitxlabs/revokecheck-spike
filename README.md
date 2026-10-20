# revokecheck-spike

Three revocation cases for [p2panda-auth](https://crates.io/crates/p2panda-auth) 0.7.1, each run in every delivery order that respects the operations' dependencies. Peers receive group operations in different orders; the group they end up with must not depend on that order.

| Case | Rule it checks (from the crate's documentation) | Orders run | Members at the end |
|---|---|---:|---|
| A manager is removed while adding someone | The removed manager's concurrent add is invalidated | 2 | Alice |
| Two managers remove each other at the same time | Both removals apply | 2 | Claire |
| A manager is removed, re-added, and added someone in between | Back as a member, the concurrent add still invalidated | 3 | Alice, Bob |

```
$ cargo test
test removing_a_manager_drops_the_add_it_made_concurrently ... ok
test mutual_removal_applies_both_removals ... ok
test a_re_added_manager_stays_but_loses_its_concurrent_add ... ok
test result: ok. 3 passed; 0 failed
```

p2panda-auth 0.7.1 needs Rust 1.96 or newer; these ran on 1.99.1. The expected members come from the rules in the crate's documentation, not from a run of the code. This is a feasibility check before a larger study, not a review of p2panda-auth.

Written with AI coding assistants under the direction of Amir Daneshpajouh, and checked by its tests.

Licence: EUPL-1.2.
