use std::borrow::Cow;

use brigadier::StringReader;

#[test]
fn read_string_unquoted() {
    const TEXT: &str = r#"foo-0123456789._ bar baz"#;
    let mut reader = StringReader::new(TEXT);
    assert_eq!(reader.read_string(), Ok(Cow::Borrowed("foo-0123456789._")));
    assert_eq!(reader.remaining(), " bar baz");
}

#[test]
fn read_string_quoted() {
    const TEXT: &str = r#""foo"abc"#;
    let mut reader = StringReader::new(TEXT);
    assert_eq!(reader.read_string(), Ok(Cow::Borrowed("foo")));
    assert_eq!(reader.remaining(), "abc");
}

#[test]
fn read_string_quoted_unicode() {
    const TEXT: &str = "'Check: \u{2705}.'abc";
    let mut reader = StringReader::new(TEXT);
    assert_eq!(reader.read_string(), Ok(Cow::Borrowed("Check: \u{2705}.")));
    assert_eq!(reader.remaining(), "abc");
}

#[test]
fn read_string_quoted_escaped() {
    const TEXT: &str = r#""this is a\" test"abc"#;
    let mut reader = StringReader::new(TEXT);
    assert_eq!(
        reader.read_string(),
        Ok(Cow::Owned(String::from(r#"this is a" test"#)))
    );
    assert_eq!(reader.remaining(), "abc");
}
