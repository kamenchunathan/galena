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

Looking at your `build.roc` script, I can see the build process and dependencies. Here's an updated README section explaining the build process:

## Building and Development Setup

Galena is implemented in Rust with a Roc build script that orchestrates the compilation of multiple components including WebAssembly bindings, frontend assets, and backend binaries.

### Prerequisites

The build process requires these tools:

- **Roc compiler**: For compiling Roc code to various targets
- **Rust toolchain**: For building the host binaries and CLI
- **WebAssembly tools**:
  - `wasm-ld`: WebAssembly linker
  - `wasm-bindgen`: JavaScript/TypeScript bindings generator
  - `wasm2wat`: WebAssembly text format converter
- **Node.js ecosystem**:
  - `pnpm`: Package manager for frontend dependencies
- **System tools**: `cp`, `sh`, `grep`, `sed` (standard Unix utilities)

### Building the Project

The build process is automated through the `build.roc` script, which handles:

1. **Roc version verification**: Ensures the Roc compiler is available
2. **Stub library creation**: Builds a shared library stub for the target platform
3. **Backend host compilation**: Builds Rust backend host binaries using Cargo
4. **Frontend host compilation**: Builds WebAssembly frontend host
5. **WebAssembly processing**: Links and generates JavaScript bindings
6. **Frontend asset building**: Compiles frontend assets with pnpm
7. **CLI compilation**: Builds the final Galena CLI tool

To build the platform:

```just
just build
```

which runs `roc build.roc`

### Development Environment

For development, a Nix flake is provided that includes all necessary dependencies:

```bash
nix develop
```

This provides a development shell with:

- Rust toolchain with WebAssembly targets
- Roc compiler and language server
- WebAssembly tools (wasmtime, wasm-tools, wabt, wasm-bindgen)
- Node.js 24 and pnpm
- LLVM tools and debugger support

### Build Process Details

The `build.roc` script performs these steps in order:

1. **Platform Detection**: Determines the target OS and architecture
2. **Stub App Library**: Creates `platform/libapp.{dylib|so|dll}` for the target platform
3. **Backend Host**: Builds `libhost.a` and copies it to the appropriate platform-specific location
4. **Host Preprocessing**: Runs `roc preprocess-host` to prepare the surgical host
5. **Frontend Compilation**:
   - Builds the frontend host as WebAssembly
   - Extracts exported functions using `wasm2wat`
   - Links with `wasm-ld` to create the final WASM module
   - Generates TypeScript bindings with `wasm-bindgen`
6. **Frontend Assets**: Runs `pnpm build` in the frontend directory
7. **CLI Build**: Compiles the final Galena CLI with all components

### Running Applications

Once built, you can use the Galena CLI to build and run applications:

```bash
./target/release/galena_cli build examples/hello_world.roc
./target/release/galena_cli watch examples/hello_world.roc
```

### Development Workflow

For active development on the platform itself:

1. Make changes to platform code
2. Run `roc build.roc` to rebuild
3. Test with example applications

The build script automatically handles cross-platform differences and ensures all components are properly linked together.## Project Structure

When you build a Galena application, it creates:

- A `.galena/dist` directory with frontend assets
- A WebAssembly file for your frontend code
- A native binary for your backend

## Contributing

Galena is in active development and contributions are welcome to help implement missing features from Lamdera.
