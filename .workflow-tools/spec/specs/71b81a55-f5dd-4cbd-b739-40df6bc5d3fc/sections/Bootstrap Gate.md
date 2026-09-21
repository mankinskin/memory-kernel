The session-bootstrap epic gates on the **core** feedback-api milestone, not the full program.

- Bootstrap gate ticket: `c7542933` "[feedback-api] Core curation surface — URN usage counting + entity ratings (bootstrap gate)" (default store; migration candidate for memory-api).
- `effba966` (epic) and `412964a3` (runtime) `depends_on c7542933`.
- The full feedback-api program `b1e9e744` `depends_on c7542933` (core is a child milestone); heavyweight slices (at-scale search, SLOs, governance, retention) build on top and do NOT gate bootstrapping.

This spec's acceptance criteria define the core contract that `c7542933` must satisfy.