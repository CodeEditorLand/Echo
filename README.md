<table>
	<tr>
		<td align="left" valign="middle">
			<h3 align="left"> Echo</h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left">
				📣
			</h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left"> + </h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left">
				<a href="https://Editor.Land" target="_blank">
					<picture>
						<source media="(prefers-color-scheme: dark)" srcset="https://PlayForm.Cloud/Dark/Image/GitHub/Land.svg">
						<source media="(prefers-color-scheme: light)" srcset="https://PlayForm.Cloud/Image/GitHub/Land.svg">
						<img width="28" alt="Land Logo" src="https://PlayForm.Cloud/Image/GitHub/Land.svg">
					</picture>
				</a>
			</h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left">
				<a href="https://Editor.Land" target="_blank">
					Land
				</a>
			</h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left">
				🏞️
			</h3>
		</td>
	</tr>
</table>

---

# **Echo**&#x2001;📣

A Resilient, High-Performance Task Scheduler for Rust

> **VS Code's background tasks (file indexing, symbol scanning, git blame) run
> in a single-threaded Node.js process. Heavy indexing freezes everything on
> that thread. The only escape is spawning more processes, adding memory and IPC
> overhead.**

_"Indexing, search, and builds run on every CPU core in parallel. The editor
stays responsive."_

[![License: CC0-1.0](https://img.shields.io/badge/License-CC0_1.0-lightgrey.svg)](https://github.com/CodeEditorLand/Echo/tree/Current/LICENSE)
[<img src="https://editor.land/Image/Rust.svg" width="14" alt="Rust" />](https://www.rust-lang.org/)&#x2001;[![Crates.io](https://img.shields.io/crates/v/Echo.svg)](https://crates.io/crates/Echo)
[<img src="https://editor.land/Image/Rust.svg" width="14" alt="Rust" />](https://www.rust-lang.org/)&#x2001;[![Rust Version](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Tokio Version](https://img.shields.io/badge/Tokio-v1-blue.svg)](https://tokio.rs/)
[![Crossbeam Version](https://img.shields.io/badge/Crossbeam-blueviolet.svg)](https://github.com/crossbeam-rs/crossbeam)

📖 **[Rust API Documentation](https://Rust.Documentation.Editor.Land/Echo/)**

Welcome to **Echo**! This crate provides a powerful, structured concurrency
runtime for Rust applications, built on a high-performance **work-stealing
scheduler**. It is designed to be the core execution engine for application
backends like `Mountain`, integrating seamlessly with declarative systems like
the `ActionEffect` pattern. **Echo** moves beyond simple task spawning
(`tokio::spawn`) to provide a robust framework for managing, prioritizing, and
executing complex asynchronous workflows with resilience and efficiency.

**Echo** is engineered to:

1.  **Provide High-Performance Concurrency:** Utilizes a lock-free,
    work-stealing queue (`crossbeam-deque`) to ensure all worker threads remain
    busy, maximizing CPU utilization and application throughput.
2.  **Enable Structured Task Management:** Offers a clean API for submitting
    tasks with different priorities, allowing critical, UI-blocking operations
    to pre-empt background work.
3.  **Integrate Natively with Effect Systems:** Designed from the ground up to
    be the execution backend for systems like the `ActionEffect` pattern,
    providing a bridge between declarative task definitions and their concrete
    execution.

---

## Key Features&#x2001;🔐

- **Work-Stealing Scheduler:** Implements a modern, priority-aware work-stealing
  algorithm to efficiently distribute tasks across a pool of worker threads.
- **Task Prioritization:** Supports submitting tasks with `High`, `Normal`, or
  `Low` priority, ensuring that latency-sensitive operations are handled
  immediately.
- **Fluent Builder API:** A clean `SchedulerBuilder` allows for easy
  configuration of the worker pool size.
- **Graceful Shutdown:** Provides a `Stop()` method to ensure all worker threads
  complete their current tasks and exit cleanly, preventing orphaned threads.
- **Decoupled Architecture:** A generic `Queue` module provides the core
  work-stealing logic, which is consumed by the application-specific
  `Scheduler`.

---

## Core Architecture Principles&#x2001;🏗️

| Principle                  | Description                                                                                                                                                     | Key Components Involved                                             |
| :------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------ |
| **Performance**            | Use lock-free data structures (`crossbeam-deque`) and a high-performance work-stealing algorithm to achieve maximum throughput and low-latency task execution.  | `Queue::StealingQueue`, `Scheduler::Worker`                         |
| **Structured Concurrency** | Manage all asynchronous operations within a supervised pool of workers, providing graceful startup and shutdown, unlike fire-and-forget `tokio::spawn`.         | `Scheduler::Scheduler`, `Scheduler::SchedulerBuilder`               |
| **Decoupling**             | Separate the generic **Queueing Logic** from the application-specific **Scheduler Implementation**. The scheduler uses the queue to run its tasks.              | `Queue::StealingQueue<TTask>`, `Scheduler::Scheduler`, `Task::Task` |
| **Resilience**             | The scheduler's design is inherently resilient; the failure of one task (if it panics) is contained within its `tokio` task and does not crash the worker pool. | `Scheduler::Worker::Run`                                            |
| **Composability**          | Provide a simple `Submit` API that accepts any `Future<Output = ()>`, making it easy to integrate with any asynchronous Rust code.                              | `Task::Task`, `Scheduler::Scheduler::Submit`                        |

---

## Deep Dive & Component Breakdown&#x2001;🔬

To understand how `Echo`'s internal components interact to provide these
services, please refer to the detailed technical breakdown in
[`Documentation/GitHub/DeepDive.md`](https://github.com/CodeEditorLand/Echo/tree/Current/Documentation/GitHub/DeepDive.md).
This document explains the roles of the `Task`, `StealingQueue`, `Worker`, and
`Scheduler` in detail.

---

## `Echo` in the Land Ecosystem&#x2001;📣 + 🏞️

This diagram illustrates `Echo`'s role as the core execution engine within the
`Mountain` backend.

```mermaid
graph LR
    classDef common fill:#9cf,stroke:#333,stroke-width:2px;
    classDef mountain fill:#f9f,stroke:#333,stroke-width:2px;
    classDef echo fill:#ffc,stroke:#333,stroke-width:2px;
    classDef rust fill:#f9d,stroke:#333,stroke-width:1px;

	subgraph "Common (Abstract Core)"
		ActionEffect["ActionEffect (Task Definition)"]:::common
	end

	subgraph "Mountain (Application Logic)"
        ApplicationRunTime["Mountain ApplicationRunTime"]:::mountain
        MountainEnvironment["MountainEnvironment (Service Impls)"]:::mountain
        Track["Track (Request Dispatcher)"]:::mountain
	end

	subgraph "Echo (Execution Engine)"
		Scheduler["Echo Scheduler"]:::echo
		WorkStealingQueue["Work-Stealing Queue"]:::echo
        WorkerPool["Worker Pool (Tokio Threads)"]:::rust

        Scheduler -- Manages --> WorkStealingQueue;
        Scheduler -- Spawns --> WorkerPool;
        WorkerPool -- Pull tasks from --> WorkStealingQueue;
	end

    Track -- Dispatches to --> ApplicationRunTime;
    ApplicationRunTime -- Creates Future from --> ActionEffect;
    ApplicationRunTime -- Submits Future to --> Scheduler;
    WorkerPool -- Executes Future using --> MountainEnvironment;
```

---

## Project Structure Overview&#x2001;🗺️

The `Echo` repository is organized into a few core modules with a clear
separation of concerns:

```
Echo/
└── Source/
    ├── Library.rs               # Crate root, declares all modules.
    ├── Scheduler/               # The main public API: Scheduler and SchedulerBuilder.
    ├── Queue/                   # The generic, high-performance work-stealing queue library.
    └── Task/                    # The concrete definition of a Task and its Priority.
```

---

## Getting Started&#x2001;🚀

### Installation&#x2001;📥

To add `Echo` to your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
Echo = { git = "https://github.com/CodeEditorLand/Echo.git", branch = "Current" }
```

**Key Dependencies:**

- `tokio = { version = "*", features = ["full"] }`
- `crossbeam-deque = "*"`
- `rand = "*"`
- `log = "*"`
- `num_cpus = "*"`

### Usage&#x2001;🚀

1.  **Initialize the Scheduler:** Create and start the scheduler when your
    application starts. It is typically wrapped in an `Arc` to be shared safely
    across your application.

```rust
use std::sync::Arc;
use Echo::Scheduler::SchedulerBuilder;
use Echo::Task::Priority;

let Scheduler = Arc::new(SchedulerBuilder::Create().WithWorkerCount(8).Build());
```

2.  **Submit Tasks:** Use the `Scheduler` instance to submit asynchronous work
    from anywhere in your application.

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

3.  **Graceful Shutdown:** Before your application exits, ensure a clean
    shutdown of all worker threads.

```rust
// Note: Arc::try_unwrap requires the Arc to have only one strong reference.
if let Ok(mut Scheduler) = Arc::try_unwrap(Scheduler) {
    Scheduler.Stop().await;
}
```

---

## Help Us Boost Performance: A Call for Contributions!&#x2001;🫱🏻‍🫲🏿

`Echo` is built on a high-performance foundation, but there's always room to
push the boundaries of speed and efficiency. We maintain a detailed roadmap of
features and performance optimizations, with tasks suitable for all skill
levels.

| Contribution Level | Example Tasks                                               |
| :----------------- | :---------------------------------------------------------- |
| **Quick Wins**     | Implement faster random number generation for stealing.     |
| **Architectural**  | Add a true sleep/notification system for idle workers.      |
| **Expert Tuning**  | Build a `criterion` benchmark suite; implement CPU pinning. |
| **Advanced Logic** | Introduce an anti-starvation mechanism for tasks.           |

**Interested in tackling one of these challenges?** 👉🏻

- **[Check out our full TODO](https://github.com/CodeEditorLand/Echo/tree/Current/Documentation/GitHub/Todo.md)**
  for challenges!
- **[Follow our Contribution Guide](https://github.com/CodeEditorLand/Echo/tree/Current/)**
  to get started!

---

## See Also

- [Echo Documentation](https://editor.land/Doc/echo)
- [Architecture Overview](https://editor.land/Doc/architecture)
- [Why Rust](https://editor.land/Doc/why-rust)
- [Mountain](https://github.com/CodeEditorLand/Mountain)
- [Common](https://github.com/CodeEditorLand/Common)

---

## License&#x2001;⚖️

This project is released into the public domain under the **Creative Commons CC0
Universal** license. You are free to use, modify, distribute, and build upon
this work for any purpose, without any restrictions. For the full legal text,
see the [`LICENSE`](https://github.com/CodeEditorLand/Echo/tree/Current/) file.

---

## Changelog&#x2001;📜

Stay updated with our progress! See
[`CHANGELOG.md`](https://github.com/CodeEditorLand/Echo/tree/Current/) for a
history of changes specific to **Echo**.

---

## Funding \& Acknowledgements&#x2001;🙏🏻

**Echo** is a core element of the **Land** ecosystem. This project is funded
through [NGI0 Commons Fund](https://NLnet.NL/commonsfund), a fund established by
[NLnet](https://NLnet.NL) with financial support from the European Commission's
[Next Generation Internet](https://ngi.eu) program. Learn more at the
[NLnet project page](https://NLnet.NL/project/Land).

The project is operated by PlayForm, based in Sofia, Bulgaria.

PlayForm acts as the open-source steward for Code Editor Land under the NGI0
Commons Fund grant.

<table>
	<thead>
		<tr>
			<th align="left"><strong>Land</strong></th>
			<th align="left"><strong>PlayForm</strong></th>
			<th align="left"><strong>NLnet</strong></th>
			<th align="left"><strong>NGI0 Commons Fund</strong></th>
		</tr>
	</thead>
	<tbody>
		<tr>
			<td align="left" valign="middle">
				<a href="https://Editor.Land">
					<img width="60" src="https://raw.githubusercontent.com/CodeEditorLand/Asset/refs/heads/Current/Logo/Land.svg" alt="Land">
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://PlayForm.Cloud">
					<img width="76" src="https://raw.githubusercontent.com/PlayForm/Asset/refs/heads/Current/Logo/PlayForm.svg" alt="PlayForm">
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://NLnet.NL">
					<img width="240" src="https://NLnet.NL/logo/banner.svg" alt="NLnet">
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://NLnet.NL/commonsfund">
					<img width="240" src="https://NLnet.NL/image/logos/NGI0CommonsFund_tag_black_mono.svg" alt="NGI0 Commons Fund">
				</a>
			</td>
		</tr>
	</tbody>
</table>

---

**Project Maintainers**: Source Open
([Source/Open@Editor.Land](mailto:Source/Open@Editor.Land)) |
[GitHub Repository](https://github.com/CodeEditorLand/Echo) |
[Report an Issue](https://github.com/CodeEditorLand/Echo/issues) |
[Security Policy](https://github.com/CodeEditorLand/Echo/security/policy)
