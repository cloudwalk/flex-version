use super::*;

fn parse(input: &str) -> Version {
    input.parse().unwrap()
}

#[test]
fn test_equals() {
    let expected: Version = parse("1.2");

    assert_eq!(parse("1.2.0"), expected);
    assert_eq!(parse("1.2.0.0.0.0"), expected);
    assert_ne!(parse("1.2.0.0.0.1"), expected);
    assert_ne!(parse("1.2.0.0.0.a"), expected);
}

#[test]
fn test_ordering() {
    let versions: Vec<Version> = vec![
        parse("0.9"),
        parse("1.0.a.2"),
        parse("1.0.b1"),
        parse("1.0"),
        parse("1"),
        parse("1.0.0.0"),
        parse("1.0.1"),
    ];

    let mut versions_sorted = versions.clone();
    versions_sorted.reverse();
    versions_sorted.sort();

    assert_eq!(versions, versions_sorted)
}

/// Some versions have the format "7.4.1 (4452929)", so we must be able to parse the
/// trailing parenthesized component.
#[test]
fn test_calculator_version() {
    "7.4.1 (4452929)"
        .parse::<Version>()
        .expect("invalid version");
}

/// Some versions have the format "10.0.014 (Isengard_RC01.phone_dynamic)", so we must be
/// able to parse the trailing parenthesized component.
#[test]
fn test_messaging_version() {
    "10.0.014 (Isengard_RC01.phone_dynamic)"
        .parse::<Version>()
        .expect("invalid version");
}

/// Some vendors sometimes use goofy versions, and we must be able to parse them.
#[test]
fn test_vendor_version() {
    "3.10.6.0004_tacIssue_RIDCrush_Issue_211101_182805"
        .parse::<Version>()
        .expect("invalid version");
}

#[test]
fn test_version_with_timestamp() {
    "3.10.0+20251203173456"
        .parse::<Version>()
        .expect("invalid version");
}