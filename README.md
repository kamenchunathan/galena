# Galena: Lamdera for Roc

Galena is a proof of concept reimplementation of [Lamdera](https://www.lamdera.com/) for the Roc programming language. Much like how Lamdera simplifies full-stack web development in Elm, Galena aims to bring the same streamlined experience to Roc.

The name "Galena" is a mineral pun, continuing the tradition from Rust - Galena being a lead-sulfide mineral that, like the project itself, creates connections between components (in this case, frontend and backend).

## Current Status

Galena is an early-stage project and is **not feature complete**. It provides the basic architecture for building full-stack Roc applications with shared types between frontend and backend, but many features from Lamdera are still being implemented.

## Core Concept

Galena follows Lamdera's philosophy of "Don't hide complexity, remove it." Instead of writing glue code to connect various components, Galena handles the communication between frontend and backend, allowing you to focus on business logic.

For full details about the Lamdera model that Galena is replicating, refer to the [official Lamdera documentation](https://www.lamdera.com/).

## Galena App Structure: Types and Functions

The `platform/main.roc` file establishes the core structure of a Galena application. Here's a concise breakdown of the required types and functions:

### Required Types

A Galena app must define these five core types:

1. **`FrontendModel`**: The state of your frontend application
2. **`BackendModel`**: The state of your backend application
3. **`FrontendMsg`**: Messages handled by the frontend (UI events, user actions)
4. **`ToBackendMsg`**: Messages sent from frontend to backend
5. **`ToFrontendMsg`**: Messages sent from backend to frontend

### Frontend Functions

`frontendApp` must implement `Frontend.frontend` with these functions:

```roc
frontendApp = Frontend.frontend {
    init!: /* Initial frontend model */,
    update: /* Handle frontend messages */,
    view: /* Render the UI */,
    updateFromBackend: /* Process messages from backend */
}
```

- **`init!`**: Creates the initial frontend model
- **`update`**: Processes `FrontendMsg`, updates model, and optionally sends `ToBackendMsg`
- **`view`**: Renders the UI based on the current model
- **`updateFromBackend`**: Handles incoming `ToFrontendMsg` from backend

The `update` function returns a tuple with the updated model and an optional message to send to the backend:

```roc
update: FrontendMsg, FrontendModel -> (FrontendModel, Result ToBackendMsg [NoOp])
```

### Backend Functions

`backendApp` must implement `Backend.backend` with these functions:

```roc
backendApp = Backend.backend {
    init!: /* Initial backend model */,
    update!: /* Handle backend-specific messages */,
    update_from_frontend: /* Process messages from frontend */
}
```

- **`init!`**: Creates the initial backend model
- **`update!`**: Processes backend messages and optionally sends responses to frontends
- **`update_from_frontend`**: Transforms incoming `ToBackendMsg` into backend-specific messages

The backend `update!` function returns the updated model and an optional message to send to a specific client:

```roc
update!: BackendMsg, BackendModel -> (BackendModel, Result (Str, ToFrontendMsg) [NoOp])
```

The `update_from_frontend` function receives client information and a message:

```roc
update_from_frontend: Str, Str, ToBackendMsg -> BackendMsg
```

Where the first `Str` is the client ID, the second `Str` is the session ID, and the function converts the message to an appropriate `BackendMsg`.

### App Declaration

This structure is declared in your application's main file:

```roc
app [
    FrontendModel,
    BackendModel,
    ToFrontendMsg,
    FrontendMsg,
    ToBackendMsg,
    frontendApp,
    backendApp,
] { galena: platform "path/to/platform/main.roc" }
```

This pattern enables type-safe communication between frontend and backend components while maintaining a clear separation of concerns.

## Hello World Example

Here's the included example showing a simple counter application:

```roc
app [
    FrontendModel,
    BackendModel,
    ToFrontendMsg,
    FrontendMsg,
    ToBackendMsg,
    frontendApp,
    backendApp,
] { galena: platform "../platform/main.roc" }

import galena.Backend as Backend exposing [Backend]
import galena.Frontend as Frontend exposing [Frontend]
import galena.View as View

FrontendModel : { counter : I32 }

BackendModel : {
    counter : I32,
}

ToFrontendMsg : I32

ToBackendMsg : I32

FrontendMsg : [Increment, NoOp]

BackendendMsg : [UpdateCounter Str I32]

frontendApp : Frontend FrontendModel FrontendMsg ToFrontendMsg ToBackendMsg
frontendApp = Frontend.frontend {
    init!: { counter: 0 },

    update: frontend_update,

    view: view,

    updateFromBackend: |_| NoOp,
}

frontend_update : FrontendMsg, FrontendModel -> (FrontendModel, Result ToBackendMsg [NoOp])
frontend_update = |msg, model|
    when msg is
        Increment ->
            incr = model.counter + 1
            ({ counter: incr }, Ok incr)

        NoOp -> (model, Err NoOp)

view : FrontendModel -> View.View FrontendMsg
view = |model|
    View.div
        [View.id "main", View.class "bg-red-400 text-xl font-semibold"]
        [
            View.div [] [
                View.text (Num.to_str model.counter),
                View.button
                    [
                        View.id "incr",
                        View.class "bg-slate-400 border-1 border-blue-400 outline-none",
                        View.on_click Increment,
                    ]
                    [View.text "+"],
            ],
        ]

backendApp : Backend BackendModel BackendendMsg ToFrontendMsg ToBackendMsg
backendApp = Backend.backend {
    init!: { counter: 0 },
    update!: |msg, model|
        when msg is
            UpdateCounter client_id client_counter ->
                (
                    { counter: model.counter + client_counter },
                    Ok (client_id, model.counter + client_counter),
                ),
    update_from_frontend: update_from_frontend,
}

update_from_frontend : Str, Str, ToBackendMsg -> BackendendMsg
update_from_frontend = |client_id, _, client_counter| UpdateCounter client_id client_counter
```

This example demonstrates:

1. A counter on the frontend
2. Incrementing the counter locally and sending the value to the backend
3. The backend updating its own counter and sending a response back

### Major gotchas

> You currently cannot use tagged unions as ToBackendMsg and ToBackendMsg. kinda defeats the purpose but that'll hopefully be fixed soon

## Building and Development Setup

Galena is implemented in Rust with Nix dependencies, and uses Just as a command runner for development workflows.

### Prerequisites

- Rust toolchain
- Nix package manager
- Just command runner
- Roc source code (built locally)

### Building the Project

1. First, build the platform code:

```bash
just build-platform
```

This compiles the necessary Rust components that enable the Galena platform.

2. Once the platform is built, you can run Galena commands using Cargo:

```bash
cargo run --package galena_cli -- build examples/hello_world.roc
```

Replace `$ROC_SRC_CODE_PATH` with the path to your local Roc source code directory.

### Development Workflow

For active development, use the watch command to automatically rebuild when files change:

```bash
cargo run --package galena_cli -- watch --paths platform/ examples/hello_world.roc
```

This will:

1. Watch both the platform directory and your example application
2. Rebuild automatically when changes are detected
3. Run the application after successful builds

### Project Structure

- `crates/galena_cli`: The CLI tool for building and running Galena applications
- `platform/`: Core platform code for Galena
- `examples/`: Example applications

When modifying the platform code, make sure to build both the platform and your application to see changes take effect.

Note: Since Galena is a work in progress, you may need to build from source with the latest Roc version to ensure compatibility.

### Options

- `--roc-bin`: Specify a custom path to the Roc binary this is important for installations of roc with nix

## Development Workflow

Use `galena watch` during development to automatically rebuild on changes. It does not provide live reload and only rebuilds the binaries on changes

## Project Structure

When you build a Galena application, it creates:

- A `.galena/dist` directory with frontend assets
- A WebAssembly file for your frontend code
- A native binary for your backend

## Contributing

Galena is in active development and contributions are welcome to help implement missing features from Lamdera.
