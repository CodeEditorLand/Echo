// @module Echo Crate
// @description A high-performance, structured concurrency and task scheduling
// library for Rust, built on top of `tokio` and a work-stealing queue. Echo is
// designed to natively execute `Future`s, providing a robust runtime for
// complex, asynchronous applications.
//

#![allow(non_snake_case, non_camel_case_types)]

// --- Public API ---
pub mod scheduler;
pub mod task;

// --- Internal Implementation ---
mod queue;
