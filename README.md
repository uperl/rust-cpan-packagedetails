# cpan-packagedetails

A Rust library for reading, querying, editing, and writing CPAN
`02packages.details.txt(.gz)` files — the package index [PAUSE](https://pause.perl.org/)
publishes describing every Perl module known to CPAN, its version, and the
distribution that provides it.

## Features

- Load from disk or from an in-memory byte buffer; gzip is detected
  automatically from its magic number, so `.gz` and plain-text files both
  just work.
- Access to the header fields (`File`, `URL`, `Description`, `Columns`,
  `Intended-For`, `Written-By`, `Line-Count`, `Last-Updated`, or any other
  field present) preserving their original order.
- Iterate all entries, look one up by exact package name, or search by
  substring or SQL `LIKE` pattern (`%`/`_` wildcards).
- Add or remove entries; adding a package name that's already present is an
  error.
- Save back to disk or to an in-memory buffer, plain text or gzip-compressed.

## Example

```rust
use cpan_packagedetails::PackageDetails;

fn main() -> anyhow::Result<()> {
    let mut pd = PackageDetails::load_file("02packages.details.txt.gz")?;

    if let Some(entry) = pd.get("Some::Module") {
        println!("{} is at {}", entry.package(), entry.path());
    }

    for entry in pd.search_like("Foo::%")? {
        println!("{}", entry.package());
    }

    pd.save_file("02packages.details.txt.gz")?;
    Ok(())
}
```

## License

Licensed under the [MIT license](LICENSE).
