// @module task
// @description This module defines the core data structures for the Echo
// scheduler, including the `Task` itself and its `Priority`.
//

#![allow(non_snake_case, non_camel_case_types)]

// --- Sub-modules ---
pub mod Priority;
pub mod Task;

// --- Public Re-exports ---

// The enum representing the priority of a task.
// @see Priority
//
// pub use self::Priority::Priority;
// The struct representing a single unit of work for the scheduler.
// @see Task
// pub use self::Task::Task;
