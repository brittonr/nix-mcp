use crate::prompts::types::{
    MigrateToFlakesArgs, OptimizeClosureArgs, SetupDevEnvironmentArgs, TroubleshootBuildArgs,
};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{GetPromptResult, PromptMessage, PromptMessageContent, PromptMessageRole};
use rmcp::service::RequestContext;
use rmcp::ErrorData as McpError;
use rmcp::{prompt, prompt_router, RoleServer};

/// Nix-specific prompt generators for common tasks
pub struct NixPrompts;

impl NixPrompts {
    pub fn new() -> Self {
        Self
    }
}

#[prompt_router]
impl NixPrompts {
    /// Generate a nix flake template based on requirements
    #[prompt(name = "generate_flake")]
    pub async fn generate_flake(
        &self,
        Parameters(args): Parameters<serde_json::Map<String, serde_json::Value>>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<Vec<PromptMessage>, McpError> {
        let project_type = args
            .get("project_type")
            .and_then(|v| v.as_str())
            .unwrap_or("generic");

        let prompt = format!(
            "Generate a Nix flake.nix file for a {} project. Include appropriate buildInputs, development shell, and package definition.",
            project_type
        );

        Ok(vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::text(prompt),
        }])
    }

    /// Guide for setting up a Nix development environment for a specific project type
    #[prompt(name = "setup_dev_environment")]
    pub async fn setup_dev_environment(
        &self,
        Parameters(args): Parameters<SetupDevEnvironmentArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let use_flakes = args.use_flakes.unwrap_or(true);
        let deps = args
            .dependencies
            .as_ref()
            .map(|d| d.join(", "))
            .unwrap_or_else(|| "none specified".to_string());

        let messages = vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "I need to set up a Nix development environment for a {} project.\n\
                    Additional dependencies: {}\n\
                    Use flakes: {}\n\n\
                    Please provide:\n\
                    1. A complete flake.nix (if using flakes) or shell.nix file\n\
                    2. Explanation of the key components\n\
                    3. Commands to enter and use the development environment\n\
                    4. Best practices for this project type with Nix",
                args.project_type, deps, use_flakes
            ),
        )];

        Ok(GetPromptResult {
            description: Some(format!(
                "Setup {} development environment",
                args.project_type
            )),
            messages,
        })
    }

    /// Help troubleshoot Nix build failures with diagnostic guidance
    #[prompt(name = "troubleshoot_build")]
    pub async fn troubleshoot_build(
        &self,
        Parameters(args): Parameters<TroubleshootBuildArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let error_context = args
            .error_message
            .as_ref()
            .map(|e| format!("\n\nError message:\n{}", e))
            .unwrap_or_default();

        let messages = vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                format!(
                    "I'm having trouble building: {}{}\n\n\
                    Please help me:\n\
                    1. Identify the root cause of the build failure\n\
                    2. Suggest specific debugging commands to run (like nix log, nix why-depends, etc.)\n\
                    3. Provide potential solutions or workarounds\n\
                    4. Explain common patterns that might cause this issue\n\
                    5. Recommend preventive measures for the future",
                    args.package, error_context
                ),
            ),
        ];

        Ok(GetPromptResult {
            description: Some(format!("Troubleshoot build failure for {}", args.package)),
            messages,
        })
    }

    /// Guide for migrating existing projects to Nix flakes
    #[prompt(name = "migrate_to_flakes")]
    pub async fn migrate_to_flakes(
        &self,
        Parameters(args): Parameters<MigrateToFlakesArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let project_context = args
            .project_type
            .as_ref()
            .map(|p| format!(" for a {} project", p))
            .unwrap_or_default();

        let messages = vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "I want to migrate to Nix flakes{}.\n\
                    Current setup: {}\n\n\
                    Please provide:\n\
                    1. Step-by-step migration plan\n\
                    2. Example flake.nix based on my current setup\n\
                    3. How to handle inputs and lock files\n\
                    4. Common pitfalls to avoid\n\
                    5. Benefits I'll gain from using flakes\n\
                    6. Backward compatibility considerations",
                project_context, args.current_setup
            ),
        )];

        Ok(GetPromptResult {
            description: Some("Migrate to Nix flakes".to_string()),
            messages,
        })
    }

    /// Help optimize package closure size with actionable recommendations
    #[prompt(name = "optimize_closure")]
    pub async fn optimize_closure(
        &self,
        Parameters(args): Parameters<OptimizeClosureArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let size_context = args
            .current_size
            .as_ref()
            .map(|s| format!("\nCurrent closure size: {}", s))
            .unwrap_or_default();
        let target_context = args
            .target
            .as_ref()
            .map(|t| format!("\nTarget: {}", t))
            .unwrap_or_default();

        let messages = vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "I need to optimize the closure size for: {}{}{}\n\n\
                    Please help me:\n\
                    1. Analyze dependency tree to identify large dependencies\n\
                    2. Suggest specific packages or features to remove or replace\n\
                    3. Provide Nix expressions to create minimal variants\n\
                    4. Recommend build flags or overrides to reduce size\n\
                    5. Explain trade-offs between size and functionality\n\
                    6. Show how to measure and verify improvements",
                args.package, size_context, target_context
            ),
        )];

        Ok(GetPromptResult {
            description: Some(format!("Optimize closure for {}", args.package)),
            messages,
        })
    }
}
