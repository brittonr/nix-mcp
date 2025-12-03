use crate::common::security::helpers::{audit_tool_execution, with_timeout};
use crate::common::security::{validate_flake_ref, validation_error_to_mcp, AuditLogger};
use rmcp::{
    handler::server::wrapper::Parameters, model::*, tool, tool_router, ErrorData as McpError,
};
use std::sync::Arc;

use super::types::{
    ClanAnalyzeRosterArgs, ClanAnalyzeSecretsArgs, ClanAnalyzeTagsArgs, ClanAnalyzeVarsArgs,
    ClanFlakeCreateArgs, ClanSecretsListArgs, ClanVmCreateArgs,
};

/// Tools for analyzing Clan infrastructure and managing flakes.
///
/// This struct provides operations for analyzing Clan infrastructure configurations,
/// understanding relationships between machines, secrets, and users, as well as
/// creating flakes and testing VMs. These tools help maintain and understand
/// complex Clan deployments.
///
/// # Available Operations
///
/// - **Infrastructure Analysis**: [`clan_analyze_secrets`](Self::clan_analyze_secrets), [`clan_analyze_vars`](Self::clan_analyze_vars), [`clan_analyze_tags`](Self::clan_analyze_tags), [`clan_analyze_roster`](Self::clan_analyze_roster)
/// - **Secret Management**: [`clan_secrets_list`](Self::clan_secrets_list)
/// - **Flake Management**: [`clan_flake_create`](Self::clan_flake_create)
/// - **Testing**: [`clan_vm_create`](Self::clan_vm_create)
/// - **Documentation**: [`clan_help`](Self::clan_help)
///
/// # Caching Strategy
///
/// No caching for analysis tools (infrastructure state changes frequently).
///
/// # Timeouts
///
/// - Analysis tools: 60 seconds (ACL, vars, tags, roster analysis)
/// - `clan_secrets_list`: 30 seconds (quick listing)
/// - `clan_flake_create`: 60 seconds (template creation)
/// - `clan_vm_create`: 600 seconds (10 minutes - VM build and launch)
/// - `clan_help`: No timeout (synchronous, read-only)
///
/// # Security
///
/// All operations include validation and logging:
/// - Flake references checked for shell metacharacters
/// - Machine names validated for hostname compliance
/// - All operations audited with parameters
///
/// # Analysis Tools
///
/// The analysis tools provide insights into Clan infrastructure:
/// - **Secrets**: Shows which machines have access to which secrets (ACLs)
/// - **Vars**: Shows variable ownership and usage across machines
/// - **Tags**: Shows tag assignments for organizing machines
/// - **Roster**: Shows user configuration and access across the infrastructure
///
/// # Examples
///
/// ```no_run
/// use onix_mcp::clan::AnalysisTools;
/// use onix_mcp::clan::types::ClanAnalyzeSecretsArgs;
/// use rmcp::handler::server::wrapper::Parameters;
/// use std::sync::Arc;
///
/// # async fn example(tools: AnalysisTools) -> Result<(), Box<dyn std::error::Error>> {
/// // Analyze secret ownership across machines
/// let result = tools.clan_analyze_secrets(Parameters(ClanAnalyzeSecretsArgs {
///     flake: Some(".".to_string()),
/// })).await?;
/// # Ok(())
/// # }
/// ```
pub struct AnalysisTools {
    audit: Arc<AuditLogger>,
}

impl AnalysisTools {
    /// Creates a new `AnalysisTools` instance with audit logging.
    ///
    /// # Arguments
    ///
    /// * `audit` - Shared audit logger for security event logging
    ///
    /// # Note
    ///
    /// AnalysisTools does not use caching as infrastructure analysis
    /// must reflect current state, which changes frequently.
    pub fn new(audit: Arc<AuditLogger>) -> Self {
        Self { audit }
    }
}

#[tool_router]
impl AnalysisTools {
    #[tool(description = "Analyze Clan secret (ACL) ownership across machines")]
    pub async fn clan_analyze_secrets(
        &self,
        Parameters(ClanAnalyzeSecretsArgs { flake }): Parameters<ClanAnalyzeSecretsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        // Validate flake path to prevent path traversal
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        audit_tool_execution(&self.audit, "clan_analyze_secrets", Some(serde_json::json!({"flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_analyze_secrets", 60, || async {
                // Try local flake first, then fall back to onix-core
                let mut cmd = tokio::process::Command::new("sh");
                cmd.args(["-c", &format!(
                    "cd {} && (nix run .#acl 2>/dev/null || nix run github:onixcomputer/onix-core#acl) 2>&1",
                    flake_str
                )]);

                let output = cmd.output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute acl command: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("ACL analysis failed.\n\nError:\n{}{}", stdout, stderr)
                    )]));
                }

                Ok(CallToolResult::success(vec![Content::text(
                    format!("Clan Secret (ACL) Ownership Analysis:\n\n{}{}", stdout, stderr)
                )]))
            }).await
        }).await
    }

    #[tool(description = "Analyze Clan vars ownership across machines")]
    pub async fn clan_analyze_vars(
        &self,
        Parameters(ClanAnalyzeVarsArgs { flake }): Parameters<ClanAnalyzeVarsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        // Validate flake path to prevent path traversal
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        audit_tool_execution(&self.audit, "clan_analyze_vars", Some(serde_json::json!({"flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_analyze_vars", 60, || async {
                let mut cmd = tokio::process::Command::new("sh");
                cmd.args(["-c", &format!(
                    "cd {} && (nix run .#vars 2>/dev/null || nix run github:onixcomputer/onix-core#vars) 2>&1",
                    flake_str
                )]);

                let output = cmd.output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute vars command: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("Vars analysis failed.\n\nError:\n{}{}", stdout, stderr)
                    )]));
                }

                Ok(CallToolResult::success(vec![Content::text(
                    format!("Clan Vars Ownership Analysis:\n\n{}{}", stdout, stderr)
                )]))
            }).await
        }).await
    }

    #[tool(description = "Analyze Clan machine tags across the infrastructure")]
    pub async fn clan_analyze_tags(
        &self,
        Parameters(ClanAnalyzeTagsArgs { flake }): Parameters<ClanAnalyzeTagsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        // Validate flake path to prevent path traversal
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        audit_tool_execution(&self.audit, "clan_analyze_tags", Some(serde_json::json!({"flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_analyze_tags", 60, || async {
                let mut cmd = tokio::process::Command::new("sh");
                cmd.args(["-c", &format!(
                    "cd {} && (nix run .#tags 2>/dev/null || nix run github:onixcomputer/onix-core#tags) 2>&1",
                    flake_str
                )]);

                let output = cmd.output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute tags command: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("Tags analysis failed.\n\nError:\n{}{}", stdout, stderr)
                    )]));
                }

                Ok(CallToolResult::success(vec![Content::text(
                    format!("Clan Machine Tags Analysis:\n\n{}{}", stdout, stderr)
                )]))
            }).await
        }).await
    }

    #[tool(description = "Analyze Clan user roster configurations")]
    pub async fn clan_analyze_roster(
        &self,
        Parameters(ClanAnalyzeRosterArgs { flake }): Parameters<ClanAnalyzeRosterArgs>,
    ) -> Result<CallToolResult, McpError> {
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        // Validate flake path to prevent path traversal
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        audit_tool_execution(&self.audit, "clan_analyze_roster", Some(serde_json::json!({"flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_analyze_roster", 60, || async {
                let mut cmd = tokio::process::Command::new("sh");
                cmd.args(["-c", &format!(
                    "cd {} && (nix run .#roster 2>/dev/null || nix run github:onixcomputer/onix-core#roster) 2>&1",
                    flake_str
                )]);

                let output = cmd.output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute roster command: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("Roster analysis failed.\n\nError:\n{}{}", stdout, stderr)
                    )]));
                }

                Ok(CallToolResult::success(vec![Content::text(
                    format!("Clan User Roster Analysis:\n\n{}{}", stdout, stderr)
                )]))
            }).await
        }).await
    }

    #[tool(
        description = "List secrets in a Clan flake",
        annotations(read_only_hint = true)
    )]
    pub async fn clan_secrets_list(
        &self,
        Parameters(ClanSecretsListArgs { flake }): Parameters<ClanSecretsListArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::{validate_flake_ref, validation_error_to_mcp};

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_secrets_list",
            Some(serde_json::json!({"flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_secrets_list", 30, || async {
                    let output = tokio::process::Command::new("clan")
                        .args(["secrets", "list", "--flake", &flake_str])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(format!("Failed to execute clan: {}", e), None)
                        })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !output.status.success() {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Failed to list secrets:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    let result = if stdout.trim().is_empty() {
                        "No secrets configured.".to_string()
                    } else {
                        format!("Clan Secrets:\n\n{}", stdout)
                    };

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(description = "Create a new Clan flake from a template")]
    pub async fn clan_flake_create(
        &self,
        Parameters(ClanFlakeCreateArgs {
            directory,
            template,
        }): Parameters<ClanFlakeCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::{validate_path, validation_error_to_mcp};

        // Validate directory path
        validate_path(&directory).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 60s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_flake_create",
            Some(serde_json::json!({"directory": &directory})),
            || async {
                with_timeout(&self.audit, "clan_flake_create", 60, || async {
                    let mut args = vec!["flakes", "create", &directory];

                    let template_str;
                    if let Some(ref t) = template {
                        template_str = t.clone();
                        args.push("--template");
                        args.push(&template_str);
                    }

                    let output = tokio::process::Command::new("clan")
                        .args(&args)
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(format!("Failed to execute clan: {}", e), None)
                        })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !output.status.success() {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Failed to create Clan flake:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Clan flake created in '{}'.\n\n{}{}",
                        directory, stdout, stderr
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(description = "Create and run a VM for a Clan machine (useful for testing)")]
    pub async fn clan_vm_create(
        &self,
        Parameters(ClanVmCreateArgs { machine, flake }): Parameters<ClanVmCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::{
            validate_flake_ref, validate_machine_name, validation_error_to_mcp,
        };

        // Validate machine name
        validate_machine_name(&machine).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 120s timeout)
        audit_tool_execution(&self.audit, "clan_vm_create", Some(serde_json::json!({"machine": &machine, "flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_vm_create", 120, || async {
                let output = tokio::process::Command::new("clan")
                    .args(["vms", "create", &machine, "--flake", &flake_str])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute clan: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("VM creation failed:\n\n{}{}", stdout, stderr)
                    )]));
                }

                Ok(CallToolResult::success(vec![Content::text(
                    format!("VM created for machine '{}'.\n\n{}{}\n\nNote: This creates a VM configuration. Use 'clan vms run {}' to start it.", machine, stdout, stderr, machine)
                )]))
            }).await
        }).await
    }

    #[tool(
        description = "Get help and information about Clan - the peer-to-peer NixOS management framework"
    )]
    pub fn clan_help(
        &self,
        Parameters(_args): Parameters<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<CallToolResult, McpError> {
        let help_text = r#"Clan - Peer-to-Peer NixOS Management Framework

Clan is a framework built on NixOS that enables declarative, collaborative management
of distributed systems. It provides tools for managing machines, backups, secrets, and more.

KEY CONCEPTS:

1. Clan Flake
   A Git repository containing your infrastructure as code:
   - Machine configurations
   - Shared services and modules
   - Secrets and variables
   - Network topology

2. Machines
   Individual systems managed by Clan. Each machine has:
   - Hardware configuration
   - NixOS configuration
   - Service definitions
   - Access to shared secrets

3. Services
   Modular components that add functionality:
   - Networking (VPN, mesh networks)
   - Backups (automated, versioned)
   - Monitoring and observability
   - Custom application stacks

AVAILABLE TOOLS:

Machine Management:
- clan_machine_create - Create new machine configurations
- clan_machine_list - List all machines in flake
- clan_machine_update - Update/deploy machine configurations
- clan_machine_delete - Remove machine configurations
- clan_machine_install - Install NixOS to a remote host (destructive!)
- clan_machine_build - Build machine configuration locally for testing

Backup Operations:
- clan_backup_create - Create backups for machines
- clan_backup_list - List available backups
- clan_backup_restore - Restore from backup

Flake & Project:
- clan_flake_create - Initialize new Clan project

Secrets:
- clan_secrets_list - View configured secrets

Testing & Building:
- clan_vm_create - Create VMs for testing configurations
- nixos_build - Build NixOS configurations from flakes

Analysis Tools:
- clan_analyze_secrets - Analyze secret (ACL) ownership across machines
- clan_analyze_vars - Analyze vars ownership across machines
- clan_analyze_tags - Analyze machine tags
- clan_analyze_roster - Analyze user roster configurations

COMMON WORKFLOWS:

1. Creating a New Clan Project:
   clan_flake_create(directory="my-infrastructure")

2. Adding a Machine:
   clan_machine_create(name="webserver", target_host="192.168.1.10")

3. Deploying to Production:
   clan_machine_install(machine="webserver", target_host="192.168.1.10", confirm=true)

4. Regular Updates:
   clan_machine_update(machines=["webserver"])

5. Backup & Restore:
   clan_backup_create(machine="webserver")
   clan_backup_list(machine="webserver")
   clan_backup_restore(machine="webserver", provider="borgbackup", name="2024-12-01")

DOCUMENTATION:
- Main docs: https://docs.clan.lol
- Repository: https://git.clan.lol/clan/clan-core
- Option search: https://docs.clan.lol/option-search/

BENEFITS:
- Declarative infrastructure (everything in Git)
- Peer-to-peer collaboration
- Reproducible builds (Nix)
- Integrated backups and secrets
- Testing with VMs before deployment
"#;

        Ok(CallToolResult::success(vec![Content::text(help_text)]))
    }
}
