//! What the generator emitted has to hold for every icon, not just for House.

use typed_lucide::{Category, Icon, Unknown};

/// The generator's rule, restated here so a regeneration that changes it fails
/// loudly: kebab to PascalCase, digits kept, a leading digit gets an underscore.
fn pascal(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for part in name.split('-') {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

#[test]
fn every_name_round_trips_through_parse() {
    for &icon in Icon::ALL {
        assert_eq!(icon.name().parse::<Icon>(), Ok(icon));
        assert_eq!(Icon::from_name(icon.name()), Some(icon));
        assert_eq!(icon.to_string(), icon.name());
    }
    assert_eq!(Icon::ALL.len(), Icon::COUNT);
}

#[test]
fn an_unknown_name_is_an_error_not_a_panic() {
    assert_eq!(
        "arrow-rite".parse::<Icon>(),
        Err(Unknown("arrow-rite".to_string()))
    );
    assert_eq!(
        Icon::from_name("House"),
        None,
        "names are kebab-case, not Pascal"
    );
    assert_eq!(Icon::from_name(""), None);
}

#[test]
fn the_kebab_name_and_the_variant_name_agree_both_ways() {
    let mut variants = std::collections::BTreeMap::new();
    for &icon in Icon::ALL {
        // kebab -> variant
        let variant = format!("{icon:?}");
        assert_eq!(
            pascal(icon.name()),
            variant,
            "{} names the wrong variant",
            icon.name()
        );
        // variant -> kebab: the variant names are unique, so the map back is a
        // table (PascalCase alone cannot tell `arrow-down-0-1` from a name
        // spelled `arrow-down-01`).
        if let Some(other) = variants.insert(variant.clone(), icon.name()) {
            panic!("{} and {} both answer to {variant}", icon.name(), other);
        }
    }
    assert_eq!(variants.len(), Icon::COUNT);
    assert_eq!(variants["House"], "house");
    assert_eq!(variants["ArrowDown01"], "arrow-down-0-1");
    assert_eq!(variants["Grid2x2"], "grid-2x2");
}

#[test]
fn the_names_are_sorted_kebab_case_and_unique() {
    let names: Vec<&str> = Icon::ALL.iter().map(|icon| icon.name()).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        names, sorted,
        "from_name binary-searches, so the names must be sorted"
    );
    for name in names {
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "{name} is not kebab-case"
        );
    }
}

#[test]
fn every_body_is_well_formed_and_holds_no_svg_element() {
    for &icon in Icon::ALL {
        let body = icon.body();
        assert!(!body.is_empty(), "{icon} has no body");
        assert!(!body.contains("<svg"), "{icon} still carries an <svg>");
        assert!(!body.contains("class="), "{icon} still carries a class");
        let wrapped = format!("<svg>{body}</svg>");
        let doc = roxmltree::Document::parse(&wrapped)
            .unwrap_or_else(|e| panic!("{icon} is not well-formed XML: {e}"));
        assert!(
            doc.root_element().children().any(|c| c.is_element()),
            "{icon} has no elements"
        );
    }
}

#[test]
fn every_icon_has_tags_and_categories_that_point_back_at_it() {
    for &icon in Icon::ALL {
        assert!(!icon.tags().is_empty(), "{icon} has no tags");
        assert!(!icon.categories().is_empty(), "{icon} has no categories");
        for category in icon.categories() {
            assert!(
                category.icons().contains(&icon),
                "{category} does not list {icon}"
            );
        }
        for tag in icon.tags() {
            assert!(
                typed_lucide::icons_with_tag(tag).contains(&icon),
                "the tag {tag} does not list {icon}"
            );
        }
    }
}

#[test]
fn categories_know_their_names_and_their_icons() {
    for &category in Category::ALL {
        assert_eq!(Category::from_name(category.name()), Some(category));
        assert!(!category.icons().is_empty(), "{category} is empty");
        for icon in category.icons() {
            assert!(icon.categories().contains(&category));
        }
    }
    assert_eq!(Category::ALL.len(), Category::COUNT);
    assert_eq!(Category::from_name("no-such-category"), None);
    assert!(Icon::House.categories().contains(&Category::Buildings));
    assert!(Category::Navigation.icons().contains(&Icon::House));
}
