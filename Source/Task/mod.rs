//! Declares the constituent modules of a `Task`.
//!
//! A task is composed of its concrete `Task` definition and its `Priority`.

#![allow(non_snake_case, non_camel_case_types)]

/// Defines the execution priority level of a task.
pub mod Priority;

/// Defines the structure of a task to be executed.
pub mod Task;
