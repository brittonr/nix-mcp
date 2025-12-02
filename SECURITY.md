# Security Policy for Onix-MCP

## Overview

This document describes the security model, threat analysis, and security features of the Onix-MCP Nix server.

## Security Model

### Trust Boundaries

**Boundary 1: MCP Client → Onix-MCP Server**
- **Trust Level**: Authenticated user
- **Transport**: stdio (process isolation) or HTTP (network)
- **Protection**: Input validation, audit logging, rate limiting

**Boundary 2: Onix-MCP Server → Nix/System**
- **Trust Level**: Trusted process with user permissions
- **Protection**: Input sanitization, command construction safety, timeouts

**Boundary 3: Onix-MCP Server → External Services**
- **Trust Level**: Untrusted network services
- **Protection**: URL validation, HTTPS enforcement, timeout handling

### Security Principles

1. **Deny by Default**: All operations require explicit validation
2. **Defense in Depth**: Multiple security layers (validation → sanitization → audit)
3. **Least Privilege**: Tools declare safety characteristics (read-only, destructive)
4. **Fail Securely**: Validation failures are logged and rejected
5. **Audit Everything**: All tool invocations and security events are logged

## Threat Model

### In-Scope Threats

1. **Command Injection**: Malicious input attempting to execute arbitrary commands
   - **Mitigation**: Input validation, command argument separation, no shell execution

2. **Path Traversal**: Attempts to access files outside allowed directories
   - **Mitigation**: Path canonicalization, parent directory blocking

3. **Resource Exhaustion**: DoS via expensive operations
   - **Mitigation**: Timeouts, rate limiting, cancellation support

4. **Information Disclosure**: Leaking sensitive data in logs or errors
   - **Mitigation**: Secret redaction, structured error messages

5. **Privilege Escalation**: Attempting unauthorized operations
   - **Mitigation**: Tool safety annotations, operation confirmation for destructive actions

### Out-of-Scope Threats

- Physical access to the host machine
- Kernel vulnerabilities
- Nix sandbox escape (trust Nix's security model)
- Side-channel attacks
- Social engineering

## Security Features

### 1. Input Validation

All user inputs are validated before use:

**Package Names**
- Pattern: `^[a-zA-Z0-9_][a-zA-Z0-9_\-\.]*$`
- Max length: 255 characters
- No path traversal patterns (`..`, `/`, `\`)

**Flake References**
- Supports: relative paths, absolute paths, Git URLs, registry names
- Max length: 1000 characters
- No shell metacharacters (`;`, `|`, `` ` ``, `$`, etc.)
- No null bytes

**Nix Expressions**
- Max length: 10,000 characters
- Blocked patterns: `__noChroot`, `trustedUsers`, dangerous builtins
- No shell command substitution (`$(...)`, `` `...` ``)

**Machine Names (Clan)**
- Pattern: `^[a-zA-Z0-9_\-]+$`
- Max length: 63 characters (hostname rules)
- Cannot start or end with hyphen

**URLs**
- Must be `http://`, `https://`, or `ftp://`
- Max length: 2048 characters
- No unencoded spaces or null bytes

**Commands (nix-shell)**
- Max length: 1000 characters
- Warning logged for dangerous patterns (`rm -rf`, `dd if=`, etc.)
- No null bytes

### 2. Audit Logging

All security-relevant events are logged to structured logs (stderr):

**Logged Events**
- Tool invocations (with parameters, duration, success/failure)
- Input validation failures
- Suspicious activity
- Rate limit violations
- Operation timeouts
- Authentication events (future)
- Dangerous operations

**Log Format**
```json
{
  "timestamp": "2025-12-02T10:30:00Z",
  "level": "INFO",
  "security_level": "warning",
  "event": {
    "event_type": "ToolInvoked",
    "tool_name": "nix_build",
    "parameters": {"package": "nixpkgs#hello"},
    "success": true,
    "duration_ms": 2500
  }
}
```

**Security Levels**
- `Info`: Normal operations
- `Warning`: Suspicious but allowed
- `Error`: Security violation
- `Critical`: Serious security breach attempt

### 3. Timeout Protection

All operations have enforced timeouts:

| Operation Type | Timeout | Rationale |
|----------------|---------|-----------|
| Search/Query   | 30s     | Network + local search |
| Build          | 300s    | Nix builds can be slow |
| Eval           | 30s     | Prevent infinite loops |
| Shell command  | 60s     | User commands |
| Default        | 120s    | Conservative default |

Timeouts can be cancelled early by clients using cancellation tokens.

### 4. Tool Safety Annotations

Tools declare their safety characteristics:

**Read-Only Tools** (`readOnlyHint: true`)
- `search_packages`, `get_package_info`, `search_options`
- `flake_metadata`, `flake_show`
- `get_closure_size`, `why_depends`, `show_derivation`
- Safe to call repeatedly, no side effects

**Idempotent Tools** (`idempotentHint: true`)
- `format_nix`, `validate_nix`, `lint_nix`
- Safe to retry on failure

**Destructive Tools** (`destructiveHint: true`)
- `clan_machine_install` - Formats disks, destructive
- `clan_machine_delete` - Removes machine configuration
- `clan_backup_restore` - Overwrites data
- Require explicit user confirmation

### 5. Command Construction Safety

**Safe Pattern** (Always Used)
```rust
// ✓ SAFE: Arguments passed separately
tokio::process::Command::new("nix")
    .arg("search")
    .arg("nixpkgs")
    .arg(&query)  // User input as separate argument
    .output()
```

**Unsafe Pattern** (Never Used)
```rust
// ✗ DANGEROUS: Shell injection risk
Command::new("sh")
    .arg("-c")
    .arg(format!("nix search nixpkgs {}", query))  // DON'T DO THIS
```

## Security Best Practices

### For Deployment

1. **Run with Minimal Privileges**
   - Do not run as root
   - Use dedicated user account
   - Apply OS-level restrictions (AppArmor, SELinux)

2. **Monitor Audit Logs**
   - Review logs for suspicious patterns
   - Alert on repeated validation failures
   - Track dangerous operation attempts

3. **Configure Timeouts**
   - Adjust based on your hardware and network
   - Set via environment variables (future enhancement)

4. **Network Security** (if using HTTP transport)
   - Bind to localhost only (`127.0.0.1`) unless remote access needed
   - Use HTTPS in production
   - Implement OAuth 2.1 authentication
   - Validate `Origin` header to prevent DNS rebinding

5. **Keep Updated**
   - Monitor security advisories
   - Update dependencies regularly
   - Run `cargo audit` periodically

### For Users

1. **Verify Tool Descriptions**
   - Read tool descriptions before use
   - Check safety annotations (read-only vs. destructive)
   - Approve destructive operations carefully

2. **Review Generated Commands**
   - For `run_in_shell`, review command before execution
   - Be cautious with untrusted input

3. **Report Security Issues**
   - See "Vulnerability Disclosure" below

## Known Limitations

1. **Trust in Nix**
   - We rely on Nix's sandbox for build isolation
   - Nix expression evaluation can be computationally expensive
   - `nix-shell` commands run with user privileges

2. **stdio Transport**
   - No authentication (relies on process isolation)
   - Suitable for local use only
   - User running server has full access

3. **Validation Coverage**
   - Not all tools have input validation yet (work in progress)
   - Some complex inputs may bypass validation

4. **Rate Limiting**
   - Not yet implemented (planned)
   - Can be DoS'd by rapid requests

## Vulnerability Disclosure

If you discover a security vulnerability in Onix-MCP:

1. **Do NOT** open a public GitHub issue
2. Email security concerns to: [your-email]
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will respond within 48 hours and work with you to address the issue.

## Security Roadmap

### Phase 1: Foundation (Current)
- ✅ Input validation framework
- ✅ Audit logging infrastructure
- ✅ Timeout enforcement
- ✅ Tool safety annotations
- 🔄 Validate all tool inputs (in progress)

### Phase 2: Enhancement (Next)
- ⏳ Rate limiting implementation
- ⏳ User confirmation for destructive operations
- ⏳ Comprehensive unit tests
- ⏳ Security integration tests

### Phase 3: Production (Future)
- ⏳ OAuth 2.1 for HTTP transport
- ⏳ Secrets management (sops-nix)
- ⏳ Command sandboxing (landlock)
- ⏳ Security metrics and monitoring
- ⏳ Automated security scanning (cargo audit in CI)

## Security Contacts

- **Lead**: [Your Name]
- **Repository**: https://github.com/[your-repo]/onix-mcp
- **Security Email**: [security@your-domain]

## Compliance

- **MCP Specification**: Compliant with MCP 2025-06-18
- **OAuth 2.1**: Supported via rmcp SDK (HTTP transport)
- **OWASP Top 10**: Addressed command injection, sensitive data exposure
- **CWE Coverage**: CWE-78 (OS Command Injection), CWE-400 (Resource Exhaustion)

## License

This security policy is part of the Onix-MCP project and is provided under the same license as the project.

---

Last Updated: 2025-12-02
Version: 1.0.0
