//! Kebab-case Lucide names to Rust identifiers.

/// `a-arrow-down` -> `AArrowDown`, `building-2` -> `Building2`, `grid-2x2` -> `Grid2x2`.
///
/// A name that would start with a digit gets a leading underscore, because a
/// Rust identifier cannot.
pub fn pascal(name: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::pascal;

    #[test]
    fn kebab_to_pascal() {
        for (kebab, expected) in [
            ("a-arrow-down", "AArrowDown"),
            ("accessibility", "Accessibility"),
            ("building-2", "Building2"),
            ("trash-2", "Trash2"),
            ("house", "House"),
            ("food-beverage", "FoodBeverage"),
            ("grid-2x2", "Grid2x2"),
            ("arrow-down-0-1", "ArrowDown01"),
            ("axis-3d", "Axis3d"),
            ("2fa", "_2fa"),
        ] {
            assert_eq!(pascal(kebab), expected, "pascal({kebab})");
        }
    }
}
