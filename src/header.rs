/// The header/preamble fields of a package details file, e.g.:
///
/// ```text
/// File:         02packages.details.txt
/// URL:          https://www.perl.com/CPAN/modules/02packages.details.txt
/// Description:  Package names found in directory $CPAN/authors/id/
/// Columns:      package name, version, path
/// Intended-For: Automated fetch routines, namespace documentation.
/// Line-Count:   123456
/// Last-Updated: Tue, 02 Sep 2025 12:00:00 GMT
/// ```
///
/// Field order is preserved as read (or as inserted), and lookups by key are
/// case-insensitive while the original casing is preserved on output.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Header {
    fields: Vec<(String, String)>,
}

impl Header {
    /// An empty header with no fields set.
    pub fn new() -> Self {
        Header::default()
    }

    /// A header populated with the standard PAUSE fields for a fresh,
    /// empty package details file.
    pub fn standard() -> Self {
        let mut header = Header::new();
        header.set("File", "02packages.details.txt");
        header.set(
            "Description",
            "Package names found in directory $CPAN/authors/id/",
        );
        header.set("Columns", "package name, version, path");
        header.set(
            "Intended-For",
            "Automated fetch routines, namespace documentation.",
        );
        header.set_line_count(0);
        header
    }

    fn index_of(&self, key: &str) -> Option<usize> {
        self.fields
            .iter()
            .position(|(k, _)| k.eq_ignore_ascii_case(key))
    }

    /// Look up a header field by name (case-insensitive).
    pub fn get(&self, key: &str) -> Option<&str> {
        self.index_of(key).map(|i| self.fields[i].1.as_str())
    }

    /// Set a header field, preserving its original position if it already
    /// exists (case-insensitively), or appending it as a new field.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        match self.index_of(&key) {
            Some(i) => self.fields[i].1 = value,
            None => self.fields.push((key, value)),
        }
    }

    /// Remove a header field, returning its value if it was present.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.index_of(key).map(|i| self.fields.remove(i).1)
    }

    /// Iterate over all header fields in file order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.fields.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Number of header fields.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn file(&self) -> Option<&str> {
        self.get("File")
    }

    pub fn set_file(&mut self, value: impl Into<String>) {
        self.set("File", value);
    }

    pub fn url(&self) -> Option<&str> {
        self.get("URL")
    }

    pub fn set_url(&mut self, value: impl Into<String>) {
        self.set("URL", value);
    }

    pub fn description(&self) -> Option<&str> {
        self.get("Description")
    }

    pub fn set_description(&mut self, value: impl Into<String>) {
        self.set("Description", value);
    }

    pub fn columns(&self) -> Option<&str> {
        self.get("Columns")
    }

    pub fn set_columns(&mut self, value: impl Into<String>) {
        self.set("Columns", value);
    }

    pub fn intended_for(&self) -> Option<&str> {
        self.get("Intended-For")
    }

    pub fn set_intended_for(&mut self, value: impl Into<String>) {
        self.set("Intended-For", value);
    }

    pub fn written_by(&self) -> Option<&str> {
        self.get("Written-By")
    }

    pub fn set_written_by(&mut self, value: impl Into<String>) {
        self.set("Written-By", value);
    }

    /// The `Line-Count` header field, parsed as a number.
    pub fn line_count(&self) -> Option<usize> {
        self.get("Line-Count").and_then(|v| v.trim().parse().ok())
    }

    pub fn set_line_count(&mut self, value: usize) {
        self.set("Line-Count", value.to_string());
    }

    pub fn last_updated(&self) -> Option<&str> {
        self.get("Last-Updated")
    }

    pub fn set_last_updated(&mut self, value: impl Into<String>) {
        self.set("Last-Updated", value);
    }
}
