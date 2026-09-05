use anyhow::{bail, Result};

/// A single entry (row) in the package details file: a Perl package/module
/// name, its version (if known), and the path to the distribution that
/// provides it, relative to `$CPAN/authors/id/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    package: String,
    version: Option<String>,
    path: String,
}

impl Entry {
    /// Create a new entry. `version` of `None` is written out as the literal
    /// `undef`, matching PAUSE's convention for packages with no declared
    /// `$VERSION`.
    pub fn new(package: impl Into<String>, version: Option<String>, path: impl Into<String>) -> Self {
        Entry {
            package: package.into(),
            version,
            path: path.into(),
        }
    }

    /// The Perl package/module name, e.g. `Some::Module`.
    pub fn package(&self) -> &str {
        &self.package
    }

    /// The declared version, if any.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// The path to the distribution, relative to `$CPAN/authors/id/`,
    /// e.g. `A/AB/ABCD/Some-Dist-1.23.tar.gz`.
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn set_version(&mut self, version: Option<String>) {
        self.version = version;
    }

    pub fn set_path(&mut self, path: impl Into<String>) {
        self.path = path.into();
    }

    /// The version as it would be written to the file (`undef` when unknown).
    pub(crate) fn version_field(&self) -> &str {
        self.version.as_deref().unwrap_or("undef")
    }

    /// Parse a single non-blank line from the entries section of the file.
    ///
    /// Lines are whitespace-separated `package version path` triples. The
    /// version field is rarely, if ever, made up of multiple tokens, but any
    /// tokens between the first (package) and last (path) are joined back
    /// together with a single space just in case.
    pub(crate) fn parse_line(line: &str, line_no: usize) -> Result<Self> {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let (package, version, path) = match tokens.len() {
            0 | 1 => {
                bail!("invalid entry on line {line_no}: {line:?}");
            }
            2 => (tokens[0], None, tokens[1]),
            _ => {
                let version = tokens[1..tokens.len() - 1].join(" ");
                (tokens[0], Some(version), tokens[tokens.len() - 1])
            }
        };
        let version = match version {
            Some(v) if v == "undef" => None,
            other => other,
        };
        Ok(Entry::new(package.to_string(), version, path.to_string()))
    }
}
