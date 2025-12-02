# GEMINI.md: Project Context for `nix-mcp`

This file provides a comprehensive overview of the `nix-mcp` monorepo for AI-powered development assistance.

## Project Overview

This is a monorepo for the **Model Context Protocol (MCP)**, a protocol designed to allow Large Language Models (LLMs) to interact with external tools and data sources in a standardized way. The repository contains several key components:

*   **Rust SDK (`rust-sdk`):** The official Rust SDK for implementing the MCP. It includes the core `rmcp` crate and `rmcp-macros` for procedural macro support.
*   **OpenAPI Converter (`rmcp-openapi`):** A Rust workspace that provides tools to convert OpenAPI specifications into MCP tools. This allows AI agents to interact with standard REST APIs. It can be used as a library or as a standalone MCP server.
*   **Reference Servers (`servers`):** A collection of reference MCP server implementations that demonstrate how to expose various functionalities as tools for LLMs. These servers are written in TypeScript and Python and are managed as an npm workspace. Examples include servers for filesystem operations, Git, web fetching, and more.
*   **Nix Environment:** The entire monorepo uses [Nix](https://nixos.org/) to provide a consistent and reproducible development environment, as defined in `flake.nix`. This environment includes all necessary dependencies for the Rust, TypeScript, and Python components.

## Building and Running

The project is a monorepo with components in different languages. Here’s how to build and run each part:

### 1. Development Environment

The primary way to enter the development environment is by using `direnv`, which is configured to use the `flake.nix` file.

```bash
# Allow direnv to load the environment (run once)
direnv allow

# The shell will be automatically configured when you enter the directory.
```

Alternatively, you can use `nix develop`:

```bash
nix develop
```

### 2. Rust Components (`rust-sdk` and `rmcp-openapi`)

The Rust projects are managed with Cargo workspaces.

```bash
# Navigate to the desired Rust project directory
cd rust-sdk
# or
cd rmcp-openapi

# Build the workspace
cargo build --workspace

# Run tests
cargo test --workspace
```

### 3. TypeScript Servers (`servers`)

The TypeScript servers are managed as an npm workspace.

```bash
# Navigate to the servers directory
cd servers

# Install dependencies
npm install

# Build all server packages
npm run build

# To run a specific server (e.g., the 'everything' server)
npx @modelcontextprotocol/server-everything
```

## Development Conventions

*   **Rust:** The Rust projects adhere to standard Rust conventions.
    *   **Formatting:** Use `cargo fmt` to format the code.
    *   **Linting:** Use `cargo clippy` to check for common mistakes. The `rust-sdk` project includes a `clippy.toml` for configuration.
    *   **Toolchain:** The `rust-sdk` project specifies the Rust toolchain in `rust-toolchain.toml`.
*   **TypeScript:** The TypeScript projects in the `servers` directory follow standard npm and TypeScript conventions.
    *   Code is organized into individual packages within the `src` directory.
    *   Each package has its own `package.json` and `tsconfig.json`.
*   **Nix:** All dependencies and the development shell are managed through `flake.nix`. Any new system-level dependencies should be added there.
*   **Git:** This is a Git repository. Please follow standard Git practices for branching and commits.

This `GEMINI.md` file should provide a solid foundation for any future interactions with this project.
