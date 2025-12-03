//! Common infrastructure and utilities module.
//!
//! This module contains shared infrastructure used across all MCP tools:
//!
//! # Modules
//!
//! - [`cache`] - TTL-based cache implementation for expensive operations
//! - [`cache_registry`] - Centralized cache management across all tools
//! - [`tool_registry`] - Central registry for all tool module instances
//! - [`security`] - Input validation, audit logging, and security utilities
//! - [`nix_server`] - Main MCP server implementation
//! - [`nix_tools_helpers`] - Helper functions for Nix tool implementations
//! - [`command`] - Command execution utilities (currently unused)
//! - [`caching`] - Advanced caching strategies (currently unused)
//!
//! # Architecture
//!
//! The common module provides the foundation for the onix-mcp server:
//!
//! ```text
//! NixServer
//!   ├── ToolRegistry (manages all tool instances)
//!   │   ├── PackageTools, BuildTools, etc.
//!   │   └── Each tool has Arc<AuditLogger> and Arc<CacheRegistry>
//!   ├── CacheRegistry (7 specialized caches)
//!   └── AuditLogger (security event logging)
//! ```

pub mod cache;
pub mod cache_registry;
pub mod caching;
pub mod command;
pub mod nix_server;
pub mod nix_tools_helpers;
pub mod security;
pub mod tool_registry;
