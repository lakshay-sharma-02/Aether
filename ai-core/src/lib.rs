//! AI-Core — Aether's kernel-level AI subsystem
//!
//! This is the AI layer that runs in ring 0 as a first-class kernel subsystem,
//! not a userspace application. It provides:
//!
//! - File access prediction (Markov chains + neural prefetcher)
//! - Importance scoring and storage tiering
//! - Semantic search (TF-IDF)
//! - Ransomware detection (entropy analysis)
//! - Cross-session memory and workspace intelligence
//! - Proactive suggestions (Jarvis)
//!
//! ## no_std Design
//!
//! This crate is `no_std` and designed to run in the kernel without any
//! standard library dependencies. All heap allocations go through the
//! kernel's allocator interface.
//!
//! ## Architecture
//!
//! ```text
//! Kernel Events (file open, write, close)
//!     ↓
//! AICore::on_file_*() methods
//!     ↓
//! Individual AI modules (logger, markov, importance, etc.)
//!     ↓
//! Predictions and insights returned to kernel
//! ```

#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;

// Re-export hashbrown as our HashMap/HashSet implementation
pub use hashbrown::{HashMap, HashSet};

// ───────────────────────────────────────────────────────────────────────────
// Kernel Interface Traits
// ───────────────────────────────────────────────────────────────────────────

/// Interface that the kernel must implement to support AI-Core.
///
/// The kernel provides timing, memory allocation, and (future) I/O services
/// to the AI subsystem.
pub trait KernelInterface {
    /// Get current time in seconds since UNIX epoch (from RTC or TSC).
    fn current_time_secs(&self) -> u64;

    /// Allocate memory (forwards to kernel allocator).
    ///
    /// Returns null pointer on allocation failure.
    unsafe fn alloc(&self, size: usize, align: usize) -> *mut u8;

    /// Deallocate memory (forwards to kernel allocator).
    unsafe fn dealloc(&self, ptr: *mut u8, size: usize, align: usize);
}

// ───────────────────────────────────────────────────────────────────────────
// Global Allocator Setup
// ───────────────────────────────────────────────────────────────────────────

/// Global allocator that delegates to the kernel.
///
/// The kernel must set this via `set_kernel_allocator()` during early boot.
static mut KERNEL_ALLOCATOR: Option<&'static dyn KernelInterface> = None;

/// Set the kernel allocator interface.
///
/// Must be called once during kernel initialization before any AI-Core
/// functions are used.
///
/// # Safety
///
/// - Must be called exactly once during kernel init
/// - Must be called before any allocations occur
/// - The provided reference must remain valid for the lifetime of the kernel
pub unsafe fn set_kernel_interface(kernel: &'static dyn KernelInterface) {
    KERNEL_ALLOCATOR = Some(kernel);
}

/// Get the current kernel interface.
///
/// Panics if called before `set_kernel_interface()`.
#[inline]
pub fn kernel() -> &'static dyn KernelInterface {
    unsafe {
        KERNEL_ALLOCATOR.expect("AI-Core: kernel interface not initialized")
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Custom Allocator Implementation
// ───────────────────────────────────────────────────────────────────────────

use core::alloc::{GlobalAlloc, Layout};

struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match KERNEL_ALLOCATOR {
            Some(k) => k.alloc(layout.size(), layout.align()),
            None => core::ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(k) = KERNEL_ALLOCATOR {
            k.dealloc(ptr, layout.size(), layout.align());
        }
    }
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator;

#[cfg(not(test))]
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("AI-Core: allocation failed: size={}, align={}", layout.size(), layout.align());
}

// ───────────────────────────────────────────────────────────────────────────
// Panic Handler
// ───────────────────────────────────────────────────────────────────────────

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // In a real kernel, this would log to serial/VGA and halt
    // For now, just halt the CPU
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Module Declarations
// ───────────────────────────────────────────────────────────────────────────

pub mod logger;
pub mod markov;
pub mod importance;
pub mod search;

// Modules to be added:
// pub mod entropy;
// pub mod neural;
// pub mod memory;
// pub mod workspace;
// pub mod jarvis;
// pub mod ai;

// ───────────────────────────────────────────────────────────────────────────
// Exports
// ───────────────────────────────────────────────────────────────────────────

// Re-export commonly used types
pub use logger::{AccessLog, AccessEvent, AccessKind};
pub use markov::MarkovPrefetcher;
pub use importance::{ImportanceEngine, FileScore, StorageTier};
pub use search::{SearchIndex, IndexedFile, SearchResult};

// Main AI subsystem will be exported here after all modules are ported
// pub use ai::AICore;

// ───────────────────────────────────────────────────────────────────────────
// Version Info
// ───────────────────────────────────────────────────────────────────────────

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "Aether AI-Core";

/// Returns the AI-Core version string.
pub fn version() -> &'static str {
    VERSION
}
