//! Parameter types for Clan MCP tools.
//!
//! This module defines all parameter types used by the Clan infrastructure management
//! tools. Each type corresponds to a specific Clan operation and includes field-level
//! documentation with examples.

use rmcp::schemars;

/// Parameters for creating a new Clan machine configuration.
///
/// Used by [`MachineTools::clan_machine_create`](crate::clan::MachineTools::clan_machine_create).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanMachineCreateArgs;
///
/// let args = ClanMachineCreateArgs {
///     name: "webserver".to_string(),
///     template: Some("new-machine".to_string()),
///     target_host: Some("192.168.1.10".to_string()),
///     flake: None,
/// };
/// ```
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

/// Parameters for listing all Clan machines in a flake.
///
/// Used by [`MachineTools::clan_machine_list`](crate::clan::MachineTools::clan_machine_list).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanMachineListArgs;
///
/// let args = ClanMachineListArgs {
///     flake: Some(".".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineListArgs {
    /// Optional flake directory path (default: current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

/// Parameters for updating Clan machine configurations.
///
/// Used by [`MachineTools::clan_machine_update`](crate::clan::MachineTools::clan_machine_update).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanMachineUpdateArgs;
///
/// // Update all machines
/// let args = ClanMachineUpdateArgs {
///     machines: None,
///     flake: None,
/// };
///
/// // Update specific machines
/// let args = ClanMachineUpdateArgs {
///     machines: Some(vec!["web1".to_string(), "web2".to_string()]),
///     flake: Some(".".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineUpdateArgs {
    /// Machines to update (empty for all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machines: Option<Vec<String>>,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

/// Parameters for deleting a Clan machine configuration.
///
/// Used by [`MachineTools::clan_machine_delete`](crate::clan::MachineTools::clan_machine_delete).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanMachineDeleteArgs;
///
/// let args = ClanMachineDeleteArgs {
///     name: "old-server".to_string(),
///     flake: None,
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanMachineDeleteArgs {
    /// Name of the machine to delete
    pub name: String,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

/// Parameters for installing a Clan machine to a target host.
///
/// Used by [`MachineTools::clan_machine_install`](crate::clan::MachineTools::clan_machine_install).
///
/// **WARNING**: This operation is destructive and overwrites the target disk.
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanMachineInstallArgs;
///
/// let args = ClanMachineInstallArgs {
///     machine: "webserver".to_string(),
///     target_host: "root@192.168.1.10".to_string(),
///     flake: None,
///     confirm: Some(true),
/// };
/// ```
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

/// Parameters for building a Clan machine configuration locally.
///
/// Used by [`MachineTools::clan_machine_build`](crate::clan::MachineTools::clan_machine_build).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanMachineBuildArgs;
///
/// let args = ClanMachineBuildArgs {
///     machine: "webserver".to_string(),
///     flake: None,
///     use_nom: Some(true),
/// };
/// ```
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

/// Parameters for creating a backup of a Clan machine.
///
/// Used by [`BackupTools::clan_backup_create`](crate::clan::BackupTools::clan_backup_create).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanBackupCreateArgs;
///
/// let args = ClanBackupCreateArgs {
///     machine: "webserver".to_string(),
///     provider: Some("local".to_string()),
///     flake: None,
/// };
/// ```
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

/// Parameters for listing backups of a Clan machine.
///
/// Used by [`BackupTools::clan_backup_list`](crate::clan::BackupTools::clan_backup_list).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanBackupListArgs;
///
/// let args = ClanBackupListArgs {
///     machine: "webserver".to_string(),
///     provider: None,
///     flake: None,
/// };
/// ```
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

/// Parameters for restoring a backup to a Clan machine.
///
/// Used by [`BackupTools::clan_backup_restore`](crate::clan::BackupTools::clan_backup_restore).
///
/// **WARNING**: This operation is destructive and overwrites data.
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanBackupRestoreArgs;
///
/// let args = ClanBackupRestoreArgs {
///     machine: "webserver".to_string(),
///     provider: "local".to_string(),
///     name: "backup-2024-01-01".to_string(),
///     service: Some("nginx".to_string()),
///     flake: None,
/// };
/// ```
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

/// Parameters for creating a new Clan flake from a template.
///
/// Used by [`AnalysisTools::clan_flake_create`](crate::clan::AnalysisTools::clan_flake_create).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanFlakeCreateArgs;
///
/// let args = ClanFlakeCreateArgs {
///     directory: "./my-infrastructure".to_string(),
///     template: Some("minimal".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanFlakeCreateArgs {
    /// Directory to create the Clan flake in
    pub directory: String,
    /// Optional template to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

/// Parameters for listing secrets in a Clan flake.
///
/// Used by [`AnalysisTools::clan_secrets_list`](crate::clan::AnalysisTools::clan_secrets_list).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanSecretsListArgs;
///
/// let args = ClanSecretsListArgs {
///     flake: Some(".".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanSecretsListArgs {
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

/// Parameters for creating a VM for testing a Clan machine.
///
/// Used by [`AnalysisTools::clan_vm_create`](crate::clan::AnalysisTools::clan_vm_create).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanVmCreateArgs;
///
/// let args = ClanVmCreateArgs {
///     machine: "webserver".to_string(),
///     flake: None,
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanVmCreateArgs {
    /// Machine name to create VM for
    pub machine: String,
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

/// Parameters for analyzing secret (ACL) ownership across machines.
///
/// Used by [`AnalysisTools::clan_analyze_secrets`](crate::clan::AnalysisTools::clan_analyze_secrets).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanAnalyzeSecretsArgs;
///
/// let args = ClanAnalyzeSecretsArgs {
///     flake: Some(".".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanAnalyzeSecretsArgs {
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

/// Parameters for analyzing variable ownership across machines.
///
/// Used by [`AnalysisTools::clan_analyze_vars`](crate::clan::AnalysisTools::clan_analyze_vars).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanAnalyzeVarsArgs;
///
/// let args = ClanAnalyzeVarsArgs {
///     flake: None,
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanAnalyzeVarsArgs {
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

/// Parameters for analyzing machine tag assignments.
///
/// Used by [`AnalysisTools::clan_analyze_tags`](crate::clan::AnalysisTools::clan_analyze_tags).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanAnalyzeTagsArgs;
///
/// let args = ClanAnalyzeTagsArgs {
///     flake: Some(".".to_string()),
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanAnalyzeTagsArgs {
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}

/// Parameters for analyzing user roster configurations.
///
/// Used by [`AnalysisTools::clan_analyze_roster`](crate::clan::AnalysisTools::clan_analyze_roster).
///
/// # Examples
///
/// ```
/// use onix_mcp::clan::types::ClanAnalyzeRosterArgs;
///
/// let args = ClanAnalyzeRosterArgs {
///     flake: None,
/// };
/// ```
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ClanAnalyzeRosterArgs {
    /// Optional flake directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flake: Option<String>,
}
