// @module queue
// @description This module provides the high-performance, concurrent queueing
// implementations used by the Echo scheduler. These are internal components
// of the library.
//

#![allow(non_snake_case, non_camel_case_types)]

pub mod StealingQueue;

// Re-exports the `StealingQueue` for use within the `Echo` crate, but keeps it
// private from external consumers.
// @see StealingQueue
//
// pub(crate) use self::StealingQueue::StealingQueue;
