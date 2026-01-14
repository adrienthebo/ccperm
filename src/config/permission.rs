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
    GitHub,
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
            PermissionCategory::GitHub => write!(f, "GitHub"),
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
    pub tool: String,
    pub argument: String,
    pub has_wildcard: bool,
    pub raw: String,
    pub category: PermissionCategory,
}

impl Permission {
    pub fn parse(s: &str) -> Self {
        let raw = s.to_string();
        let (tool, argument, has_wildcard) = Self::parse_permission_string(s);
        let category = Self::categorize(&tool, &argument);

        Permission {
            tool,
            argument,
            has_wildcard,
            raw,
            category,
        }
    }

    fn parse_permission_string(s: &str) -> (String, String, bool) {
        // Format: "Tool(argument)" or "Tool(argument:*)"
        if let Some(paren_start) = s.find('(') {
            if let Some(paren_end) = s.rfind(')') {
                let tool = s[..paren_start].to_string();
                let inner = &s[paren_start + 1..paren_end];
                let has_wildcard = inner.ends_with(":*");
                let argument = if has_wildcard {
                    inner[..inner.len() - 2].to_string()
                } else {
                    inner.to_string()
                };
                return (tool, argument, has_wildcard);
            }
        }
        // Fallback: treat the whole string as a simple permission
        (String::new(), s.to_string(), false)
    }

    fn categorize(tool: &str, argument: &str) -> PermissionCategory {
        let arg_lower = argument.to_lowercase();

        match tool {
            "WebFetch" => PermissionCategory::Web,
            "Bash" | "" => {
                if arg_lower.starts_with("git ") || arg_lower == "git" {
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

    pub fn display_short(&self) -> String {
        let chars: Vec<char> = self.raw.chars().collect();
        if chars.len() > 50 {
            let truncated: String = chars[..47].iter().collect();
            format!("{}...", truncated)
        } else {
            self.raw.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bash_permission() {
        let p = Permission::parse("Bash(npm install:*)");
        assert_eq!(p.tool, "Bash");
        assert_eq!(p.argument, "npm install");
        assert!(p.has_wildcard);
        assert_eq!(p.category, PermissionCategory::Npm);
    }

    #[test]
    fn test_parse_webfetch_permission() {
        let p = Permission::parse("WebFetch(domain:github.com)");
        assert_eq!(p.tool, "WebFetch");
        assert_eq!(p.argument, "domain:github.com");
        assert!(!p.has_wildcard);
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
}
