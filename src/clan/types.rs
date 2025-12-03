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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanBackupCreateArgs {
    /// Machine name to backup
    pub machine: String,
    /// Optional backup provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanBackupListArgs {
    /// Machine name to list backups for
    pub machine: String,
    /// Optional backup provider to filter by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanBackupRestoreArgs {
    /// Machine name to restore backup to
    pub machine: String,
    /// Backup provider
    pub provider: String,
    /// Backup name/identifier
    pub name: String,
    /// Optional service to restore (restore all if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}
