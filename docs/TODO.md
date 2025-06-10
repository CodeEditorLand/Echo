### **Help Us Boost Performance: A Call for Contributions!** 🚀

`Echo` is built on a high-performance foundation, but there's always room to
push the boundaries of speed and efficiency. We invite the community to help us
implement the following performance enhancements. These are fantastic
opportunities to learn about low-level systems programming and make a real
impact on the project.

---

### Level 1: Quick Wins & Low-Hanging Fruit ✅

These tasks are ideal for first-time contributors. They provide measurable
performance gains with low implementation complexity.

#### **TODO 1: Implement Faster Random Number Generation**

- **The Goal:** Replace the cryptographically secure (and slower) `rand::rng()`
  with a much faster non-cryptographic Pseudo-Random Number Generator (PRNG) for
  selecting steal victims.
- **The Problem:** The current `Steal` method in `Queue::StealingQueue` uses
  `rand::rng()`, which involves OS-level interaction and is overkill for our
  needs. This adds unnecessary overhead in a hot loop.
- **Proposed Solution:**
    1.  Add the `fastrand` crate as a dependency.
    2.  Modify the `Steal` method in `Echo/Source/Queue/StealingQueue.rs` to use
        `fastrand::usize(..)` for choosing the starting index for stealing.
    3.  Remove the `rand` crate dependency if it's no longer used elsewhere.
- **Impact:** Reduces system call overhead during steal attempts, improving
  performance when workers are contending for tasks.
- **Difficulty:** Low
- **Skills:** Basic Rust, dependency management.

---

### Level 2: Architectural Enhancements 🌍

This task involves a more significant change to the scheduler's core logic,
focusing on improving idle performance and latency.

#### **TODO 2: Implement True Worker Sleep with a Notification System**

- **The Goal:** Eliminate busy-waiting in idle workers. Instead of `sleep(1ms)`,
  workers should enter a deep, OS-level sleep and only be woken up when new work
  is available.
- **The Problem:** The current `tokio::time::sleep()` loop in an idle `Worker`
  consumes CPU cycles and introduces up to 1ms of latency for newly submitted
  tasks.
- **Proposed Solution:**
    1.  Introduce a `tokio::sync::Notify` primitive into the
        `Queue::StealingQueue::Share` struct.
    2.  In `Queue::StealingQueue::Submit()`, after a task is successfully pushed
        to an injector, call `Notifier.notify_one()` to wake up a single
        sleeping worker.
    3.  In `Scheduler::Worker::Run()`, replace the `else { sleep(...) }` block
        with a call to `await` the `Notifier.notified()` future.
- **Impact:** Drastically reduces CPU usage for an idle scheduler and minimizes
  latency for the first task submitted to an idle system. This is crucial for
  GUI applications and servers with bursty workloads.
- **Difficulty:** Medium
- **Skills:** `async` Rust, understanding of `tokio` synchronization primitives.

---

### Level 3: Expert-Level Tuning & Measurement ⚙️

These tasks are for experienced developers who are passionate about
systems-level performance, benchmarking, and hardware affinity.

#### **TODO 3: Establish a Comprehensive Benchmarking Suite**

- **The Goal:** Create a suite of benchmarks using the `criterion` crate to
  rigorously measure scheduler performance and validate the impact of
  optimizations.
- **The Problem:** Without benchmarks, we are "flying blind." We cannot prove
  that changes are actually improving performance.
- **Proposed Solution:**
    1.  Add `criterion = { version = "0.5", features = ["async_tokio"] }` as a
        `[dev-dependency]`.
    2.  Create a `benches/` directory at the root of the project.
    3.  Implement several benchmark scenarios in `benches/scheduler_bench.rs`,
        such as:
        - **Throughput:** Measure the time to submit and execute a massive
          number of tiny tasks (e.g., 1,000,000).
        - **Latency:** Measure the time from `Submit()` to completion for a
          single task on an idle scheduler.
        - **Contention:** Benchmark performance when all workers are heavily
          contending for tasks from the global queue.
- **Impact:** Provides the entire project with a tool to make data-driven
  performance decisions. This is foundational for any serious performance work.
- **Difficulty:** Medium
- **Skills:** Benchmarking practices, `criterion` usage, `async` benchmarking.

#### **TODO 4: Implement CPU Core Affinity (Thread Pinning)**

- **The Goal:** Allow the `Scheduler` to pin each of its worker threads to a
  specific CPU core.
- **The Problem:** On modern multi-socket servers (NUMA architecture), a thread
  running on one CPU that accesses memory allocated by another CPU suffers a
  significant performance penalty. OS thread scheduling can also migrate threads
  between cores, causing cache misses.
- **Proposed Solution:**
    1.  Add the `core_affinity` crate as a dependency.
    2.  In `Scheduler::Scheduler::Create()`, before spawning each `tokio` task,
        get the list of available core IDs.
    3.  Use `tokio::task::spawn_blocking` in conjunction with
        `core_affinity::set_for_current()` to pin the thread to a specific core
        ID before starting the `Worker::Run` async loop. This is a complex task
        that requires careful integration with Tokio's threading model.
- **Impact:** Can provide a massive performance boost on server-grade hardware
  by maximizing cache locality and eliminating NUMA cross-socket memory access
  penalties. This is an expert-level optimization for achieving bare-metal
  performance.
- **Difficulty:** High
- **Skills:** Deep understanding of OS schedulers, CPU architecture (NUMA), and
  the `tokio` runtime's threading model.

### **Level 4: Advanced Scheduling Logic & Fairness** 🧠

This level moves beyond raw speed and into the "intelligence" of the scheduler,
focusing on fairness and preventing common concurrency pitfalls.

#### **TODO 5: Implement LIFO Slot for Recently Awoken Tasks**

- **The Goal:** Improve the performance of "ping-pong" workloads, where a task
  awaits a short I/O operation and then immediately needs to run again.
- **The Problem:** When an `async` task completes an I/O operation (e.g., a
  database query), its `Waker` is called, and it gets re-submitted to the
  scheduler. This often pushes it to the back of a global queue, adding
  unnecessary latency.
- **Proposed Solution:**
    1.  Add a special, single-element "LIFO slot" to each `Worker`'s local
        state. This slot is separate from the `crossbeam-deque`.
    2.  When a task is awoken by its `Waker`, instead of being pushed to the
        global `Injector`, it is placed directly into the LIFO slot of the _same
        worker_ that was running it before.
    3.  Modify the `Worker::Run` loop to check this LIFO slot _before_ checking
        its main local deques.
- **Impact:** Dramatically improves cache locality and reduces latency for
  I/O-bound tasks that frequently yield and resume. This is a key feature of the
  Tokio runtime itself.
- **Difficulty:** High
- **Skills:** Deep understanding of `async` `Future`s, `Waker`, and `Context`
  interaction.

#### **TODO 6: Introduce an Anti-Starvation Mechanism**

- **The Goal:** Prevent low-priority tasks from _never_ running on a perpetually
  busy scheduler.
- **The Problem:** If there is a constant, high-volume stream of `High` and
  `Normal` priority tasks, the scheduler's logic will always prefer them,
  potentially causing `Low` priority tasks to "starve" and never get a chance to
  execute.
- **Proposed Solution:**
    1.  Add a counter to each `Worker`'s state.
    2.  Every N tasks a worker completes (e.g., every 61 tasks, a prime number
        to avoid harmonic issues), the worker is _forced_ to try stealing one
        task specifically from the `Low` priority queue system.
    3.  If it finds a `Low` priority task, it executes it, then resets its
        counter and returns to its normal scheduling logic.
- **Impact:** Guarantees fairness and ensures that even on a fully loaded
  system, background and maintenance tasks will eventually make progress.
- **Difficulty:** Medium
- **Skills:** State management within the worker loop, algorithm design.

---

### **Level 5: Observability & Introspection** 🔬

A high-performance system is a black box without good tooling. This level is
about adding the tools needed to understand, debug, and profile the scheduler's
behavior in real-time.

#### **TODO 7: Expose Internal Metrics**

- **The Goal:** Provide a mechanism to query the state and performance of the
  scheduler at runtime.
- **The Problem:** It's currently impossible to know how many tasks are queued,
  how many steals have occurred, or how busy each worker is.
- **Proposed Solution:**
    1.  Create a `SchedulerMetrics` struct containing `AtomicUsize` counters for
        various events (e.g., `tasks_submitted`, `tasks_completed`,
        `steals_succeeded`, `steals_failed`, `workers_parked`).
    2.  Add an `Arc<SchedulerMetrics>` to the `Scheduler` and pass clones to
        each `Worker`.
    3.  Instrument the code: increment the appropriate counters at key points
        (e.g., in `Submit`, `Run`, `Steal`).
    4.  Add a `Scheduler::Metrics()` method that returns a snapshot of the
        current metrics, allowing external tools to monitor the scheduler's
        health.
- **Impact:** Enables powerful debugging, monitoring, and auto-scaling
  decisions. It transforms the scheduler from a black box into a transparent,
  observable system.
- **Difficulty:** Medium
- **Skills:** Concurrency primitives (`AtomicUsize`), API design.

#### **TODO 8: Integrate with `tracing` for Granular Timings**

- **The Goal:** Provide detailed, structured logs and timing information about
  the entire lifecycle of a task, compatible with modern observability platforms
  like Jaeger or Datadog.
- **The Problem:** `log` is good for simple messages, but `tracing` allows for
  structured, hierarchical "spans" that can measure the duration of specific
  operations.
- **Proposed Solution:**
    1.  Replace the `log` crate with the `tracing` crate.
    2.  Wrap key operations in `tracing::span!` macros. For example:
        - Create a span in `Scheduler::Submit` that gets a unique task ID.
        - The `Worker` can "enter" this span when it begins executing the task.
        - Create sub-spans for `PopLocal` and `StealFromSystem` to see where
          time is being spent.
    3.  The application using the `Echo` library can then configure a `tracing`
        subscriber to export this data to performance analysis tools.
- **Impact:** Provides unparalleled insight into performance bottlenecks. You
  can visually see how long tasks wait in the queue versus how long they take to
  execute.
- **Difficulty:** Medium
- **Skills:** `tracing` crate API, structured logging concepts.

---

### **Level 6: Modular Extensibility** 🧩

This level focuses on making the scheduler more flexible and adaptable to
different kinds of workloads.

#### **TODO 9: Support for Named Queues and Concurrency Limits**

- **The Goal:** Fully implement the `SchedulerBuilder::Queue()` API to allow
  users to create separate, named execution pools within the same scheduler,
  each with its own concurrency limit.
- **The Problem:** The current scheduler has one unified pool of workers. Some
  applications need to limit concurrency for specific types of tasks (e.g.,
  "only allow 4 concurrent disk I/O operations").
- **Proposed Solution:** This is a major architectural challenge.
    1.  The `SchedulerBuilder` would collect configurations for named queues.
    2.  The `Scheduler` would need to maintain multiple `Queue` systems or tag
        tasks with a queue name.
    3.  A "supervisor" or "dispatcher" component would be needed. When a worker
        becomes free, it wouldn't just steal from anywhere; it would ask the
        dispatcher which queue it should service based on current concurrency
        levels. This might involve using `tokio::sync::Semaphore` to manage
        concurrency limits for each named queue.
- **Impact:** Transforms `Echo` from a general-purpose scheduler into a highly
  sophisticated runtime capable of managing complex, heterogeneous workloads
  with fine-grained control.
- **Difficulty:** Very High
- **Skills:** Advanced architectural design, complex state management, deep
  knowledge of concurrency patterns and primitives.
