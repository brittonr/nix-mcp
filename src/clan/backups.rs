use crate::common::security::helpers::{
    audit_tool_execution, validation_error_to_mcp, with_timeout,
};
use crate::common::security::input_validation::validate_flake_ref;
use crate::common::security::{validate_machine_name, AuditLogger};
use rmcp::{
    handler::server::wrapper::Parameters, model::*, tool, tool_router, ErrorData as McpError,
};
use std::sync::Arc;

use super::types::{ClanBackupCreateArgs, ClanBackupListArgs, ClanBackupRestoreArgs};

pub struct BackupTools {
    audit: Arc<AuditLogger>,
}

impl BackupTools {
    pub fn new(audit: Arc<AuditLogger>) -> Self {
        Self { audit }
    }
}

#[tool_router]
impl BackupTools {
    #[tool(description = "Create a backup for a Clan machine")]
    pub async fn clan_backup_create(
        &self,
        Parameters(ClanBackupCreateArgs {
            machine,
            provider,
            flake,
        }): Parameters<ClanBackupCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_machine_name;

        // Validate machine name
        validate_machine_name(&machine).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 120s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_backup_create",
            Some(serde_json::json!({"machine": &machine, "flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_backup_create", 120, || async {
                    let mut args = vec!["backups", "create", &machine];

                    args.push("--flake");
                    args.push(&flake_str);

                    let provider_str;
                    if let Some(ref p) = provider {
                        provider_str = p.clone();
                        args.push("--provider");
                        args.push(&provider_str);
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
                            "Backup creation failed:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Backup created for machine '{}'.\n\n{}{}",
                        machine, stdout, stderr
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "List backups for a Clan machine",
        annotations(read_only_hint = true)
    )]
    pub async fn clan_backup_list(
        &self,
        Parameters(ClanBackupListArgs {
            machine,
            provider,
            flake,
        }): Parameters<ClanBackupListArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_machine_name;

        // Validate machine name
        validate_machine_name(&machine).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_backup_list",
            Some(serde_json::json!({"machine": &machine, "flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_backup_list", 30, || async {
                    let mut args = vec!["backups", "list", &machine];

                    args.push("--flake");
                    args.push(&flake_str);

                    let provider_str;
                    if let Some(ref p) = provider {
                        provider_str = p.clone();
                        args.push("--provider");
                        args.push(&provider_str);
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
                            "Failed to list backups:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    let result = if stdout.trim().is_empty() {
                        format!("No backups found for machine '{}'.", machine)
                    } else {
                        format!("Backups for machine '{}':\n\n{}", machine, stdout)
                    };

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Restore a backup for a Clan machine",
        annotations(destructive_hint = true)
    )]
    pub async fn clan_backup_restore(
        &self,
        Parameters(ClanBackupRestoreArgs {
            machine,
            provider,
            name,
            service,
            flake,
        }): Parameters<ClanBackupRestoreArgs>,
    ) -> Result<CallToolResult, McpError> {
        use crate::common::security::helpers::{audit_tool_execution, with_timeout};
        use crate::common::security::validate_machine_name;

        // Validate machine name
        validate_machine_name(&machine).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Validate backup name (basic alphanumeric check)
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(McpError::invalid_params(
                "Invalid backup name: must be non-empty alphanumeric with dashes, underscores, or dots",
                None,
            ));
        }

        // Log dangerous operation
        self.audit.log_dangerous_operation(
            "clan_backup_restore",
            true,
            &format!("Restoring backup '{}' for machine '{}'", name, machine),
        );

        // Execute with security features (audit logging + 120s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_backup_restore",
            Some(serde_json::json!({"machine": &machine, "backup": &name, "flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_backup_restore", 120, || async {
                    let mut args = vec!["backups", "restore", &machine, &provider, &name];

                    args.push("--flake");
                    args.push(&flake_str);

                    let service_str;
                    if let Some(ref s) = service {
                        service_str = s.clone();
                        args.push("--service");
                        args.push(&service_str);
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
                            "Backup restore failed:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Backup '{}' restored for machine '{}'.\n\n{}{}",
                        name, machine, stdout, stderr
                    ))]))
                })
                .await
            },
        )
        .await
    }
}
