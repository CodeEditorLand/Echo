//! # Task Module
//!
//! Declares the constituent modules of a `Task`. A schedulable task is composed
//! of its concrete `Task` definition (the operation to be performed) and its
//! `Priority` level, which guides the scheduler's execution order.

#![allow(non_snake_case, non_camel_case_types)]

/// Defines the execution priority level of a task.
pub mod Priority;

/// Defines the structure of a task to be executed.
pub mod Task;
