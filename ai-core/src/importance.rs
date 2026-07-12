//! File importance scorer
//! Scores every file 0.0-1.0 based on access patterns.
//! This score drives: desktop surfacing, storage tiering,
//! prefetch priority, and search result ranking.

#![allow(unused_imports)]

use crate::{HashMap, kernel};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;

/// Storage tier — where a file lives based on importance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageTier {
    Hot,   // NVMe — accessed constantly
    Warm,  // SSD — accessed regularly
    Cold,  // HDD — rarely accessed
}

impl StorageTier {
    pub fn label(&self) -> &str {
        match self {
            StorageTier::Hot  => "🔥 HOT",
            StorageTier::Warm => "🌤 WARM",
            StorageTier::Cold => "🧊 COLD",
        }
    }
}

/// Score for a single file
#[derive(Debug, Clone)]
pub struct FileScore {
    pub ino: u64,
    pub name: String,
    pub score: f32,          // 0.0 to 1.0
    pub access_count: u32,
    pub last_access: u64,    // unix seconds
    pub tier: StorageTier,
}

impl FileScore {
    pub fn tier_from_score(score: f32) -> StorageTier {
        if score >= 0.6 { StorageTier::Hot }
        else if score >= 0.3 { StorageTier::Warm }
        else { StorageTier::Cold }
    }
}

/// The importance engine
pub struct ImportanceEngine {
    // ino -> (name, access_count, last_access_secs, total_open_secs)
    pub stats: HashMap<u64, (String, u32, u64, u64)>,
}

pub const MAX_TRACKED_FILES: usize = 10_000;

impl ImportanceEngine {
    pub fn new() -> Self {
        Self { stats: HashMap::new() }
    }

    /// Record an access to a file
    ///
    /// # Parameters
    /// - `ino`: File inode number
    /// - `name`: File name
    /// - `open_duration_secs`: How long the file was open
    /// - `now`: Current time in seconds since UNIX epoch
    pub fn record_access(&mut self, ino: u64, name: &str, open_duration_secs: u64, now: u64) {
        // Cap: if at limit and this is a new file, evict the lowest-scored entry
        if !self.stats.contains_key(&ino) && self.stats.len() >= MAX_TRACKED_FILES {
            // Find and remove the lowest scored entry
            let to_remove = self.stats.iter()
                .map(|(k, _)| (*k, self.score_with_time(*k, now)))
                .min_by(|a, b| {
                    // Use total_cmp for proper f32 comparison in no_std
                    a.1.total_cmp(&b.1)
                })
                .map(|(k, _)| k);
            if let Some(k) = to_remove {
                self.stats.remove(&k);
            }
        }

        let entry = self.stats.entry(ino).or_insert((name.to_string(), 0, 0, 0));
        entry.1 += 1;                          // increment access count
        entry.2 = now;                         // update last access
        entry.3 += open_duration_secs;         // accumulate open time
    }

    /// Record an access using the kernel time source.
    ///
    /// This is the convenience method for use in the kernel where the kernel
    /// interface is initialized. In tests, use `record_access` with explicit time.
    pub fn record_access_kernel(&mut self, ino: u64, name: &str, open_duration_secs: u64) {
        let now = kernel().current_time_secs();
        self.record_access(ino, name, open_duration_secs, now);
    }

    /// Score a file — combines recency + frequency + open time
    /// Returns 0.0 (cold/unimportant) to 1.0 (hot/critical)
    ///
    /// # Parameters
    /// - `ino`: File inode number
    /// - `now`: Current time in seconds since UNIX epoch
    pub fn score_with_time(&self, ino: u64, now: u64) -> f32 {
        let (_, count, last_access, open_secs) = match self.stats.get(&ino) {
            Some(s) => s,
            None => return 0.0,
        };

        // Recency score — decays over time
        // 1.0 if accessed just now, 0.0 if not accessed in 30 days
        let age_secs = now.saturating_sub(*last_access) as f32;
        let recency = (1.0 - (age_secs / (30.0 * 86400.0))).max(0.0);

        // Frequency score — log scale so 100 accesses isn't 100x better than 10
        // Use libm for ln() in no_std (logf is natural logarithm)
        let frequency = libm::logf(*count as f32).max(0.0) / libm::logf(10.0);
        let frequency = frequency.min(1.0);

        // Engagement score — time spent with file matters
        let engagement = (*open_secs as f32 / 3600.0).min(1.0); // cap at 1 hour

        // Weighted combination
        let score = (recency * 0.4) + (frequency * 0.4) + (engagement * 0.2);
        score.min(1.0)
    }

    /// Score a file using the kernel time source.
    ///
    /// This is the convenience method for use in the kernel where the kernel
    /// interface is initialized. In tests, use `score_with_time` with explicit time.
    pub fn score(&self, ino: u64) -> f32 {
        let now = kernel().current_time_secs();
        self.score_with_time(ino, now)
    }

    /// Get scored + ranked list of all files
    /// This is what drives the "desktop" surface
    pub fn ranked_files(&self) -> Vec<FileScore> {
        let now = kernel().current_time_secs();
        self.ranked_files_with_time(now)
    }

    /// Get scored + ranked list of all files with explicit time
    pub fn ranked_files_with_time(&self, now: u64) -> Vec<FileScore> {
        let mut scores: Vec<FileScore> = self.stats.iter()
            .map(|(ino, (name, count, last_access, _))| {
                let score = self.score_with_time(*ino, now);
                FileScore {
                    ino: *ino,
                    name: name.clone(),
                    score,
                    access_count: *count,
                    last_access: *last_access,
                    tier: FileScore::tier_from_score(score),
                }
            })
            .collect();

        scores.sort_by(|a, b| b.score.total_cmp(&a.score));
        scores
    }

    /// What tier should this file be on?
    pub fn tier(&self, ino: u64) -> StorageTier {
        FileScore::tier_from_score(self.score(ino))
    }

    /// What tier should this file be on? (with explicit time)
    pub fn tier_with_time(&self, ino: u64, now: u64) -> StorageTier {
        FileScore::tier_from_score(self.score_with_time(ino, now))
    }

    /// Files important enough to surface on "desktop"
    pub fn desktop_files(&self, limit: usize) -> Vec<FileScore> {
        let now = kernel().current_time_secs();
        self.desktop_files_with_time(limit, now)
    }

    /// Files important enough to surface on "desktop" with explicit time
    pub fn desktop_files_with_time(&self, limit: usize, now: u64) -> Vec<FileScore> {
        self.ranked_files_with_time(now)
            .into_iter()
            .filter(|f| f.score >= 0.3)
            .take(limit)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_increases_with_access() {
        let mut engine = ImportanceEngine::new();
        let now = 1000000;
        engine.record_access(2, "main.rs", 0, now);
        let score1 = engine.score_with_time(2, now);

        for _ in 0..20 {
            engine.record_access(2, "main.rs", 60, now);
        }
        let score2 = engine.score_with_time(2, now);

        assert!(score2 > score1);
    }

    #[test]
    fn test_unknown_file_scores_zero() {
        let engine = ImportanceEngine::new();
        let now = 1000000;
        assert_eq!(engine.score_with_time(999, now), 0.0);
    }

    #[test]
    fn test_tier_assignment() {
        let mut engine = ImportanceEngine::new();
        let now = 1000000;
        // High frequency = hot
        for _ in 0..50 {
            engine.record_access(2, "hot.rs", 120, now);
        }
        assert_eq!(engine.tier_with_time(2, now), StorageTier::Hot);

        // Never accessed = cold
        assert_eq!(engine.tier_with_time(999, now), StorageTier::Cold);
    }

    #[test]
    fn test_ranked_files_sorted() {
        let mut engine = ImportanceEngine::new();
        let now = 1000000;
        engine.record_access(2, "rarely.rs", 0, now);
        for _ in 0..30 {
            engine.record_access(3, "often.rs", 60, now);
        }

        let ranked = engine.ranked_files_with_time(now);
        assert_eq!(ranked[0].name, "often.rs");
    }

    #[test]
    fn test_desktop_files() {
        let mut engine = ImportanceEngine::new();
        let now = 1000000;
        for _ in 0..20 {
            engine.record_access(2, "important.rs", 30, now);
        }
        engine.record_access(3, "unimportant.rs", 0, now);

        let desktop = engine.desktop_files_with_time(10, now);
        assert!(desktop.iter().any(|f| f.name == "important.rs"));
    }
}
