use cpan_packagedetails::{Entry, PackageDetails};

const SAMPLE: &str = "\
File:         02packages.details.txt
URL:          https://www.perl.com/CPAN/modules/02packages.details.txt
Description:  Package names found in directory $CPAN/authors/id/
Columns:      package name, version, path
Intended-For: Automated fetch routines, namespace documentation.
Line-Count:   3
Last-Updated: Tue, 02 Sep 2025 12:00:00 GMT

Foo::Bar      1.23  A/AB/ABCD/Foo-Bar-1.23.tar.gz
Foo::Baz      undef A/AB/ABCD/Foo-Bar-1.23.tar.gz
Quux::Thing   0.01  Q/QU/QUUX/Quux-Thing-0.01.tar.gz
";

#[test]
fn parses_header_fields() {
    let pd = PackageDetails::parse_str(SAMPLE).unwrap();
    assert_eq!(pd.header().file(), Some("02packages.details.txt"));
    assert_eq!(
        pd.header().url(),
        Some("https://www.perl.com/CPAN/modules/02packages.details.txt")
    );
    assert_eq!(pd.header().line_count(), Some(3));
    assert_eq!(
        pd.header().last_updated(),
        Some("Tue, 02 Sep 2025 12:00:00 GMT")
    );
}

#[test]
fn parses_entries() {
    let pd = PackageDetails::parse_str(SAMPLE).unwrap();
    assert_eq!(pd.len(), 3);

    let foo_bar = pd.get("Foo::Bar").unwrap();
    assert_eq!(foo_bar.version(), Some("1.23"));
    assert_eq!(foo_bar.path(), "A/AB/ABCD/Foo-Bar-1.23.tar.gz");

    let foo_baz = pd.get("Foo::Baz").unwrap();
    assert_eq!(foo_baz.version(), None, "undef should parse to None");

    assert!(pd.get("Does::NotExist").is_none());
}

#[test]
fn duplicate_package_in_file_is_an_error() {
    let dup = "Columns: package name, version, path\n\nFoo::Bar 1 A/foo.tar.gz\nFoo::Bar 2 A/foo2.tar.gz\n";
    let err = PackageDetails::parse_str(dup).unwrap_err();
    assert!(err.to_string().contains("duplicate package name: Foo::Bar"));
}

#[test]
fn add_entry_rejects_duplicates() {
    let mut pd = PackageDetails::new();
    pd.add_entry(Entry::new("Foo::Bar", Some("1.0".to_string()), "A/AB/Foo-Bar-1.0.tar.gz"))
        .unwrap();

    let err = pd
        .add_entry(Entry::new("Foo::Bar", Some("2.0".to_string()), "A/AB/Foo-Bar-2.0.tar.gz"))
        .unwrap_err();
    assert!(err.to_string().contains("duplicate package name: Foo::Bar"));

    // Original entry must be untouched.
    assert_eq!(pd.get("Foo::Bar").unwrap().version(), Some("1.0"));
    assert_eq!(pd.len(), 1);
}

#[test]
fn remove_entry() {
    let mut pd = PackageDetails::parse_str(SAMPLE).unwrap();
    let removed = pd.remove_entry("Foo::Bar").unwrap();
    assert_eq!(removed.package(), "Foo::Bar");
    assert!(pd.get("Foo::Bar").is_none());
    assert_eq!(pd.len(), 2);
    assert_eq!(pd.header().line_count(), Some(2));

    assert!(pd.remove_entry("Nonexistent::Module").is_none());
}

#[test]
fn search_substring_is_case_insensitive() {
    let pd = PackageDetails::parse_str(SAMPLE).unwrap();
    let mut names: Vec<&str> = pd.search_substring("foo").iter().map(|e| e.package()).collect();
    names.sort_unstable();
    assert_eq!(names, vec!["Foo::Bar", "Foo::Baz"]);

    assert!(pd.search_substring("zzz").is_empty());
}

#[test]
fn search_like_wildcards() {
    let pd = PackageDetails::parse_str(SAMPLE).unwrap();

    let mut names: Vec<&str> = pd
        .search_like("Foo::%")
        .unwrap()
        .iter()
        .map(|e| e.package())
        .collect();
    names.sort_unstable();
    assert_eq!(names, vec!["Foo::Bar", "Foo::Baz"]);

    let single = pd.search_like("Quux::Th_ng").unwrap();
    assert_eq!(single.len(), 1);
    assert_eq!(single[0].package(), "Quux::Thing");

    assert!(pd.search_like("Nothing::Like::%").unwrap().is_empty());
}

#[test]
fn round_trip_plain_text() {
    let pd = PackageDetails::parse_str(SAMPLE).unwrap();
    let text = pd.to_plain_string();
    let reparsed = PackageDetails::parse_str(&text).unwrap();

    assert_eq!(reparsed.len(), pd.len());
    for entry in pd.entries() {
        let other = reparsed.get(entry.package()).unwrap();
        assert_eq!(other.version(), entry.version());
        assert_eq!(other.path(), entry.path());
    }
}

#[test]
fn round_trip_through_gzip_bytes() {
    let pd = PackageDetails::parse_str(SAMPLE).unwrap();
    let gz = pd.to_gz_bytes().unwrap();
    assert_eq!(&gz[..2], &[0x1f, 0x8b], "should be real gzip output");

    let reloaded = PackageDetails::load_bytes(&gz).unwrap();
    assert_eq!(reloaded.len(), pd.len());
    assert_eq!(reloaded.get("Quux::Thing").unwrap().path(), "Q/QU/QUUX/Quux-Thing-0.01.tar.gz");
}

#[test]
fn round_trip_through_files() {
    let dir = std::env::temp_dir().join(format!("cpan-packagedetails-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    let mut pd = PackageDetails::new();
    pd.add_entry(Entry::new("My::Module", Some("0.1".to_string()), "M/MY/ME/My-Module-0.1.tar.gz"))
        .unwrap();

    let gz_path = dir.join("02packages.details.txt.gz");
    let plain_path = dir.join("02packages.details.txt");
    pd.save_file(&gz_path).unwrap();
    pd.save_file(&plain_path).unwrap();

    let from_gz = PackageDetails::load_file(&gz_path).unwrap();
    let from_plain = PackageDetails::load_file(&plain_path).unwrap();

    assert_eq!(from_gz.len(), 1);
    assert_eq!(from_plain.len(), 1);
    assert_eq!(from_gz.get("My::Module").unwrap().version(), Some("0.1"));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn sort_by_package_orders_entries() {
    let mut pd = PackageDetails::new();
    pd.add_entry(Entry::new("Zebra::Thing", None, "z.tar.gz")).unwrap();
    pd.add_entry(Entry::new("Alpha::Thing", None, "a.tar.gz")).unwrap();
    pd.sort_by_package();

    let names: Vec<&str> = pd.packages().collect();
    assert_eq!(names, vec!["Alpha::Thing", "Zebra::Thing"]);
}
