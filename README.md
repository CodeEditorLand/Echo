# 📣 [Echo] — Asynchronous Action Processing System

`Echo` is a Rust library designed for managing and executing asynchronous actions efficiently. It leverages a worker-stealer pattern and asynchronous queues to handle complex workflows with features like metadata management, function planning, and robust error handling.

## Table of Contents

-   [Introduction](#Introduction)
-   [Features](#Features)
-   [Installation](#Installation)
-   [Usage](#Usage)
-   [Architecture](#Architecture)
-   [Contributing](CONTRIBUTING.md)
-   [License](LICENSE)

## Introduction

`Echo` provides a robust framework for defining, queuing, and executing actions
asynchronously. It's designed to handle complex workflows with features like
metadata management, function planning, and error handling.

## Features

- **Asynchronous Operations:** Built with Rust's async/await syntax for non-blocking execution.
- **Action Planning:** Define and execute actions with custom logic using a flexible plan system.
- **Metadata Management:** Attach metadata to actions for additional context and control.
- **Error Handling:** Comprehensive error management with custom `Error` types.
- **Retry Mechanism:** Built-in retry logic for failed actions with exponential backoff.
- **Hooks:** Supports pre and post-execution hooks for added flexibility.
- **Serialization:** Actions can be serialized and deserialized for persistence or network transfer (in progress).

## Installation

To get started with `Echo`, follow these steps:

1. **Add to your Cargo.toml**:

```toml
[dependencies]
Echo = { git = "HTTPS://GitHub.Com/CodeEditorLand/Echo.git" }
```

2. **Build the Project**:

```bash
cargo build
```

## Usage

Here's a basic example demonstrating how to define and execute an action:

```rust
use Echo::Sequence::{
    Action::{Error::Enum as Error, Struct as Action, Trait as ActionTrait},
    Life::Struct as Life,
    Plan::{Formality::Struct as Formality, Struct as Plan},
    Production::Struct as Production,
    Site::Trait as Site,
    Struct as Sequence,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the action's logic
    let read_function = |_args: Vec<serde_json::Value>| async move {
        // Access the provided path (replace with actual logic)
        let path = "path/to/file.txt"; 

        // Simulate reading from the path
        let content = format!("Content read from: {}", path);

        Ok(json!(content))
    };

    // Create an action plan
    let plan = Plan::New()
        .WithSignature(Echo::Struct::Sequence::Action::Signature::Struct {
            Name: "Read".to_string(),
        })
        .WithFunction("Read", read_function)?
        .Build();

    // Create a work queue
    let work = Arc::new(Production::New());

    // Create a lifecycle context (replace with your actual configuration)
    let context = Life::Struct {
        Span: Arc::new(dashmap::DashMap::new()),
        Fate: Arc::new(config::Config::default()),
        Cache: Arc::new(Mutex::new(dashmap::DashMap::new())),
        Karma: Arc::new(dashmap::DashMap::new()),
    };

    // Define a worker to execute actions
    struct SimpleWorker;

    #[async_trait::async_trait]
    impl Site for SimpleWorker {
        async fn Receive(
            &self,
            action: Box<dyn ActionTrait>,
            context: &Life,
        ) -> Result<(), Error> {
            action.Execute(context).await
        }
    }
    let worker = Arc::new(SimpleWorker);

    // Create an action processor
    let processor = Arc::new(Sequence::New(worker, work.clone(), context));

    // Create an action and add it to the queue
    let action = Action::New(
        "Read",
        json!("SomeData"), 
        Arc::clone(&plan),
    );
    work.Assign(Box::new(action)).await;

    // Run the processor
    processor.Run().await;

    Ok(())
}
```

## Architecture

### Core Components

- **Action:** Represents a unit of work with associated metadata, content, and execution logic.
- **Plan:** Defines the structure and functions for different action types.
- **Production:** A thread-safe queue for managing pending actions.
- **Site:** Implements the logic for receiving and executing actions from the queue.
- **Sequence:** Orchestrates the execution of actions using workers and the work queue.
- **Life:** Provides a shared context and configuration for actions during execution.

### Diagrams

#### Class Diagram

```mermaid
classDiagram
    class Error {
        <<enumeration>>
        +License
        +Execution
        +Routing
        +Cancellation
    }

    class ActionSignature {
        +Name
    }

    class Action {
        +Metadata
        +Content
        +License
        +Plan
        +New
        +WithMetadata
        +Execute
    }

    class Life {
        +Span
        +Fate
        +Cache
        +Karma
    }

    class Formality {
        +Signature
        +Function
        +New
        +Sign
        +Add
        +Remove
    }

    class Plan {
        +Formality
        +New
        +WithSignature
        +WithFunction
        +Build
    }

    class Production {
        +Line
        +New
        +Do
        +Assign
    }

    class Signal {
        +0
        +New
        +Get
        +Set
    }

    class Vector {
        +Entry
        +New
        +Insert
        +Get
    }

    class Sequence {
        +Site
        +Production
        +Life
        +Time
        +New
        +Run
        +Again
        +Shutdown
    }

    class SimpleWorker {
        <<Example>>
        +Receive
    }

    Action --|> ActionTrait
    SimpleWorker ..|> Site
    Sequence o-- Site
    Sequence o-- Production
    Sequence o-- Life
    Action o-- Formality
    Formality o-- ActionSignature
    Plan o-- Formality
    Production o-- ActionTrait
    Sequence o-- Signal
    Action o-- Vector
    Life o-- Cycle
    Life o-- Production
```

#### Sequence Diagram

```mermaid
sequenceDiagram
    participant Main
    participant Plan
    participant Production
    participant Life
    participant SimpleWorker
    participant Sequence
    participant Action

    Main->>Plan: New()
    Main->>Plan: WithSignature()
    Main->>Plan: WithFunction()
    Plan->>Formality: Sign()
    Plan->>Formality: Add()
    Main->>Production: New()
    Main->>Life: Create
    Main->>SimpleWorker: Create
    Main->>Sequence: New(worker, work, context)
    Main->>Action: New("Read", json!("SomeData"), plan)
    Main->>Production: Assign(action)
    Main->>Sequence: Run()
    Sequence->>Production: Do()
    Production-->>Sequence: Some(Action)
    Sequence->>SimpleWorker: Receive(action, context)
    SimpleWorker->>Action: Execute(context)
    Action->>Formality: Remove()
    Formality-->>Action: Function
    Action->>Function: call()
    Main->>Sequence: Shutdown()
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the [LICENSE](LICENSE) file.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a history of changes to this component.

[Echo]: HTTPS://GitHub.Com/CodeEditorLand/Echo
