use std::process::Command;

pub(crate) const MIN_LIMINE_VERSION_TEXT: &str = "12.3.2";

const MIN_LIMINE_VERSION: ToolVersion = ToolVersion {
    major: 12,
    minor: 3,
    patch: 2,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct HostToolVersions {
    pub(crate) rustc: String,
    pub(crate) cargo: String,
    pub(crate) limine: String,
    pub(crate) xorriso: String,
    pub(crate) qemu: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ToolVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

impl ToolVersion {
    const fn display(self) -> ToolVersionDisplay {
        ToolVersionDisplay(self)
    }
}

struct ToolVersionDisplay(ToolVersion);

impl std::fmt::Display for ToolVersionDisplay {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}.{}.{}",
            self.0.major, self.0.minor, self.0.patch
        )
    }
}

pub(crate) fn validate_host_tools() -> Result<HostToolVersions, String> {
    let rustc = capture_version("rustc")?;
    let cargo = capture_version("cargo")?;
    let limine = capture_version("limine")?;
    let xorriso = capture_version("xorriso")?;
    let qemu = capture_version("qemu-system-x86_64")?;

    let limine_version = parse_limine_version(&limine)?;
    if limine_version < MIN_LIMINE_VERSION {
        return Err(format!(
            "Limine version {} is below required minimum {}",
            limine_version.display(),
            MIN_LIMINE_VERSION.display()
        ));
    }

    Ok(HostToolVersions {
        rustc,
        cargo,
        limine,
        xorriso,
        qemu,
    })
}

fn capture_version(tool: &str) -> Result<String, String> {
    let output = Command::new(tool)
        .arg("--version")
        .output()
        .map_err(|error| format!("required host tool unavailable: {tool}: {error}"))?;
    if !output.status.success() {
        return Err(format!("required host tool failed version check: {tool}"));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("{tool} --version stdout was not valid UTF-8: {error}"))?;
    first_non_empty_line(&stdout)
        .map(str::to_owned)
        .ok_or_else(|| format!("{tool} --version produced no version line"))
}

fn parse_limine_version(output: &str) -> Result<ToolVersion, String> {
    for word in output.split_whitespace() {
        if let Some(version) = parse_semver_triplet(word) {
            return Ok(version);
        }
    }

    Err(format!("failed to parse Limine version from: {output}"))
}

fn parse_semver_triplet(word: &str) -> Option<ToolVersion> {
    let trimmed = word.trim_matches(|character: char| !character.is_ascii_digit());
    let mut parts = trimmed.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }

    Some(ToolVersion {
        major,
        minor,
        patch,
    })
}

fn first_non_empty_line(contents: &str) -> Option<&str> {
    contents
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{MIN_LIMINE_VERSION, ToolVersion, parse_limine_version};

    #[test]
    fn limine_version_parser_accepts_version_banner() -> Result<(), String> {
        assert_eq!(
            parse_limine_version("Limine 12.3.2")?,
            ToolVersion {
                major: 12,
                minor: 3,
                patch: 2,
            }
        );
        Ok(())
    }

    #[test]
    fn limine_version_gate_tracks_current_minimum() -> Result<(), String> {
        assert!(
            parse_limine_version("Limine 12.3.2")? >= MIN_LIMINE_VERSION,
            "the pinned CI Limine version must satisfy the xtask minimum"
        );
        assert!(
            parse_limine_version("Limine 12.3.1")? < MIN_LIMINE_VERSION,
            "older Limine patch releases must remain below the gate"
        );
        Ok(())
    }
}
