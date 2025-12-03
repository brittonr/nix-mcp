/// Argument types for Nix prompts
use rmcp::schemars;

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SetupDevEnvironmentArgs {
    /// Project type (e.g., "rust", "python", "nodejs", "go", "c", "generic")
    pub project_type: String,
    /// Additional tools or dependencies needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
    /// Whether to use flakes (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_flakes: Option<bool>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TroubleshootBuildArgs {
    /// The package or flake reference that's failing to build
    pub package: String,
    /// Error message or build log excerpt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct MigrateToFlakesArgs {
    /// Current setup description (e.g., "using nix-shell", "using configuration.nix")
    pub current_setup: String,
    /// Project type if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_type: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct OptimizeClosureArgs {
    /// Package to optimize
    pub package: String,
    /// Current closure size if known (in bytes or human-readable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_size: Option<String>,
    /// Target size or reduction goal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}
