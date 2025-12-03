//! Clan infrastructure management tools module.
//!
//! This module provides MCP tools for managing NixOS infrastructure using
//! [Clan](https://docs.clan.lol), a peer-to-peer NixOS management framework
//! for declarative infrastructure deployment.
//!
//! # Module Organization
//!
//! - [`machines`] - Machine lifecycle management (create, update, delete, install, build)
//! - [`backups`] - Backup operations (create, list, restore)
//! - [`analysis`] - Infrastructure analysis (secrets, vars, tags, roster, flakes, VMs)
//!
//! # Clan Workflow
//!
//! Typical Clan workflow using these tools:
//!
//! 1. **Initialize**: Create Clan flake with [`AnalysisTools::clan_flake_create`]
//! 2. **Define Machines**: Add machines with [`MachineTools::clan_machine_create`]
//! 3. **Test Locally**: Build with [`MachineTools::clan_machine_build`]
//! 4. **Deploy**: Update machines with [`MachineTools::clan_machine_update`]
//! 5. **Backup**: Create backups with [`BackupTools::clan_backup_create`]
//! 6. **Analyze**: Review infrastructure with analysis tools
//!
//! # Security
//!
//! All Clan operations include:
//! - Flake path validation to prevent path traversal
//! - Machine name validation to prevent injection attacks
//! - Audit logging for all destructive operations
//! - Confirmation prompts for dangerous operations (install, delete)
//!
//! # Examples
//!
//! ```no_run
//! use onix_mcp::clan::{MachineTools, ClanMachineListArgs};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create machine tools
//! let audit = Arc::new(/* audit logger */);
//! let tools = MachineTools::new(audit);
//!
//! // List all machines in the current Clan flake
//! // let result = tools.clan_machine_list(Parameters(ClanMachineListArgs {
//! //     flake: None, // Uses current directory
//! // })).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Clan.lol Overview
//!
//! Clan provides:
//! - Declarative multi-machine NixOS deployments
//! - Peer-to-peer architecture (no central server required)
//! - Integrated secrets management (sops)
//! - Automated backups and restore
//! - Tags and roster for machine organization
//! - VM testing for configurations

pub mod analysis;
pub mod backups;
pub mod machines;
pub mod types;

pub use analysis::AnalysisTools;
pub use backups::BackupTools;
pub use machines::MachineTools;
pub use types::{
    ClanAnalyzeRosterArgs, ClanAnalyzeSecretsArgs, ClanAnalyzeTagsArgs, ClanAnalyzeVarsArgs,
    ClanBackupCreateArgs, ClanBackupListArgs, ClanBackupRestoreArgs, ClanFlakeCreateArgs,
    ClanMachineBuildArgs, ClanMachineCreateArgs, ClanMachineDeleteArgs, ClanMachineInstallArgs,
    ClanMachineListArgs, ClanMachineUpdateArgs, ClanSecretsListArgs, ClanVmCreateArgs,
};
