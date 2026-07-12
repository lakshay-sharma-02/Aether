//! Access logger — the foundation of everything AI in the kernel
//!
//! Every file open/close/write gets recorded here.
//! This log is what the Markov chain, importance scorer,
//! and semantic search all learn from.
//!
//! ## no_std Port Notes
//!
//! - Replaced `std::collections::VecDeque` with `alloc::collections::VecDeque`
//! - Replaced `std::time::SystemTime` with kernel time parameter
//! - `AccessEvent::now()` becomes `AccessEvent::new()` with explicit time param

use alloc::collections::VecDeque;
use alloc::string::String;

/// A single file access event
#[derive(Debug, Clone)]
pub struct AccessEvent {
    pub ino: u64,
    pub name: String,
    pub kind: AccessKind,
    pub timestamp: u64,        // unix seconds
    pub duration_secs: u64,    // how long file was open (0 if unknown)
    pub size_bytes: u64,       // file size at time of access
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccessKind {
    Open,
    Write,
    Read,
    Close,
    Delete,
}

impl AccessEvent {
    /// Create a new access event with explicit timestamp.
    ///
    /// In kernel context, get the timestamp from `ai_core::kernel().current_time_secs()`.
    pub fn new(ino: u64, name: &str, kind: AccessKind, size_bytes: u64, timestamp: u64) -> Self {
        Self {
            ino,
            name: String::from(name),
            kind,
            timestamp,
            duration_secs: 0,
            size_bytes,
        }
    }

    /// Was this access recent? (within last N seconds)
    pub fn is_recent(&self, within_secs: u64, now: u64) -> bool {
        now.saturating_sub(self.timestamp) <= within_secs
    }

    /// Was this access yesterday?
    pub fn is_yesterday(&self, now: u64) -> bool {
        let age = now.saturating_sub(self.timestamp);
        age >= 86400 && age < 172800 // between 24h and 48h ago
    }

    /// Was this access today?
    pub fn is_today(&self, now: u64) -> bool {
        self.is_recent(86400, now)
    }
}

/// The access log — bounded size, most recent events
pub struct AccessLog {
    events: VecDeque<AccessEvent>,
    max_events: usize,
}

impl AccessLog {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: VecDeque::new(),
            max_events,
        }
    }

    /// Record a file access
    pub fn record(&mut self, event: AccessEvent) {
        if self.events.len() >= self.max_events {
            self.events.pop_front(); // drop oldest
        }
        self.events.push_back(event);
    }

    /// Get all events for a specific file
    pub fn events_for(&self, ino: u64) -> alloc::vec::Vec<&AccessEvent> {
        self.events.iter().filter(|e| e.ino == ino).collect()
    }

    /// Get recent open events in order (for Markov chain)
    pub fn recent_opens(&self, limit: usize) -> alloc::vec::Vec<&AccessEvent> {
        self.events.iter()
            .filter(|e| e.kind == AccessKind::Open)
            .rev()
            .take(limit)
            .collect()
    }

    /// Get all events from yesterday
    pub fn yesterday(&self, now: u64) -> alloc::vec::Vec<&AccessEvent> {
        self.events.iter().filter(|e| e.is_yesterday(now)).collect()
    }

    /// Get all events from today
    pub fn today(&self, now: u64) -> alloc::vec::Vec<&AccessEvent> {
        self.events.iter().filter(|e| e.is_today(now)).collect()
    }

    /// How many times has this file been accessed?
    pub fn access_count(&self, ino: u64) -> usize {
        self.events.iter().filter(|e| e.ino == ino).count()
    }

    /// Last access time for a file
    pub fn last_access(&self, ino: u64) -> Option<u64> {
        self.events.iter()
            .filter(|e| e.ino == ino)
            .map(|e| e.timestamp)
            .max()
    }

    pub fn all_events(&self) -> &VecDeque<AccessEvent> {
        &self.events
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_records_and_retrieves() {
        let mut log = AccessLog::new(1000);
        let now = 1000000;
        log.record(AccessEvent::new(2, "main.rs", AccessKind::Open, 1024, now));
        log.record(AccessEvent::new(3, "lib.rs", AccessKind::Open, 512, now + 1));
        log.record(AccessEvent::new(2, "main.rs", AccessKind::Write, 1024, now + 2));

        assert_eq!(log.access_count(2), 2);
        assert_eq!(log.access_count(3), 1);
        assert_eq!(log.len(), 3);
    }

    #[test]
    fn test_bounded_size() {
        let mut log = AccessLog::new(5);
        let now = 1000000;
        for i in 0..10 {
            log.record(AccessEvent::new(i, "file", AccessKind::Open, 0, now + i));
        }
        assert_eq!(log.len(), 5);
    }

    #[test]
    fn test_today_filter() {
        let mut log = AccessLog::new(100);
        let now = 1000000;
        log.record(AccessEvent::new(2, "recent.rs", AccessKind::Open, 0, now));
        assert_eq!(log.today(now).len(), 1);
    }
}
