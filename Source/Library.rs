//! A Resilient, High-Performance Task Scheduler.
//!
//! Provides a structured concurrency runtime for Rust applications, built on a
//! high-performance, priority-aware, work-stealing scheduler. It is designed
//! to be a robust and efficient core execution engine.

#![allow(non_snake_case, non_camel_case_types)]

// --- Crate Modules ---
// Declares the main modules that constitute the library.
pub mod Queue;

pub mod Scheduler;

pub mod Task;
