//! Read, query, edit, and write CPAN `02packages.details.txt(.gz)` files —
//! the package index PAUSE publishes describing every Perl module known to
//! CPAN, its version, and the distribution that provides it.
//!
//! ```no_run
//! use cpan_packagedetails::PackageDetails;
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut pd = PackageDetails::load_file("02packages.details.txt.gz")?;
//!
//! if let Some(entry) = pd.get("Some::Module") {
//!     println!("{} is at {}", entry.package(), entry.path());
//! }
//!
//! for entry in pd.search_like("Foo::%")? {
//!     println!("{}", entry.package());
//! }
//!
//! pd.save_file("02packages.details.txt.gz")?;
//! # Ok(())
//! # }
//! ```

mod entry;
mod header;
mod like;

pub use entry::Entry;
pub use header::Header;

use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use std::io::{Read, Write};
use std::path::Path;

/// An in-memory representation of a CPAN `02packages.details.txt` file: its
/// header fields plus the list of package/version/path entries.
#[derive(Debug, Clone, Default)]
pub struct PackageDetails {
    header: Header,
    entries: IndexMap<String, Entry>,
}

const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

impl PackageDetails {
    /// An empty package details file with the standard PAUSE header fields
    /// and no entries.
    pub fn new() -> Self {
        PackageDetails {
            header: Header::standard(),
            entries: IndexMap::new(),
        }
    }

    // ---------------------------------------------------------------
    // Loading
    // ---------------------------------------------------------------

    /// Load a package details file from disk. The file is transparently
    /// gzip-decompressed if it looks gzip-compressed (regardless of its
    /// extension), otherwise it's read as plain text.
    pub fn load_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = std::fs::read(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        Self::load_bytes(&bytes)
    }

    /// Load a package details file from an in-memory byte buffer, e.g. one
    /// downloaded over the network. Gzip-compressed input is detected via
    /// its magic number and decompressed automatically.
    pub fn load_bytes(data: &[u8]) -> Result<Self> {
        let text = if data.starts_with(&GZIP_MAGIC) {
            let mut decoder = flate2::read::GzDecoder::new(data);
            let mut s = String::new();
            decoder
                .read_to_string(&mut s)
                .context("failed to gunzip package details data")?;
            s
        } else {
            String::from_utf8(data.to_vec()).context("input is not valid UTF-8")?
        };
        Self::parse_str(&text)
    }

    /// Parse a package details file already decoded as plain text.
    pub fn parse_str(text: &str) -> Result<Self> {
        let mut header = Header::new();
        let mut lines = text.lines().enumerate();
        let mut in_header = true;

        let mut entries = IndexMap::new();

        for (idx, line) in &mut lines {
            let line_no = idx + 1;
            if in_header {
                if line.trim().is_empty() {
                    in_header = false;
                    continue;
                }
                let (key, value) = line
                    .split_once(':')
                    .with_context(|| format!("invalid header line {line_no}: {line:?}"))?;
                header.set(key.trim(), value.trim());
            } else {
                if line.trim().is_empty() {
                    continue;
                }
                let entry = Entry::parse_line(line, line_no)?;
                let package = entry.package().to_string();
                if entries.insert(package.clone(), entry).is_some() {
                    bail!("duplicate package name: {package}");
                }
            }
        }

        Ok(PackageDetails { header, entries })
    }

    // ---------------------------------------------------------------
    // Saving
    // ---------------------------------------------------------------

    /// Serialize to the plain-text file format (no compression).
    pub fn to_plain_string(&self) -> String {
        let mut out = String::new();

        let key_width = self
            .header
            .iter()
            .map(|(k, _)| k.len() + 1) // +1 for the trailing colon
            .max()
            .unwrap_or(0);
        for (key, value) in self.header.iter() {
            let key_colon = format!("{key}:");
            out.push_str(&format!("{key_colon:<key_width$} {value}\n"));
        }
        out.push('\n');

        let name_width = self.entries.values().map(|e| e.package().len()).max().unwrap_or(0);
        let version_width = self
            .entries
            .values()
            .map(|e| e.version_field().len())
            .max()
            .unwrap_or(0);
        for entry in self.entries.values() {
            out.push_str(&format!(
                "{:<name_width$} {:<version_width$} {}\n",
                entry.package(),
                entry.version_field(),
                entry.path(),
            ));
        }

        out
    }

    /// Serialize to the plain-text file format as raw bytes.
    pub fn to_plain_bytes(&self) -> Vec<u8> {
        self.to_plain_string().into_bytes()
    }

    /// Serialize and gzip-compress, matching the usual `.gz` distribution
    /// format of `02packages.details.txt.gz`.
    pub fn to_gz_bytes(&self) -> Result<Vec<u8>> {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&self.to_plain_bytes())?;
        Ok(encoder.finish()?)
    }

    /// Save to disk, gzip-compressed if `path` ends in `.gz` (case-insensitive),
    /// plain text otherwise.
    pub fn save_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let is_gz = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("gz"));
        if is_gz {
            self.save_file_gz(path)
        } else {
            self.save_file_plain(path)
        }
    }

    /// Save to disk as a gzip-compressed file, regardless of extension.
    pub fn save_file_gz(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        std::fs::write(path, self.to_gz_bytes()?)
            .with_context(|| format!("failed to write {}", path.display()))
    }

    /// Save to disk as plain text, regardless of extension.
    pub fn save_file_plain(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        std::fs::write(path, self.to_plain_bytes())
            .with_context(|| format!("failed to write {}", path.display()))
    }

    // ---------------------------------------------------------------
    // Header
    // ---------------------------------------------------------------

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut Header {
        &mut self.header
    }

    // ---------------------------------------------------------------
    // Entry listing
    // ---------------------------------------------------------------

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// All entries, in file order.
    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.values()
    }

    /// All package names, in file order.
    pub fn packages(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(|s| s.as_str())
    }

    // ---------------------------------------------------------------
    // Lookup / search
    // ---------------------------------------------------------------

    /// Look up an entry by its exact package name.
    pub fn get(&self, package: &str) -> Option<&Entry> {
        self.entries.get(package)
    }

    /// Whether an entry for this exact package name exists.
    pub fn contains(&self, package: &str) -> bool {
        self.entries.contains_key(package)
    }

    /// Find all entries whose package name contains `needle`
    /// (case-insensitive substring search).
    pub fn search_substring(&self, needle: &str) -> Vec<&Entry> {
        let needle = needle.to_ascii_lowercase();
        self.entries
            .values()
            .filter(|e| e.package().to_ascii_lowercase().contains(&needle))
            .collect()
    }

    /// Find all entries whose package name matches a SQL `LIKE` pattern
    /// (`%` = any sequence of characters, `_` = any single character, `\`
    /// escapes a literal `%`, `_` or `\`). Matching is case-insensitive.
    pub fn search_like(&self, pattern: &str) -> Result<Vec<&Entry>> {
        self.entries
            .values()
            .filter_map(|e| match like::like_match(e.package(), pattern) {
                Ok(true) => Some(Ok(e)),
                Ok(false) => None,
                Err(err) => Some(Err(err)),
            })
            .collect()
    }

    // ---------------------------------------------------------------
    // Mutation
    // ---------------------------------------------------------------

    /// Add a new entry. Returns an error if an entry for this package name
    /// already exists; the existing entry is left untouched.
    pub fn add_entry(&mut self, entry: Entry) -> Result<()> {
        if self.entries.contains_key(entry.package()) {
            bail!("duplicate package name: {}", entry.package());
        }
        self.entries.insert(entry.package().to_string(), entry);
        self.header.set_line_count(self.entries.len());
        Ok(())
    }

    /// Remove the entry for `package`, if any, returning it.
    pub fn remove_entry(&mut self, package: &str) -> Option<Entry> {
        let removed = self.entries.shift_remove(package);
        if removed.is_some() {
            self.header.set_line_count(self.entries.len());
        }
        removed
    }

    /// Reorder entries alphabetically by package name (byte/ASCII order,
    /// matching PAUSE's own ordering).
    pub fn sort_by_package(&mut self) {
        self.entries.sort_unstable_keys();
    }
}
