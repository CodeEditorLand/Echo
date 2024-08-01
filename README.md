# 📣 [Echo] — Asynchronous Action Processing System

Echo is a sophisticated asynchronous action processing system designed to manage
and execute various types of actions efficiently. It implements a worker-stealer
pattern and utilizes asynchronous queues to manage tasks effectively.

## Table of Contents

-   [Introduction](#Introduction)
-   [Features](#Features)
-   [Installation](#Installation)
-   [Usage](#Usage)
-   [Architecture](#Architecture)
-   [Contributing](CONTRIBUTING.md)
-   [License](LICENSE)

## Introduction

Echo provides a robust framework for defining, queuing, and executing actions
asynchronously. It's designed to handle complex workflows with features like
metadata management, function planning, and error handling.

## Features

-   **WebSocket Communication**: Facilitates real-time communication between
    different components of the system.
-   **Asynchronous Operations**: Utilizes Rust's async/await syntax for
    non-blocking execution.
-   **Action Planning**: Flexible system for defining and executing actions with
    custom logic.
-   **Metadata Management**: Each action can carry metadata for additional
    context and control.
-   **Error Handling**: Comprehensive error handling with custom `ActionError`
    types.
-   **Retry Mechanism**: Built-in retry logic for failed actions with
    exponential backoff.
-   **Hooks**: Support for pre and post-execution hooks.
-   **Serialization**: Actions can be serialized and deserialized for
    persistence or network transfer.

## Installation

To get started with Echo, follow these steps:

1. **Add to your Cargo.toml**:

```toml
[dependencies]
Echo = { git = "HTTPS://github.com/CodeEditorLand/Echo.git" }
```

2. **Build the Project**:

```bash
cargo build
```

## Usage

Here's a basic example of how to use Echo:

```rust
use echo::{Action, ActionProcessor, ExecutionContext, Plan, PlanBuilder, Work, Worker};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Plan
    let plan = PlanBuilder::new()
        .with_signature(ActionSignature {
            name: "Read".to_string(),
            input_types: vec!["String".to_string()],
            output_type: "String".to_string(),
        })
        .with_function("Read", |args| async move {
            let path = args[0].as_str().unwrap();
            Ok(serde_json::json!(format!("Read content from: {}", path)))
        })?
        .build();

    // Create a Work queue
    let work = Arc::new(Work::new());

    // Create an ExecutionContext
    let context = ExecutionContext::new(/* ... */);

    // Create a Worker
    struct SimpleWorker;
    #[async_trait]
    impl Worker for SimpleWorker {
        async fn Receive(&self, action: Box<dyn ActionTrait>, context: &ExecutionContext) -> Result<(), ActionError> {
            action.execute(context).await
        }
    }
    let worker = Arc::new(SimpleWorker);

    // Create an ActionProcessor
    let processor = Arc::new(ActionProcessor::new(worker, work.clone(), context));

    // Create and assign an Action
    let action = Box::new(
        Action::new("Read", ReadAction { path: "some_path".to_string() }, Arc::new(plan))
            .with_metadata("delay", serde_json::json!(1))
    );
    work.assign(action).await;

    // Run the processor
    processor.Run().await;

    Ok(())
}
```

## Architecture

### Core Components

-   **Action**: Represents a unit of work with metadata and content.
-   **Plan**: Defines the structure and execution logic for actions.
-   **Work**: Manages the queue of actions to be processed.
-   **Worker**: Implements the logic for receiving and executing actions.
-   **ActionProcessor**: Coordinates the execution of actions using Workers and
    Work queues.
-   **ExecutionContext**: Provides shared context for action execution.

### WebSocket Communication

WebSockets are used to facilitate real-time communication between the Tauri
application, [Sun](HTTPS://GitHub.com/CodeEditorLand/Sun.git), and
[River](HTTPS://GitHub.com/CodeEditorLand/River.git). This ensures that file
operations are executed promptly and efficiently.

[Echo]: HTTPS://GitHub.Com/CodeEditorLand/Echo

### Diagrams

#### Class Diagram

```mermaid
classDiagram
    Signal~T~ <|-- Action
    VectorDatabase <|-- Action
    Plan <|-- Action
    ActionTrait <|.. Action
    Worker <|-- ActionProcessor
    Work <|-- ActionProcessor
    ExecutionContext <|-- ActionProcessor
    ActionSignature <|-- Plan
    Plan <-- PlanBuilder
```

#### Sequence Diagram

```mermaid
sequenceDiagram
    participant Client
    participant ActionProcessor
    participant Work
    participant Worker
    participant Action
    participant ExecutionContext
    participant Plan

    Client->>ActionProcessor: New(Site, Work, Context)
    Client->>ActionProcessor: Run()
    loop Until Shutdown
        ActionProcessor->>Work: Execute()
        Work-->>ActionProcessor: Some(Action)
        ActionProcessor->>ActionProcessor: ExecuteWithRetry(Action)
        ActionProcessor->>Worker: Receive(Action, Context)
        Worker->>Action: Execute(Context)
        Action->>ExecutionContext: Get hooks and Execute
        Action->>Plan: GetFunction(ActionType)
        Plan-->>Action: Some(Function)
        Action->>Action: Execute function
        Action-->>Worker: Result
        Worker-->>ActionProcessor: Result
    end
    Client->>ActionProcessor: Shutdown()
    ActionProcessor->>ActionProcessor: Set ShutdownSignal
```

#### [Example](./Example/Queue.rs)

#### Class Diagram

```mermaid
classDiagram
    class ReadAction {
        +Path: String
    }

    class Action~ReadAction~ {
        +ExecuteLogic(Context: &ExecutionContext) Result<(), ActionError>
    }

    class SimpleWorker {
        +Receive(Action: Box<dyn ActionTrait>, Context: &ExecutionContext) Result<(), ActionError>
    }

    class EmptyContent

    class Main {
        +Main() Result<(), Box<dyn std::error::Error>>
    }

    class ExecutionContext {
        +HookMap: Arc<DashMap<String, Hook>>
        +Config: Arc<Config>
        +Cache: Arc<Mutex<DashMap<String, serde_json::Value>>>
    }

    class Work {
        +New()
        +Assign(Action: Box<dyn ActionTrait>)
    }

    class ActionProcessor {
        +New(Site: Arc<dyn Worker>, Work: Arc<Work>, Context: ExecutionContext)
        +Run()
        +Shutdown()
    }

    class PlanBuilder {
        +New()
        +WithSignature(Signature: ActionSignature)
        +WithFunction(Name: &str, Func: F)
        +Build() Plan
    }

    class Plan {
        +AddSignature(Signature: ActionSignature)
        +AddFunction(Name: &str, Func: F)
    }

    Action~ReadAction~ --|> ActionTrait
    SimpleWorker ..|> Worker
    Main ..> ReadAction : creates
    Main ..> SimpleWorker : creates
    Main ..> ExecutionContext : creates
    Main ..> Work : creates
    Main ..> ActionProcessor : creates
    Main ..> PlanBuilder : uses
    Main ..> Plan : creates
    Main ..> Action~ReadAction~ : creates
    Main ..> EmptyContent : creates
    PlanBuilder ..> Plan : builds
    ActionProcessor o-- Work
    ActionProcessor o-- SimpleWorker
```

#### Sequence Diagram

```mermaid
sequenceDiagram
    participant Main
    participant Config
    participant Work
    participant ExecutionContext
    participant PlanBuilder
    participant Plan
    participant SimpleWorker
    participant ActionProcessor
    participant ReadAction

    Main->>Config: build()
    Main->>Work: New()
    Main->>ExecutionContext: create
    Main->>PlanBuilder: New()
    PlanBuilder->>PlanBuilder: WithSignature()
    PlanBuilder->>PlanBuilder: WithFunction()
    PlanBuilder->>Plan: Build()
    Main->>SimpleWorker: create
    Main->>ActionProcessor: New(Site, Work, Context)
    Main->>ActionProcessor: Run()
    Main->>Action: New("Commander", EmptyContent, SharedPlan)
    Main->>Action: New("Read", ReadAction, SharedPlan)
    Main->>Work: Assign(ReadAction)
    Main->>Main: sleep(5 seconds)
    Main->>ActionProcessor: Shutdown()

    ActionProcessor->>Work: Execute()
    Work-->>ActionProcessor: Some(Action)
    ActionProcessor->>SimpleWorker: Receive(Action, Context)
    SimpleWorker->>ReadAction: Execute(Context)
    ReadAction->>ReadAction: ExecuteLogic()
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a history of changes to this component.
