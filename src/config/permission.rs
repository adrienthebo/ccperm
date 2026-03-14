use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PermissionCategory {
    Git,
    Npm,
    GCloud,
    FileSystem,
    Web,
    Python,
    Cargo,
    Docker,
    Go,
    GitHub,
    Mcp,
    Skill,
    SlashCommand,
    Other,
}

impl fmt::Display for PermissionCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionCategory::Git => write!(f, "Git"),
            PermissionCategory::Npm => write!(f, "NPM"),
            PermissionCategory::GCloud => write!(f, "GCloud"),
            PermissionCategory::FileSystem => write!(f, "FileSystem"),
            PermissionCategory::Web => write!(f, "Web"),
            PermissionCategory::Python => write!(f, "Python"),
            PermissionCategory::Cargo => write!(f, "Cargo"),
            PermissionCategory::Docker => write!(f, "Docker"),
            PermissionCategory::Go => write!(f, "Go"),
            PermissionCategory::GitHub => write!(f, "GitHub"),
            PermissionCategory::Mcp => write!(f, "MCP"),
            PermissionCategory::Skill => write!(f, "Skill"),
            PermissionCategory::SlashCommand => write!(f, "SlashCommand"),
            PermissionCategory::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionType {
    Allow,
    Deny,
    Ask,
}

impl fmt::Display for PermissionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionType::Allow => write!(f, "Allow"),
            PermissionType::Deny => write!(f, "Deny"),
            PermissionType::Ask => write!(f, "Ask"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Permission {
    pub raw: String,
    pub category: PermissionCategory,
}

const KNOWN_TOOLS: &[&str] = &["Bash", "Read", "Edit", "Write", "WebFetch", "Agent", "Skill", "SlashCommand"];

impl Permission {
    pub fn validate(s: &str) -> Result<(), &'static str> {
        if s.is_empty() {
            return Err("Permission rule cannot be empty");
        }

        if let Some(paren_start) = s.find('(') {
            let tool = &s[..paren_start];
            if tool.is_empty() {
                return Err("Missing tool name before '('");
            }
            if !KNOWN_TOOLS.contains(&tool) && !tool.starts_with("mcp__") {
                return Err("Unknown tool name");
            }
            if !s.ends_with(')') {
                return Err("Missing closing ')'");
            }
            let specifier = &s[paren_start + 1..s.len() - 1];
            if specifier.is_empty() {
                return Err("Specifier cannot be empty");
            }
            Ok(())
        } else {
            // Bare tool name or MCP tool
            if KNOWN_TOOLS.contains(&s) || s.starts_with("mcp__") {
                Ok(())
            } else {
                Err("Unknown tool name")
            }
        }
    }

    pub fn parse(s: &str) -> Self {
        let raw = s.to_string();
        let (tool, argument) = if let Some(paren_start) = s.find('(') {
            let tool = &s[..paren_start];
            let paren_end = s.rfind(')').unwrap_or(s.len());
            let inner = &s[paren_start + 1..paren_end];
            let argument = inner.strip_suffix(":*").unwrap_or(inner);
            (tool, argument)
        } else {
            ("", s)
        };
        let category = Self::categorize(tool, argument);

        Permission { raw, category }
    }

    fn categorize(tool: &str, argument: &str) -> PermissionCategory {
        let arg_lower = argument.to_lowercase();

        match tool {
            "WebFetch" => PermissionCategory::Web,
            "Skill" => PermissionCategory::Skill,
            "SlashCommand" => PermissionCategory::SlashCommand,
            t if t.starts_with("mcp__") => PermissionCategory::Mcp,
            "Bash" | "" => {
                if arg_lower.starts_with("mcp__") {
                    PermissionCategory::Mcp
                } else if arg_lower.starts_with("git ") || arg_lower == "git" {
                    PermissionCategory::Git
                } else if arg_lower.starts_with("npm ")
                    || arg_lower.starts_with("npx ")
                    || arg_lower.starts_with("bun ")
                    || arg_lower.starts_with("yarn ")
                    || arg_lower.starts_with("pnpm ")
                {
                    PermissionCategory::Npm
                } else if arg_lower.starts_with("gcloud ")
                    || arg_lower.starts_with("gsutil ")
                    || arg_lower.contains("googleapis.com")
                {
                    PermissionCategory::GCloud
                } else if arg_lower.starts_with("gh ") {
                    PermissionCategory::GitHub
                } else if arg_lower.starts_with("cargo ")
                    || arg_lower.starts_with("rustup ")
                    || arg_lower.starts_with("rustc ")
                {
                    PermissionCategory::Cargo
                } else if arg_lower.starts_with("python")
                    || arg_lower.starts_with("pip ")
                    || arg_lower.starts_with("pip3 ")
                {
                    PermissionCategory::Python
                } else if arg_lower.starts_with("docker ")
                    || arg_lower.starts_with("docker-compose ")
                {
                    PermissionCategory::Docker
                } else if arg_lower.starts_with("go ")
                    || arg_lower.starts_with("golangci-lint ")
                {
                    PermissionCategory::Go
                } else if arg_lower.starts_with("cat ")
                    || arg_lower.starts_with("ls ")
                    || arg_lower.starts_with("rm ")
                    || arg_lower.starts_with("mkdir ")
                    || arg_lower.starts_with("cp ")
                    || arg_lower.starts_with("mv ")
                    || arg_lower.starts_with("chmod ")
                    || arg_lower.starts_with("find ")
                    || arg_lower.starts_with("touch ")
                {
                    PermissionCategory::FileSystem
                } else {
                    PermissionCategory::Other
                }
            }
            _ => PermissionCategory::Other,
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bash_permission() {
        let p = Permission::parse("Bash(npm install:*)");
        assert_eq!(p.category, PermissionCategory::Npm);
    }

    #[test]
    fn test_parse_webfetch_permission() {
        let p = Permission::parse("WebFetch(domain:github.com)");
        assert_eq!(p.category, PermissionCategory::Web);
    }

    #[test]
    fn test_parse_git_permission() {
        let p = Permission::parse("Bash(git commit:*)");
        assert_eq!(p.category, PermissionCategory::Git);
    }

    #[test]
    fn test_parse_gcloud_permission() {
        let p = Permission::parse("Bash(gcloud run deploy:*)");
        assert_eq!(p.category, PermissionCategory::GCloud);
    }

    #[test]
    fn test_parse_skill_permission() {
        let p = Permission::parse("Skill(add-admin-page)");
        assert_eq!(p.category, PermissionCategory::Skill);
    }

    #[test]
    fn test_parse_slash_command_permission() {
        let p = Permission::parse("SlashCommand(/ci-failures)");
        assert_eq!(p.category, PermissionCategory::SlashCommand);
    }

    #[test]
    fn test_validate_valid_rules() {
        assert!(Permission::validate("Bash").is_ok());
        assert!(Permission::validate("Bash(npm install:*)").is_ok());
        assert!(Permission::validate("Bash(git *)").is_ok());
        assert!(Permission::validate("WebFetch(domain:example.com)").is_ok());
        assert!(Permission::validate("Read(/src/**)").is_ok());
        assert!(Permission::validate("Edit").is_ok());
        assert!(Permission::validate("Write").is_ok());
        assert!(Permission::validate("Agent(Explore)").is_ok());
        assert!(Permission::validate("mcp__puppeteer").is_ok());
        assert!(Permission::validate("mcp__puppeteer__navigate").is_ok());
        assert!(Permission::validate("Skill(add-admin-page)").is_ok());
        assert!(Permission::validate("SlashCommand(/ci-failures)").is_ok());
    }

    #[test]
    fn test_validate_invalid_rules() {
        assert!(Permission::validate("").is_err());
        assert!(Permission::validate("Foo").is_err());
        assert!(Permission::validate("Bash(").is_err());
        assert!(Permission::validate("Bash()").is_err());
        assert!(Permission::validate("(npm install)").is_err());
        assert!(Permission::validate("unknown(thing)").is_err());
    }
}
