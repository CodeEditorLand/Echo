# 📣 [Echo] — Asynchronous Action Processing System

`Echo` is a Rust library designed for managing and executing asynchronous
actions efficiently. It leverages a worker-stealer pattern and asynchronous
queues to handle complex workflows with features like metadata management,
function planning, and robust error handling.

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

-   **Asynchronous Operations:** Built with Rust's async/await syntax for
    non-blocking execution.
-   **Action Planning:** Define and execute actions with custom logic using a
    flexible Plan system.
-   **Metadata Management:** Attach metadata to actions for additional Life and
    control.
-   **Error Handling:** Comprehensive error management with custom `Error`
    types.
-   **Retry Mechanism:** Built-in retry logic for failed actions with
    exponential backoff.
-   **Hooks:** Supports pre and post-execution hooks for added flexibility.
-   **Serialization:** Actions can be serialized and deserialized for
    persistence or network transfer (in progress).

## 🚀 Installation

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

## 🛠️ Usage

Here's a basic example demonstrating how to define and execute an Action:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Define the Action's logic
	let Read = |_Argument: Vec<serde_json::Value>| async move {
		// Access the provided path (replace with actual logic)
		let Path = "path/to/file.txt";

		// Simulate reading from the path
		let Content = format!("Content read from: {}", Path);

		Ok(json!(Content))
	};

	// Create an Action Plan
	let Plan = Plan::New()
		.WithSignature(Echo::Struct::Sequence::Action::Signature::Struct {
			Name: "Read".to_string(),
		})
		.WithFunction("Read", Read)?
		.Build();

	// Create a work queue
	let Production = Arc::new(Production::New());

	// Create a lifecycle Life (replace with your actual configuration)
	let Life = Life {
		Span: Arc::new(dashmap::DashMap::new()),
		Fate: Arc::new(config::Config::default()),
		Cache: Arc::new(Mutex::new(dashmap::DashMap::new())),
		Karma: Arc::new(dashmap::DashMap::new()),
	};

	// Define a Site to execute actions
	struct SimpleSite;

	#[async_trait::async_trait]
	impl Site for SimpleSite {
		async fn Receive(
			&self,
			Action: Box<dyn ActionTrait>,
			Life: &Life,
		) -> Result<(), Error> {
			Action.Execute(Life).await
		}
	}
	let Site = Arc::new(SimpleSite);

	// Create an Action Sequence
	let Sequence = Arc::new(Sequence::New(Site, Production.clone(), Life));

	// Create an Action and add it to the queue
	let Action = Action::New(
		"Read",
		json!("SomeData"),
		Arc::clone(&Plan),
	);

	Production.Assign(Box::new(Action)).await;

	// Run the Sequence
	Sequence.Run().await;

	Ok(())
}

use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use Echo::Sequence::{
	Action::{Error::Enum as Error, Struct as Action, Trait as ActionTrait},
	Life::Struct as Life,
	Plan::{Formality::Struct as Formality, Struct as Plan},
	Production::Struct as Production,
	Site::Trait as Site,
	Struct as Sequence,
};
```

## 🏛️ Architecture

### Core Components

-   **Action:** Represents a unit of Production with associated metadata,
    content, and execution logic.
-   **Plan:** Defines the structure and functions for different Action types.
-   **Production:** A thread-safe queue for managing pending actions.
-   **Site:** Implements the logic for receiving and executing actions from the
    queue.
-   **Sequence:** Orchestrates the execution of actions using workers and the
    work queue.
-   **Life:** Provides a shared Life and configuration for actions during
    execution.

### Diagrams

#### State Diagram

```mermaid
stateDiagram-v2
    [*] --> Library
    Library --> Enum
    Library --> Struct
    Library --> Trait
    Library --> Type
    Enum --> Sequence
    Sequence --> Action
    Action --> Error
    Struct --> Sequence
    Sequence --> Action
    Action --> Signature
    Sequence --> Life
    Sequence --> Plan
    Plan --> Formality
    Sequence --> Production
    Sequence --> Signal
    Sequence --> Vector
    Trait --> Sequence
    Sequence --> Action
    Sequence --> Site
    Type --> Sequence
    Sequence --> Action
    Action --> Cycle
```

#### Class Diagram

```mermaid
classDiagram
    class `Enum::Sequence::Action::Error::Enum` {
        -License(String)
        -Execution(String)
        -Routing(String)
        -Cancellation(String)
    }
    class `Struct::Sequence::Action::Signature::Struct` {
        -Name: String
    }
    class `Struct::Sequence::Action::Struct::T` {
        -Metadata: Vector
        -Content: T
        -License: Signal<bool>
        -Plan: Arc<Formality>
        +New(Action: &str, Content: T, Plan: Arc<Formality>)
        +WithMetadata(Key: &str, Value: serde_json::Value)
        +Execute(Context: &Life)
    }
    class `Struct::Sequence::Life::Struct` {
        -Span: Arc<DashMap<String, Type::Sequence::Action::Cycle::Type>>
        -Fate: Arc<Config>
        -Cache: Arc<Struct::Sequence::Mutex<DashMap<String, serde_json::Value>>>
        -Karma: Arc<DashMap<String, Arc<Struct::Sequence::Production::Struct>>>
    }
    class `Struct::Sequence::Plan::Formality::Struct` {
        -Signature: DashMap<String, Signature>
        -Function: DashMap<String, Box<dyn Fn(Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, Error>> + Send>> + Send + Sync>
        +New()
        +Sign(Signature: Signature)
        +Add(Name: &str, Function: F)
        +Remove(Name: &str)
    }
    class `Struct::Sequence::Plan::Struct` {
        -Formality: Formality
        +New()
        +WithSignature(Signature: Struct::Sequence::Action::Signature::Struct)
        +WithFunction(Name: &str, Function: F)
        +Build()
    }
    class `Struct::Sequence::Production::Struct` {
        -Line: Arc<Mutex<VecDeque<Box<dyn Action>>>>
        +New()
        +Do()
        +Assign(Action: Box<dyn Action>)
    }
    class `Struct::Sequence::Signal::Struct::T` {
        -0: Arc<Mutex<T>>
        +New(Value: T)
        +Get()
        +Set(To: T)
    }
    class `Struct::Sequence::Vector::Struct` {
        -Entry: DashMap<String, serde_json::Value>
        +New()
        +Insert(Key: String, Value: serde_json::Value)
        +Get(Key: &str)
    }
    class `Struct::Sequence::Struct` {
        -Site: Arc<dyn Site>
        -Production: Arc<Production>
        -Life: Life
        -Time: Signal<bool>
        +New(Site: Arc<dyn Site>, Production: Arc<Production>, Life: Life)
        +Run()
        +Shutdown()
    }
    class `Trait::Sequence::Action::Trait` {
        +Execute(Context: &Life)
        +Clone()
    }
    class `Trait::Sequence::Site::Trait` {
        +Receive(Action: Box<dyn Trait::Sequence::Action::Trait>, Context: &Struct::Sequence::Life::Struct)
    }
    `Enum::Sequence::Action::Error::Enum` --|> `thiserror::Error`
    `Struct::Sequence::Action::Struct::T` --|> `serde::Serialize`
    `Struct::Sequence::Action::Struct::T` --|> `serde::Deserialize`
    `Struct::Sequence::Action::Struct::T` --|> `Trait::Sequence::Action::Trait`
    `Struct::Sequence::Plan::Formality::Struct` --|> `std::fmt::Debug`
    `Struct::Sequence::Plan::Struct` *-- `Struct::Sequence::Plan::Formality::Struct`
    `Struct::Sequence::Signal::Struct::T` *-- `Struct::Sequence::Mutex`
    `Struct::Sequence::Struct` *-- `Trait::Sequence::Site::Trait`
    `Struct::Sequence::Struct` *-- `Struct::Sequence::Production::Struct`
    `Struct::Sequence::Struct` *-- `Struct::Sequence::Life::Struct`
    `Struct::Sequence::Struct` *-- `Struct::Sequence::Signal::Struct::T`
    `Trait::Sequence::Action::Trait` <.. `Struct::Sequence::Life::Struct`
    `Trait::Sequence::Action::Trait` <.. `Enum::Sequence::Action::Error::Enum`
    `Trait::Sequence::Site::Trait` --|> `async_trait::async_trait`
    `Trait::Sequence::Site::Trait` <.. `Trait::Sequence::Action::Trait`
    `Trait::Sequence::Site::Trait` <.. `Struct::Sequence::Life::Struct`
    `Trait::Sequence::Site::Trait` <.. `Enum::Sequence::Action::Error::Enum`
    `Type::Sequence::Action::Cycle::Type` --|> `Struct::Sequence::Arc`
```

#### Sequence Diagram

```mermaid
sequenceDiagram
    participant Client
    participant Action
    participant Metadata
    participant License
    participant Context
    participant Plan
    participant Hooks
    participant Function

    activate Client
    Client->>Action: Execute(Context)
    activate Action
    Note right of Action: The client initiates the execution of an action represented by the 'Action' object

    Action->>Metadata: Get("Action")
    alt "Action" not found
        Action->>Action: Return Error
        Note right of Action: Returns an error if "Action" is not found in the metadata
    else "Action" found
        Metadata-->>Action: Return Action
        Action->>License: Get()
        alt License Invalid
            Action->>Action: Return Error
            Note right of Action: Return an error if the action is not properly licensed
        else License Valid
            Action->>Metadata: Get("Delay")
            alt Delay exists
                Metadata-->>Action: Return Delay
                Action->>Action: sleep(Delay)
                Note right of Action: If a delay is specified, wait for the given duration
            end
            Action->>Metadata: Get("Hooks")
            alt Hooks exist
                Metadata-->>Action: Return Hooks
                loop Hook in Hooks
                    Action->>Context: Span.get(Hook)
                    alt Hook Function found
                        Context-->>Action: Return HookFn
                        Action->>HookFn: call()
                        alt HookFn Error
                            Action->>Action: Return Error
                            Note right of Action: If a hook function returns an error, stop execution and return the error
                        end
                    end
                end
            end
            Action->>Plan: Remove(Action)
            alt Function not found
                Action->>Action: Return Error
                Note right of Action: Return an error if no function is found for the given action
            else Function found
                Plan-->>Action: Return Function
                Action->>Action: Argument()
                Action->>Function: call(Argument)
                activate Function
                Function-->>Action: Return Result
                deactivate Function
                alt Function Error
                    Action->>Action: Return Error
                    Note right of Action: If the function execution returns an error, propagate the error
                else Function Success
                    Action->>Action: Result(Result)
                    Action->>Metadata: Get("NextAction")
                    alt NextAction exists
                        Metadata-->>Action: Return NextAction
                        Action->>Action: Execute(NextAction, Context)
                        alt NextAction Error
                            Action->>Action: Return Error
                            Note right of Action: If the execution of the next action results in an error, return the error
                        end
                    end
                end
            end
        end
    end
    deactivate Action
    Client->>Client: Return Result
    Note right of Client: Returns the result of the action execution, which can be a success or an error
```

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for
guidelines and feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT [LICENSE](LICENSE).

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a history of changes to this component.

[Echo]: HTTPS://GitHub.Com/CodeEditorLand/Echo
