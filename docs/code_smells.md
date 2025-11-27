### Code Smell Audit and Task Backlog (bevy_2048)

Last updated: 2025-11-24 07:34

This document captures code smells found in the current codebase and proposes actionable tasks to resolve them. Each task includes rationale and acceptance criteria. Priorities: P0 (urgent correctness/stability), P1 (important quality/maintainability), P2 (refactor/tests), P3 (nice-to-have/perf polish).

---

### P0 — Correctness and Stability

- [DONE] Fix despawn/removal during iteration causing ECS errors
  - Location: `src/game/systems/movement.rs` lines ~16–47
  - Smell: Commented WARN about removing/despawning entities while iterating the same query; this can lead to “entity does not exist” errors.
  - Approach: Collect mutations into a local buffer (e.g., `Vec<Command>` or entity lists) and apply after the loop, or split into two systems: one to mark results, another to apply (with ordering). Consider using `Commands` safely but avoid removing components/despawning inside the same query iteration that depends on them.
  - Acceptance:
    - No WARN logs about missing entities during `queued_movement_system`.
    - Movements and merges behave identically for the player.
  - Status notes: Implemented buffering via `to_delete` and applied despawns after the iteration. See `movement.rs` lines 22–51.

- [PARTIAL] Remove panics/unwraps in runtime systems; handle missing data safely
  - Locations (non-exhaustive):
    - `src/game/systems/game_logic.rs`: `extract_value` uses `unwrap()` and `expect(...)`; `collision_system` and `process_collision_messages` use multiple `unwrap()`; `get_direction` `panic!()` on unexpected cases.
    - `src/game/systems/process.rs`: `transform_query.get_mut(merge_entity) else { panic!(...) }`.
    - `src/game/components.rs`: `on_value_insert` uses multiple `expect(...)`.
  - Smell: Panics in gameplay code can crash the app on data drift or ordering issues.
  - Approach: Replace with graceful handling:
    - Return `Option`/`Result` and early-return/skip when inconsistent; add warning logs.
    - For `get_direction`, return `Option<Direction>`; handle `None` by skipping collision or falling back.
  - Acceptance:
    - No `panic!`/`unwrap()`/`expect()` in runtime systems (test code or initialization may keep asserts).
    - Game no longer crashes on missing component/resource; instead logs warnings and continues or transitions to safe state.
  - Status notes: `get_direction` now returns `Option<Direction>` and is handled (see `game_logic.rs` 236–245 and usage at 221). However, multiple panics/unwraps remain: `process.rs` (lines 29–31, 45–47), `game_logic.rs` (e.g., 205–206, 257–258), `game/mod.rs` board setup (121, 126), `systems/effect.rs` (~30), and `components.rs` (~31). Further refactoring needed.

- [PARTIAL] Fix potential infinite recursion in `acquire_empty_tile`
  - Location: `src/game/systems/game_logic.rs` lines ~130–145
  - Smell: Recursive retry when a randomly picked cell is occupied; if the board is full, recursion never terminates.
  - Approach: Build a list of empty cells once and randomly choose from it; if none, return `None` and let caller decide (e.g., set Lose/Win or skip spawning).
  - Acceptance:
    - `acquire_empty_tile` returns `Option<(col,row,val)>`.
    - `produce_new_tile_system` handles `None` without recursion and without spawning.
  - Status notes: `acquire_empty_tile` implemented with non-recursive selection and `Option` return; unit tests added (see `game_logic.rs` 134–151, 380–397). `produce_new_tile_system` currently `panic!`s on `None` (line 161); needs graceful handling.

- [TODO] Prevent double spawning in `produce_new_tile_system`
  - Location: `src/game/mod.rs` lines ~107–116 (TODO comment) and `src/game/systems/game_logic.rs` lines ~147–159
  - Smell: Spawning new tile sometimes occurs twice upon entering `Decision`.
  - Approach: Audit state transitions and scheduling. Ensure `OnEnter(GameState::Decision)` isn’t added multiple times or chained with other transitions that re-enter; consider adding a guard resource/flag or move spawn logic into a single, well-ordered system with a run criteria that only runs once per transition.
  - Acceptance:
    - Add a debug counter or test to assert exactly one spawn per decision step.
    - Remove the TODO and document the fix in code comments.
  - Status notes: Currently `OnEnter(GameState::Decision)` chains `(produce_new_tile_system, the_end_system)` (see `game/mod.rs` 101–105). No guard flag or counter present; issue not yet addressed.

---

### P1 — Maintainability and Data Hygiene

- [PARTIAL] Eliminate magic numbers in coordinate conversions
  - Location: `src/game/utils.rs` lines ~5–19 (`500.`, `375.`), and uses of `RECT_SIZE`/board origin across UI code.
  - Smell: Hard-coded offsets tie rendering to a specific board placement and resolution; brittle for different screen sizes.
  - Approach: Centralize board origin/offset computation (e.g., derive from `SIZE`, `RECT_SIZE`, and desired center). Replace `500.`/`375.` with derived constants or a `BoardLayout` resource.
  - Acceptance:
    - A single `BoardLayout` (or similar) defines origin, spacing; all helpers use it.
    - Visual layout unchanged for current settings; can be adjusted by changing the resource.
  - Status notes: Helpers now use centralized `X_SHIFT`/`Y_SHIFT` (see `utils.rs` 6–20) instead of literals. A formal `BoardLayout` resource is not yet implemented; offsets are still constants from `game`.

- [TODO] Replace distance-threshold collision with robust AABB overlap
  - Location: `src/game/systems/game_logic.rs` `collision_system` lines ~191–211 and `process_collision_messages` lines ~230–270
  - Smell: Uses squared-distance comparison with an arbitrary `0.96` factor; direction deduced from positions; can misfire during chase or near-equal positions.
  - Approach: Use `translation_to_rect` and/or `position_to_rect` for axis-aligned bounding-box checks; define a `COLLISION_EPSILON` constant; compute overlap and direction from movement intent or last delta rather than static centers when possible.
  - Acceptance:
    - `collision_system` detects collisions via AABB overlap without `0.96` literal; replace with named constant.
    - No false positives when blocks chase each other but don’t overlap.
  - Status notes: Still uses distance-squared with `COLLISION_EPSILON` and nearest-neighbor search (`game_logic.rs` 169–224). No AABB-based detection yet.

- [DONE] Avoid `panic!` in `get_direction` and improve naming/contract
  - Location: `src/game/systems/game_logic.rs` lines ~219–228
  - Smell: Panics when vectors are “impossibly close”.
  - Approach: Return `Option<Direction>` and handle `None`; rename args to clarify higher/lower semantics or use `from_to`.
  - Acceptance:
    - No panic; covered by unit tests for diagonal/zero vectors.
  - Status notes: Implemented as `Option<Direction>`; callers ignore None; unit test exists (`get_direction_test`, 350–378).

- [TODO] Resource/config surfacing for tunables
  - Locations: `SIZE` and `RECT_SIZE` in `src/game/mod.rs`, move time in `GameParams` default, collision epsilon.
  - Smell: Values are scattered; `GameParams` only covers move time.
  - Approach: Consolidate gameplay constants into a configuration resource; expose via a single struct with defaults (and optional CLI/env overrides for desktop).
  - Acceptance:
    - A single `GameConfig` resource holds tunables; modules use it instead of literals.
  - Status notes: Not yet consolidated. `GameParams` exists for move time; other tunables remain spread across modules.

---

### P2 — Structure, Tests, and API Shape

- [PARTIAL] Split monolithic `game/mod.rs` into smaller modules/plugins
  - Location: `src/game/mod.rs` (~488 lines)
  - Smell: Mixes plugin wiring, board setup, UI, tests-like functions, and scheduling. Hard to navigate and evolve.
  - Approach: Extract:
    - `board_setup` and UI into `ui.rs`/`board.rs`.
    - state scheduling into `schedule.rs`.
    - keep only plugin assembly in `mod.rs`.
  - Acceptance:
    - `mod.rs` <= ~150 lines; extracted modules compile; no behavior change.
  - Status notes: Systems have been extracted into `src/game/systems/{input,process,game_logic,effect,movement}.rs`. `mod.rs` is reduced but still 431 lines and includes setup/UI; further extraction needed.

- [PARTIAL] Convert ad-hoc “test” functions into real unit tests
  - Location: `src/game/mod.rs` functions: `rotate_test`, `rotate_index_test`, `merge_testing`, `merge_with_movement`, etc.
  - Smell: Plain functions meant for testing living in production modules; not executed by `cargo test`.
  - Approach: Move into `#[cfg(test)] mod tests` blocks or separate `tests/` files; assert on expected outcomes; add tests for `rotate_board`, `rotate_index`, `process_row`, `is_neighbours_mergeable`, and `acquire_empty_tile` new `Option` API.
  - Acceptance:
    - `cargo test` runs and passes new tests; remove or gate old test helpers from prod builds.
  - Status notes: Several unit tests now exist in `game_logic.rs` (`rotate_test`, `rotate_index_test`, `get_direction_test`, `acquire_empty_tile_test`). Audit still needed for ad-hoc tests remaining in `mod.rs` and to add tests for `process_row` and merging helpers.

- [TODO] Improve message handling contracts
  - Locations:
    - `process_direction_messages` reads and then clears messages; takes the last one; comments around ordering.
  - Smell: Implicit assumptions about message ordering may lead to dropped inputs.
  - Approach: Consider using events with buffering or processing all inputs per frame in deterministic way; document contract.
  - Acceptance:
    - Clear documentation on how multiple key presses in one frame are handled; tests covering it.
  - Status notes: Current implementation still reads `.last()` and ignores earlier inputs (see `process.rs` 20–24). No documentation/tests yet.

---

### P3 — Performance and Polish

- [TODO] Reduce per-frame allocations in `collision_system`
  - Location: `src/game/systems/game_logic.rs` lines ~166–179 (`collect::<Vec<_>>()`, `Vec` reuse).
  - Smell: Allocates new `Vec`s each frame in Movement state.
  - Approach: Use `Local<Vec<_>>` buffers to reuse allocations across frames; consider resource pool.
  - Acceptance:
    - No repeated allocations seen in profiles; code uses `Local`.
  - Status notes: Currently allocates `tree_nodes`, `collisions`, and `processed` each run (see 173–184). No `Local` buffers.

- [TODO] Cache spatial index where useful
  - Location: same as above
  - Smell: Rebuilds `RTree` every frame.
  - Approach: If the number of entities is small this may be fine; otherwise, experiment with maintaining an index resource updated by movement.
  - Acceptance:
    - Documented decision: either cached index with correctness verified, or justification to keep simple rebuild due to small N.
  - Status notes: Still rebuilding `RTree` every frame via `bulk_load` (169–182). No documented decision yet.

- [TODO] Logging hygiene and configurability
  - Location: `src/main.rs` logging filters
  - Smell: Hard-coded log filters; not easily configurable without recompile.
  - Approach: Read from env var (e.g., `RUST_LOG` or custom) with sane defaults.
  - Acceptance:
    - Log level/filter configurable via env at runtime.
  - Status notes: Uses hard-coded filters in `LogPlugin` (main.rs 21–36). No env-driven configuration yet.

---

### Cross-cutting: Documentation and Comments

  - Locations: `game/mod.rs` (double spawn), `game/systems/movement.rs` (WARN), `game/systems/game_logic.rs` (collision, bounds).
  - Acceptance: Each TODO either resolved or referenced by a task ID in this file.
  - Status notes: Some TODOs remain (e.g., effects/win-lose handling in `mod.rs` 108–110). Double-spawn TODO not closed with a documented fix.

---

### Suggested Task Ordering (quick wins first)

1) P0: Despawn/removal safety in `queued_movement_system` (movement)
2) P0: Remove panics/unwraps in runtime systems; `get_direction -> Option`
3) P0: `acquire_empty_tile -> Option`, handle full board; fix double spawn
4) P1: Coordinate magic numbers -> `BoardLayout` resource
5) P1: Collision via AABB; remove `0.96` literal and document epsilon
6) P2: Extract modules from `game/mod.rs`; convert test helpers into `cargo test`
7) P3: Reduce allocations in collision; logging via env

---

### Notes and References

- Files reviewed:
  - `src/main.rs`
  - `src/game/mod.rs`
  - `src/game/components.rs`
  - `src/game/effects.rs`
  - `src/game/utils.rs`
  - `src/game/systems/{game_logic.rs,process.rs,movement.rs,input.rs,effect.rs}`
  - Assets and shaders not analyzed for smells in this pass.
