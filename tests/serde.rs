//! With the `serde` feature an icon is its name, and only a name Lucide has.

#![cfg(feature = "serde")]

use typed_lucide::Icon;

#[test]
fn an_icon_serializes_as_its_name() {
    assert_eq!(serde_json::to_string(&Icon::House).unwrap(), r#""house""#);
    assert_eq!(
        serde_json::to_string(&vec![Icon::AArrowDown, Icon::Grid2x2]).unwrap(),
        r#"["a-arrow-down","grid-2x2"]"#
    );
}

#[test]
fn a_name_deserializes_back_to_the_same_icon() {
    for &icon in Icon::ALL {
        let json = serde_json::to_string(&icon).unwrap();
        assert_eq!(serde_json::from_str::<Icon>(&json).unwrap(), icon);
    }
}

#[test]
fn an_unknown_name_is_refused() {
    let error = serde_json::from_str::<Icon>(r#""arrow-rite""#).unwrap_err();
    assert!(error.to_string().contains("arrow-rite"), "{error}");
    assert!(
        serde_json::from_str::<Icon>(r#""House""#).is_err(),
        "names are kebab-case"
    );
    assert!(serde_json::from_str::<Icon>("42").is_err());
}
