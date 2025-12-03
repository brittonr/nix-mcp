/// Argument types for Clan machine management tools
use rmcp::schemars;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineCreateArgs {
    /// Name of the machine to create
    pub name: String,
    /// Optional template to use (default: "new-machine")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// Optional target host address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_host: Option<String>,
    /// Optional flake directory path (default: current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineListArgs {
    /// Optional flake directory path (default: current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineUpdateArgs {
    /// Machines to update (empty for all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machines: Option<Vec<String>>,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineDeleteArgs {
    /// Name of the machine to delete
    pub name: String,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineInstallArgs {
    /// Name of the machine to install
    pub machine: String,
    /// Target SSH host to install to
    pub target_host: String,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
    /// Confirm destructive operations (overwrites disk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineBuildArgs {
    /// Machine name to build
    pub machine: String,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
    /// Use nom for better build output (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_nom: Option<bool>,
}
