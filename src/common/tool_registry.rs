use crate::common::cache_registry::CacheRegistry;
use crate::common::security::AuditLogger;
use std::sync::Arc;

/// Central registry for all tool modules in the MCP server.
///
/// This struct consolidates all specialized tool implementations,
/// making it easier to manage dependencies and maintain the server.
#[derive(Clone)]
pub struct ToolRegistry {
    // Development tools
    pub precommit: Arc<crate::dev::PreCommitTools>,

    // Process management tools
    pub pexpect: Arc<crate::process::PexpectTools>,
    pub pueue: Arc<crate::process::PueueTools>,

    // Nix ecosystem tools
    pub info: Arc<crate::nix::InfoTools>,
    pub package: Arc<crate::nix::PackageTools>,
    pub build: Arc<crate::nix::BuildTools>,
    pub develop: Arc<crate::nix::DevelopTools>,
    pub flake: Arc<crate::nix::FlakeTools>,
    pub quality: Arc<crate::nix::QualityTools>,

    // Clan infrastructure tools
    pub machine: Arc<crate::clan::MachineTools>,
    pub backup: Arc<crate::clan::BackupTools>,
    pub analysis: Arc<crate::clan::AnalysisTools>,

    // Prompts
    pub prompts: Arc<crate::prompts::NixPrompts>,
}

impl ToolRegistry {
    /// Creates a new ToolRegistry with all tool modules initialized.
    ///
    /// # Arguments
    /// * `audit` - Shared audit logger for security logging
    /// * `caches` - Shared cache registry for all caching needs
    pub fn new(audit: Arc<AuditLogger>, caches: Arc<CacheRegistry>) -> Self {
        Self {
            // Development tools - only need audit
            precommit: Arc::new(crate::dev::PreCommitTools::new(audit.clone())),

            // Process tools - only need audit
            pexpect: Arc::new(crate::process::PexpectTools::new(audit.clone())),
            pueue: Arc::new(crate::process::PueueTools::new(audit.clone())),

            // Nix info tools - only need audit
            info: Arc::new(crate::nix::InfoTools::new(audit.clone())),

            // Nix tools that use caching
            package: Arc::new(crate::nix::PackageTools::new(audit.clone(), caches.clone())),
            build: Arc::new(crate::nix::BuildTools::new(audit.clone(), caches.clone())),
            develop: Arc::new(crate::nix::DevelopTools::new(audit.clone(), caches.clone())),
            flake: Arc::new(crate::nix::FlakeTools::new(audit.clone(), caches.clone())),

            // Nix quality tools - only need audit
            quality: Arc::new(crate::nix::QualityTools::new(audit.clone())),

            // Clan infrastructure tools - only need audit
            machine: Arc::new(crate::clan::MachineTools::new(audit.clone())),
            backup: Arc::new(crate::clan::BackupTools::new(audit.clone())),
            analysis: Arc::new(crate::clan::AnalysisTools::new(audit.clone())),

            // Prompts - no dependencies
            prompts: Arc::new(crate::prompts::NixPrompts::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::security::audit_logger;

    #[test]
    fn test_tool_registry_creation() {
        let audit = audit_logger();
        let caches = Arc::new(CacheRegistry::new());

        let registry = ToolRegistry::new(audit, caches);

        // Verify all tool instances are initialized
        assert!(Arc::strong_count(&registry.precommit) >= 1);
        assert!(Arc::strong_count(&registry.pexpect) >= 1);
        assert!(Arc::strong_count(&registry.pueue) >= 1);
        assert!(Arc::strong_count(&registry.info) >= 1);
        assert!(Arc::strong_count(&registry.package) >= 1);
        assert!(Arc::strong_count(&registry.build) >= 1);
        assert!(Arc::strong_count(&registry.develop) >= 1);
        assert!(Arc::strong_count(&registry.flake) >= 1);
        assert!(Arc::strong_count(&registry.quality) >= 1);
        assert!(Arc::strong_count(&registry.machine) >= 1);
        assert!(Arc::strong_count(&registry.backup) >= 1);
        assert!(Arc::strong_count(&registry.analysis) >= 1);
        assert!(Arc::strong_count(&registry.prompts) >= 1);
    }

    #[test]
    fn test_tool_registry_clone() {
        let audit = audit_logger();
        let caches = Arc::new(CacheRegistry::new());

        let registry1 = ToolRegistry::new(audit, caches);
        let registry2 = registry1.clone();

        // Verify that cloning increases Arc reference counts
        assert!(Arc::strong_count(&registry1.package) >= 2);
        assert!(Arc::strong_count(&registry2.package) >= 2);

        // Verify both registries point to the same tool instances
        assert!(Arc::ptr_eq(&registry1.package, &registry2.package));
    }
}
