# **Echo** 🔊

A Resilient, High-Performance Task Scheduler for Rust

Echo is a bounded work-stealing task scheduler designed as the core execution engine for application backends like Mountain. It provides structured concurrency with priority-based task scheduling, ensuring that latency-sensitive operations always take precedence over background work. The scheduler distributes tasks across a Tokio thread pool using lock-free work-stealing queues, maximizing CPU utilization without the memory and IPC overhead of process-based parallelism.

The system separates generic queueing logic from application-specific scheduling concerns through a clean abstraction layer. Workers consume tasks from local FIFO deques and attempt to steal from peer workers or the global injector queue when local work is exhausted. This architecture delivers low-latency execution for foreground operations while efficiently processing batch work in the background.

[![License: CC0-1.0](https://img.shields.io/badge/License-CC0_1.0-lightgrey.svg)](https://github.com/CodeEditorLand/Echo/tree/Current/LICENSE) [<img src="https://editor.land/Image/Rust.svg" width="14" alt="Rust" />](https://www.rust-lang.org/) [![Crates.io](https://img.shields.io/crates/v/Echo.svg)](https://crates.io/crates/Echo) [<img src="https://editor.land/Image/Rust.svg" width="14" alt="Rust" />](https://www.rust-lang.org/) [![Rust Version](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/) [![Tokio Version](https://img.shields.io/badge/Tokio-v1-blue.svg)](https://tokio.rs/) [![Crossbeam Version](https://img.shields.io/badge/Crossbeam-blueviolet.svg)](https://github.com/crossbeam-rs/crossbeam)

**[Rust API Documentation](https://Rust.Documentation.editor.land/Echo/)** 📖

Welcome to **Echo**! This crate provides a structured concurrency runtime for Rust applications, built on a high-performance work-stealing scheduler. It is designed to be the core execution engine for application backends like `Mountain`, integrating seamlessly with declarative systems like the `ActionEffect` pattern. Echo moves beyond simple task spawning to provide a robust framework for managing, prioritizing, and executing complex asynchronous workflows with resilience and efficiency.

---

## Overview

Echo provides a bounded work-stealing task scheduler that acts as the core execution engine for Rust application backends. It solves the problem of unstructured concurrency by offering structured, priority-based task scheduling with graceful shutdown, ensuring that latency-sensitive operations always take precedence over background work while maximizing CPU utilization across a pool of worker threads. The crate moves beyond simple `tokio::spawn` to provide a robust framework for managing, prioritizing, and executing complex asynchronous workflows with resilience and efficiency.

### Key Features 🔐

- **Work-Stealing Scheduler:** Implements a priority-aware work-stealing algorithm using `crossbeam-deque` to efficiently distribute tasks across a pool of worker threads. Idle workers automatically steal from busy workers, eliminating scheduling bottlenecks and keeping all cores productive.
- **Task Prioritization:** Supports submitting tasks with `High`, `Normal`, or `Low` priority levels. High-priority tasks are executed first from local and global deques, ensuring that latency-sensitive operations respond immediately while background work yields gracefully.
- **Fluent Builder API:** The `SchedulerBuilder` provides a clean, chainable configuration interface for the worker pool. It defaults to the number of logical CPU cores with a minimum of two workers, and supports explicit worker count overrides as well as named queue configuration for future extensibility.
- **Graceful Shutdown:** The `Stop()` method signals all worker threads to terminate and waits for each to complete its current task before joining. An automatic `Drop` guard ensures workers are signaled to stop even if the scheduler is dropped without an explicit shutdown call.
- **Decoupled Architecture:** The generic `Queue` module provides the core work-stealing logic as a standalone library, independent of any specific scheduler implementation. The `StealingQueue<TTask>` accepts any type implementing the `Prioritized` trait, making it reusable across projects.

---

## Architecture

```mermaid
graph LR
    classDef common   fill:#d4f5d4,stroke:#27ae60,stroke-width:1px,stroke-dasharray:5 5,color:#0a3a0a;
    classDef mountain fill:#f0d0ff,stroke:#9b59b6,stroke-width:2px,color:#2c0050;
    classDef echo     fill:#fffde0,stroke:#f0b429,stroke-width:2px,color:#4a3500;
    classDef worker   fill:#ffe0f0,stroke:#c0396a,stroke-width:1px,color:#4a0020;

    subgraph COMMON["Common - Abstract Core"]
        ActionEffect["ActionEffect\n(operation as value)"]:::common
        Prioritized["Prioritized trait\n(High / Normal / Low)"]:::common
    end

    subgraph MOUNTAIN["Mountain ⛰️ - Application Logic"]
        Track["Track/ - Request Dispatcher"]:::mountain
        AppRunTime["ApplicationRunTime\n(RunTime/ApplicationRunTime/)"]:::mountain
        MountainEnv["Environment/ Providers\n(concrete service impls)"]:::mountain
        Track --> AppRunTime
    end

    subgraph ECHO["Echo 📣 - Work-Stealing Scheduler"]
        direction TB
        subgraph SCHEDULER["Scheduler/"]
            SchedBuilder["SchedulerBuilder.rs\n(fluent config, defaults to num_cpus)"]:::echo
            SchedCore["Scheduler.rs\n(Submit API + graceful Stop)"]:::echo
            Workers["Worker.rs\n(Tokio threads, steal-on-idle)"]:::worker
            SchedBuilder --> SchedCore
            SchedCore --> Workers
        end
        subgraph QUEUE["Queue/"]
            StealQ["StealingQueue.rs\n(crossbeam-deque, lock-free)"]:::echo
        end
        subgraph TASK["Task/"]
            TaskDef["Task.rs + Priority.rs\n(Future wrapper + priority level)"]:::echo
        end

        Workers -- steals from --> StealQ
        SchedCore -- enqueues --> StealQ
        TaskDef -.implements.-> Prioritized
    end

    AppRunTime -- creates Future from --> ActionEffect
    AppRunTime -- Submit Future --> SchedCore
    Workers -- executes using --> MountainEnv
```

This diagram illustrates Echo's role as the core execution engine within the `Mountain` backend.

---

## Key Components

| Component | Path | Description |
| --------- | ---- | ----------- |
| Library Entry | `Source/Library.rs` | Crate root, declares all modules. |
| Scheduler | `Source/Scheduler/` | The main public API: Scheduler and SchedulerBuilder. |
| Queue | `Source/Queue/` | The generic, high-performance work-stealing queue library. |
| Task | `Source/Task/` | The concrete definition of a Task and its Priority. |

---

## Core Architecture Principles 🏗️

| Principle | Description | Key Components Involved |
| :------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------ |
| **Performance** | Use lock-free data structures (`crossbeam-deque`) and a high-performance work-stealing algorithm to achieve maximum throughput and low-latency task execution. | `Queue::StealingQueue`, `Scheduler::Worker` |
| **Structured Concurrency** | Manage all asynchronous operations within a supervised pool of workers, providing graceful startup and shutdown, unlike fire-and-forget `tokio::spawn`. | `Scheduler::Scheduler`, `Scheduler::SchedulerBuilder` |
| **Decoupling** | Separate the generic **Queueing Logic** from the application-specific **Scheduler Implementation**. The scheduler uses the queue to run its tasks. | `Queue::StealingQueue<TTask>`, `Scheduler::Scheduler`, `Task::Task` |
| **Resilience** | The scheduler's design is inherently resilient; the failure of one task (if it panics) is contained within its `tokio` task and does not crash the worker pool. | `Scheduler::Worker::Run` |
| **Composability** | Provide a simple `Submit` API that accepts any `Future<Output = ()>`, making it easy to integrate with any asynchronous Rust code. | `Task::Task`, `Scheduler::Scheduler::Submit` |

---

## In the Land Project

Echo serves as the core execution engine for `Mountain`, the native Rust/Tauri backend of the Land Code Editor. It integrates seamlessly with the `ActionEffect` pattern from the `Common` crate, executing composed asynchronous workflows across a priority-aware worker pool. The `Mountain` runtime submits futures derived from `ActionEffect` values to the Echo scheduler, which distributes them across its workers alongside the concrete `Environment` provider implementations.

---

## Getting Started 🚀

### Installation 📥

To add Echo to your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
Echo = { git = "https://github.com/CodeEditorLand/Echo.git", branch = "Current" }
```

The crate depends on `tokio`, `crossbeam-deque`, `rand`, `log`, `num_cpus`, and `Common` from the Land workspace. All dependencies are resolved through the workspace `Cargo.toml` configuration.

### Usage 🚀

First, create and start the scheduler when your application initializes. The builder defaults to the number of logical CPU cores, with a minimum of two workers to ensure work-stealing is viable:

```rust
use std::sync::Arc;
use Echo::Scheduler::SchedulerBuilder;
use Echo::Task::Priority;

let Scheduler = Arc::new(SchedulerBuilder::Create().WithWorkerCount(8).Build());
```

Submit asynchronous tasks from anywhere in your application using the scheduler instance. Tasks are queued by priority and executed by the next available worker:

```rust
let MyTask = async {
    println!("This is running on an Echo worker thread!");
    // ... perform some work ...
};

// Submit the task with a desired priority
Scheduler.Submit(MyTask, Priority::Normal);

// Another example with high priority
Scheduler.Submit(async { /* critical work */ }, Priority::High);
```

Before your application exits, ensure a clean shutdown of all worker threads. The `Stop()` method drains the queue and waits for in-flight tasks to complete:

```rust
// Note: Arc::try_unwrap requires the Arc to have only one strong reference.
if let Ok(mut Scheduler) = Arc::try_unwrap(Scheduler) {
    Scheduler.Stop().await;
}
```

---

## API Reference

- [Rust API Documentation](https://Rust.Documentation.editor.land/Echo/) 📖

---

## Related Documentation

- [Architecture Overview](https://Editor.Land/Doc/architecture) — Internal module structure
- [Deep Dive](Documentation/GitHub/DeepDive.md) — In-depth technical details
- [Land Documentation](../../Documentation/GitHub/README.md) — Complete documentation index
- [`Mountain`](https://github.com/CodeEditorLand/Mountain) — Primary consumer of Echo
- [`Common`](https://github.com/CodeEditorLand/Common) — Abstract traits and ActionEffect system
- [Why Rust](https://Editor.Land/Doc/why-rust)
- [Contribution Guide](https://github.com/CodeEditorLand/Echo/tree/Current/CONTRIBUTING.md)
- [TODO List](https://github.com/CodeEditorLand/Echo/tree/Current/Documentation/GitHub/Todo.md) — Performance optimization challenges
- [`CHANGELOG.md`](https://github.com/CodeEditorLand/Echo/tree/Current/CHANGELOG.md) — History of changes specific to Echo

---

## Help Us Boost Performance: A Call for Contributions! 🫱🏻‍🫲🏿

Echo is built on a high-performance foundation, but there is always room to push the boundaries of speed and efficiency. We maintain a detailed roadmap of features and performance optimizations, with tasks suitable for all skill levels.

| Contribution Level | Example Tasks |
| :----------------- | :-------------------------------------------------------- |
| **Quick Wins** | Implement faster random number generation for stealing. |
| **Architectural** | Add a notification-based wake system for idle workers. |
| **Expert Tuning** | Build a criterion benchmark suite; implement CPU pinning. |
| **Advanced Logic** | Introduce an anti-starvation mechanism for tasks. |

**Interested in tackling one of these challenges?**

- **[Check out our full TODO](https://github.com/CodeEditorLand/Echo/tree/Current/Documentation/GitHub/Todo.md)** for challenges!
- **[Follow our Contribution Guide](https://github.com/CodeEditorLand/Echo/tree/Current/CONTRIBUTING.md)** to get started!

---

## Funding

This project is funded through [NGI0 Commons Fund](https://NLnet.NL/commonsfund), a fund established by [NLnet](https://NLnet.NL) with financial support from the European Commission's Next Generation Internet program, under grant agreement No 101135429.

Echo is a core element of the Land ecosystem. This project is funded through [NGI0 Commons Fund](https://NLnet.NL/commonsfund), a fund established by [NLnet](https://NLnet.NL) with financial support from the European Commission's [Next Generation Internet](https://ngi.eu) program. Learn more at the [NLnet project page](https://NLnet.NL/project/Land).

The project is operated by PlayForm, based in Sofia, Bulgaria. PlayForm acts as the open-source steward for Code Editor Land under the NGI0 Commons Fund grant.

| | |
| --- | --- |
| [![Land](https://raw.githubusercontent.com/CodeEditorLand/Asset/refs/heads/Current/Logo/Dual/Land.svg)](https://Editor.Land) | [![PlayForm](https://raw.githubusercontent.com/PlayForm/Asset/refs/heads/Current/Logo/PlayForm.svg)](https://PlayForm.Cloud) |
| [![NLnet](https://raw.githubusercontent.com/CodeEditorLand/Asset/refs/heads/Current/Logo/NLnet.svg)](https://NLnet.NL) | [![NGI0](https://raw.githubusercontent.com/CodeEditorLand/Asset/refs/heads/Current/Logo/NGI0.svg)](https://NLnet.NL/commonsfund) |
