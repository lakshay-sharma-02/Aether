# importance.rs Tests Fixed ✅

## Problem

The `importance.rs` module was calling `kernel().current_time_secs()` which panicked during tests because the kernel interface isn't initialized in test mode.

**Failing tests:**
- `test_score_increases_with_access`
- `test_tier_assignment`
- `test_ranked_files_sorted`
- `test_desktop_files`

**Error:** 4 failed out of 17 tests

## Solution

Applied the same pattern used throughout ai-core: provide both kernel-based and time-parameterized versions of methods.

### Changes Made

**1. `record_access()` → split into two methods:**
```rust
// Test-friendly: explicit time parameter
pub fn record_access(&mut self, ino: u64, name: &str, open_duration_secs: u64, now: u64)

// Kernel convenience: uses kernel().current_time_secs()
pub fn record_access_kernel(&mut self, ino: u64, name: &str, open_duration_secs: u64)
```

**2. `score()` → split into two methods:**
```rust
// Test-friendly: explicit time parameter
pub fn score_with_time(&self, ino: u64, now: u64) -> f32

// Kernel convenience: uses kernel().current_time_secs()
pub fn score(&self, ino: u64) -> f32
```

**3. `ranked_files()` → added time-parameterized version:**
```rust
pub fn ranked_files_with_time(&self, now: u64) -> Vec<FileScore>
pub fn ranked_files(&self) -> Vec<FileScore>  // calls ranked_files_with_time
```

**4. `desktop_files()` → added time-parameterized version:**
```rust
pub fn desktop_files_with_time(&self, limit: usize, now: u64) -> Vec<FileScore>
pub fn desktop_files(&self, limit: usize) -> Vec<FileScore>  // calls desktop_files_with_time
```

**5. `tier()` → added time-parameterized version:**
```rust
pub fn tier_with_time(&self, ino: u64, now: u64) -> StorageTier
pub fn tier(&self, ino: u64) -> StorageTier  // calls tier_with_time
```

**6. Updated all tests** to use `*_with_time()` methods with explicit `now = 1000000`.

## Test Results

```bash
$ cargo test -p ai-core --lib 2>&1 | grep -E "(running|passed|failed|FAILED)"
running 17 tests
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

✅ **All 17 tests passing**

## API Design Pattern

This establishes a consistent pattern for ai-core modules:

### For Kernel Use (with kernel interface):
```rust
use ai_core::{ImportanceEngine, kernel};

let mut engine = ImportanceEngine::new();
engine.record_access_kernel(ino, "file.rs", 60);
let score = engine.score(ino);
let tier = engine.tier(ino);
let desktop = engine.desktop_files(10);
```

### For Tests (explicit time):
```rust
let mut engine = ImportanceEngine::new();
let now = 1000000;
engine.record_access(ino, "file.rs", 60, now);
let score = engine.score_with_time(ino, now);
let tier = engine.tier_with_time(ino, now);
let desktop = engine.desktop_files_with_time(10, now);
```

## Modules Status

| Module | Tests | Status |
|--------|-------|--------|
| logger.rs | 3/3 | ✅ |
| markov.rs | 3/3 | ✅ |
| importance.rs | 5/5 | ✅ (fixed) |
| search.rs | 6/6 | ✅ |
| **Total** | **17/17** | ✅ **All passing** |

## Next Steps

Ready to continue porting remaining modules:
- entropy.rs
- neural.rs
- memory.rs
- workspace.rs
- jarvis.rs

All future modules will follow this pattern: provide `*_with_time()` versions for testing and kernel-based versions for convenience.

---

**Date:** 2026-06-03  
**Status:** importance.rs tests fixed ✅
