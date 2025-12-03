use crate::common::security::helpers::{
    audit_tool_execution, validation_error_to_mcp, with_timeout,
};
use crate::common::security::input_validation::validate_flake_ref;
use crate::common::security::{validate_machine_name, AuditLogger};
use rmcp::{
    handler::server::wrapper::Parameters, model::*, tool, tool_router, ErrorData as McpError,
};
use std::sync::Arc;

use super::types::{
    ClanMachineBuildArgs, ClanMachineCreateArgs, ClanMachineDeleteArgs, ClanMachineInstallArgs,
    ClanMachineListArgs, ClanMachineUpdateArgs,
};

pub struct MachineTools {
    audit: Arc<AuditLogger>,
}

impl MachineTools {
    pub fn new(audit: Arc<AuditLogger>) -> Self {
        Self { audit }
    }
}

#[tool_router]
impl MachineTools {
    #[tool(description = "Create a new Clan machine configuration")]
    pub async fn clan_machine_create(
        &self,
        Parameters(ClanMachineCreateArgs {
            name,
            template,
            target_host,
            flake,
        }): Parameters<ClanMachineCreateArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate machine name
        validate_machine_name(&name).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 60s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_machine_create",
            Some(serde_json::json!({"name": &name, "flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_machine_create", 60, || async {
                    let mut args = vec!["machines", "create", &name];

                    let template_str = template.unwrap_or_else(|| "new-machine".to_string());
                    args.push("-t");
                    args.push(&template_str);

                    args.push("--flake");
                    args.push(&flake_str);

                    let target_host_str;
                    if let Some(ref host) = target_host {
                        target_host_str = host.clone();
                        args.push("--target-host");
                        args.push(&target_host_str);
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
                            "Failed to create machine '{}':\n\n{}{}",
                            name, stdout, stderr
                        ))]));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Successfully created machine '{}'.\n\n{}{}",
                        name, stdout, stderr
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "List all Clan machines in the flake",
        annotations(read_only_hint = true)
    )]
    pub async fn clan_machine_list(
        &self,
        Parameters(ClanMachineListArgs { flake }): Parameters<ClanMachineListArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Execute with security features (audit logging + 30s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_machine_list",
            Some(serde_json::json!({"flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_machine_list", 30, || async {
                    let output = tokio::process::Command::new("clan")
                        .args(["machines", "list", "--flake", &flake_str])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(format!("Failed to execute clan: {}", e), None)
                        })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !output.status.success() {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Failed to list machines:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    let result = if stdout.trim().is_empty() {
                        "No machines configured in this Clan flake.".to_string()
                    } else {
                        format!("Clan Machines:\n\n{}", stdout)
                    };

                    Ok(CallToolResult::success(vec![Content::text(result)]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Update Clan machine(s) - rebuilds and deploys configuration",
        annotations(destructive_hint = true)
    )]
    pub async fn clan_machine_update(
        &self,
        Parameters(ClanMachineUpdateArgs { machines, flake }): Parameters<ClanMachineUpdateArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Validate machine names if provided
        if let Some(ref m) = machines {
            for machine in m {
                validate_machine_name(machine).map_err(validation_error_to_mcp)?;
            }
        }

        // Log dangerous operation
        let machines_desc = machines
            .as_ref()
            .map(|m| m.join(", "))
            .unwrap_or_else(|| "all machines".to_string());
        self.audit.log_dangerous_operation(
            "clan_machine_update",
            true,
            &format!("Updating machines: {}", machines_desc),
        );

        // Execute with security features (audit logging + 300s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_machine_update",
            Some(serde_json::json!({"machines": &machines, "flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_machine_update", 300, || async {
                    let mut args = vec!["machines", "update"];

                    args.push("--flake");
                    args.push(&flake_str);

                    let machine_names: Vec<String>;
                    if let Some(ref m) = machines {
                        machine_names = m.clone();
                        for machine in &machine_names {
                            args.push(machine);
                        }
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
                            "Machine update failed:\n\n{}{}",
                            stdout, stderr
                        ))]));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Machine update completed.\n\n{}{}",
                        stdout, stderr
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Delete a Clan machine configuration",
        annotations(destructive_hint = true)
    )]
    pub async fn clan_machine_delete(
        &self,
        Parameters(ClanMachineDeleteArgs { name, flake }): Parameters<ClanMachineDeleteArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate machine name
        validate_machine_name(&name).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Log dangerous operation
        self.audit.log_dangerous_operation(
            "clan_machine_delete",
            true,
            &format!("Deleting machine: {}", name),
        );

        // Execute with security features (audit logging + 60s timeout)
        audit_tool_execution(
            &self.audit,
            "clan_machine_delete",
            Some(serde_json::json!({"name": &name, "flake": &flake_str})),
            || async {
                with_timeout(&self.audit, "clan_machine_delete", 60, || async {
                    let output = tokio::process::Command::new("clan")
                        .args(["machines", "delete", &name, "--flake", &flake_str])
                        .output()
                        .await
                        .map_err(|e| {
                            McpError::internal_error(format!("Failed to execute clan: {}", e), None)
                        })?;

                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    if !output.status.success() {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "Failed to delete machine '{}':\n\n{}{}",
                            name, stdout, stderr
                        ))]));
                    }

                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "Successfully deleted machine '{}'.\n\n{}{}",
                        name, stdout, stderr
                    ))]))
                })
                .await
            },
        )
        .await
    }

    #[tool(
        description = "Install Clan machine to a target host via SSH (WARNING: Destructive - overwrites disk)",
        annotations(destructive_hint = true)
    )]
    pub async fn clan_machine_install(
        &self,
        Parameters(ClanMachineInstallArgs {
            machine,
            target_host,
            flake,
            confirm,
        }): Parameters<ClanMachineInstallArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Validate machine name
        validate_machine_name(&machine).map_err(validation_error_to_mcp)?;

        // Validate flake ref if provided
        let flake_str = flake.unwrap_or_else(|| ".".to_string());
        validate_flake_ref(&flake_str).map_err(validation_error_to_mcp)?;

        // Require user confirmation for this destructive operation
        if !confirm.unwrap_or(false) {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "WARNING: Installing machine '{}' to '{}' will OVERWRITE THE DISK!\n\n\
                    This is a destructive operation that will:\n\
                    - Partition and format the target disk\n\
                    - Install NixOS\n\
                    - Deploy the Clan configuration\n\n\
                    To proceed, call this function again with confirm=true",
                machine, target_host
            ))]));
        }

        // Log dangerous operation approval
        self.audit.log_dangerous_operation(
            "clan_machine_install",
            true,
            &format!(
                "Installing machine '{}' to '{}' (user confirmed)",
                machine, target_host
            ),
        );

        // Execute with security features (audit logging + 600s timeout for install)
        audit_tool_execution(&self.audit, "clan_machine_install", Some(serde_json::json!({"machine": &machine, "target_host": &target_host, "flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_machine_install", 600, || async {
                let output = tokio::process::Command::new("clan")
                    .args(["machines", "install", &machine, &target_host, "--flake", &flake_str])
                    .output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute clan: {}", e), None))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!("Machine installation failed:\n\n{}{}", stdout, stderr)
            )]));
        }

        Ok(CallToolResult::success(vec![Content::text(
            format!("Machine '{}' successfully installed to '{}'.\n\n{}{}", machine, target_host, stdout, stderr)
        )]))
            }).await
        }).await
    }

    #[tool(
        description = "Build a Clan machine configuration locally for testing without deployment"
    )]
    pub async fn clan_machine_build(
        &self,
        Parameters(ClanMachineBuildArgs {
            machine,
            flake,
            use_nom,
        }): Parameters<ClanMachineBuildArgs>,
    ) -> Result<CallToolResult, McpError> {
        let flake_str = flake.unwrap_or_else(|| ".".to_string());

        audit_tool_execution(&self.audit, "clan_machine_build", Some(serde_json::json!({"machine": &machine, "flake": &flake_str})), || async {
            with_timeout(&self.audit, "clan_machine_build", 300, || async {
                let use_nom = use_nom.unwrap_or(false);
                let build_target = format!(".#nixosConfigurations.{}.config.system.build.toplevel", machine);

                let mut cmd = if use_nom {
                    // Check if nom is available
                    let nom_check = tokio::process::Command::new("which")
                        .arg("nom")
                        .output()
                        .await;

                    if nom_check.is_ok() && nom_check.unwrap().status.success() {
                        let mut c = tokio::process::Command::new("nom");
                        c.args(["build", &build_target]);
                        c
                    } else {
                        let mut c = tokio::process::Command::new("nix");
                        c.args(["build", &build_target]);
                        c
                    }
                } else {
                    let mut c = tokio::process::Command::new("nix");
                    c.args(["build", &build_target]);
                    c
                };

                cmd.current_dir(&flake_str);

                let output = cmd.output()
                    .await
                    .map_err(|e| McpError::internal_error(format!("Failed to execute build command: {}", e), None))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Ok(CallToolResult::success(vec![Content::text(
                        format!("Build failed for machine '{}':\n\n{}{}", machine, stdout, stderr)
                    )]));
                }

                Ok(CallToolResult::success(vec![Content::text(
                    format!("Successfully built machine '{}' configuration.\n\n{}{}\n\nThe build result is in ./result/", machine, stdout, stderr)
                )]))
            }).await
        }).await
    }
}
