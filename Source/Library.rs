#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

//! # Echo: A Resilient, High-Performance Task Scheduler
//!
//! Provides a structured concurrency runtime for Rust applications, built on a
//! high-performance, priority-aware, work-stealing scheduler. It is designed
//! to be a robust and efficient core execution engine for demanding,
//! concurrent workloads.

// --- Crate Modules ---
// Declares the main modules that constitute the library.
pub mod Queue;

pub mod Scheduler;

pub mod Task;
