# 📣 [Echo] —

Echo is a communication library designed to facilitate file reading and writing
operations across multiple applications using WebSockets.

It implements the worker-stealer pattern and utilizes asynchronous and parallel
queues to efficiently manage tasks.

## Table of Contents

-   [Introduction](#Introduction)
-   [Features](#Features)
-   [Installation](#Installation)
-   [Usage](#Usage)
-   [Architecture](#Architecture)
-   [Contributing](CONTRIBUTING.md)
-   [License](LICENSE)

## Introduction

Echo is designed to streamline the process of reading and writing files across
different applications. By leveraging WebSockets, it ensures real-time
communication and efficient task management.

## Features

-   **Asynchronous Operations**:

Utilizes asynchronous functions to handle file operations, ensuring non-blocking
execution.

-   **WebSocket Communication**:

Facilitates real-time communication between different components of the system.

## Installation

To get started with Echo, follow these steps:

1. **Clone the Repository**:

    ```bash
    git clone ssh://git@github.com/CodeEditorLand/Echo.git
    cd Echo
    ```

2. **Build the Project**:

    ```bash
    cargo build
    ```

3. **Install Dependencies**:

    ```bash
    pnpm install
    ```

4. **Build the TypeScript project**:
    ```bash
    pnpm run prepublishOnly
    ```

## Usage

## Architecture

### WebSocket Communication

WebSockets are used to facilitate real-time communication between the Tauri
application, Sun, and River. This ensures that file operations are executed
promptly and efficiently.

### Code Structure

-   **Interface**:

Defines the structure of the response object and the main asynchronous function
for handling responses.

-   **Worker**:

Contains the implementation of the worker-stealer pattern and the task queue
management.

-   **Main**:

The entry point of the Rust binaries, responsible for reading configuration
files and setting up the environment.

[Echo]: HTTPS://GitHub.Com/CodeEditorLand/Echo

```mermaid
classDiagram
    class Signal~T~ {
        -Value: Arc<Mutex<T>>
        +New(InitialValue: T)
        +Get() T
        +Set(NewValue: T)
    }

    class VectorDatabase {
        -Entries: DashMap<String, Signal<serde_json::Value>>
        +New()
        +Insert(Key: String, Value: serde_json::Value)
        +Get(Key: &str) Option<serde_json::Value>
    }

    class ActionSignature {
        +Name: String
        +InputTypes: Vec<String>
        +OutputType: String
    }

    class Plan {
        -Signatures: DashMap<String, ActionSignature>
        -Functions: DashMap<String, Box<dyn Fn(Vec<Value>) -> ...>>
        +New()
        +AddSignature(Signature: ActionSignature)
        +AddFunction(Name: &str, Func: F)
        +GetFunction(Name: &str)
    }

    class PlanBuilder {
        -Plan: Plan
        +New()
        +WithSignature(Signature: ActionSignature)
        +WithFunction(Name: &str, Func: F)
        +Build() Plan
    }

    class Action~T~ {
        +Metadata: VectorDatabase
        +Content: T
        +LicenseSignal: Signal<bool>
        +Plan: Arc<Plan>
        +New(ActionType: &str, Content: T, Plan: Arc<Plan>)
        +WithMetadata(Key: &str, Value: serde_json::Value)
        +Execute(Context: &ExecutionContext)
    }

    class ExecutionContext {
        +HookMap: Arc<DashMap<String, Hook>>
        +Config: Arc<Config>
        +Cache: Arc<Mutex<DashMap<String, serde_json::Value>>>
    }

    class Work {
        -Queue: Arc<Mutex<VecDeque<Box<dyn ActionTrait>>>>
        +New()
        +Execute() Option<Box<dyn ActionTrait>>
        +Assign(Action: Box<dyn ActionTrait>)
    }

    class ActionProcessor {
        -Site: Arc<dyn Worker>
        -Work: Arc<Work>
        -Context: ExecutionContext
        -ShutdownSignal: Signal<bool>
        +New(Site: Arc<dyn Worker>, Work: Arc<Work>, Context: ExecutionContext)
        +Run()
        +Shutdown()
    }

    class Worker {
        <<interface>>
        +Receive(Action: Box<dyn ActionTrait>, Context: &ExecutionContext)
    }

    class ActionTrait {
        <<interface>>
        +Execute(Context: &ExecutionContext)
        +Clone() Box<dyn ActionTrait>
    }

    Action ..|> ActionTrait
    Action o-- VectorDatabase
    Action o-- Signal
    Action o-- Plan
    ActionProcessor o-- Work
    ActionProcessor o-- ExecutionContext
    ActionProcessor o-- Worker
    Plan o-- ActionSignature
    PlanBuilder --> Plan
    Work o-- ActionTrait
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a history of changes to this integration.
