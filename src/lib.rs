//! Every [Lucide](https://lucide.dev) icon as a Rust enum variant, with its
//! inline SVG, its tags and categories, and a scored search.
//!
//! ```
//! use typed_lucide::Icon;
//!
//! let house = Icon::House;
//!
//! assert_eq!(house.name(), "house");
//! assert!(house.svg().starts_with("<svg class=\"icon\""));
//! assert!(house.html().starts_with("<span class=\"icon\">"));
//! assert!(house.body().starts_with("<path"));
//! ```
//!
//! A name that does not exist is a compile error. There is no build step, no
//! network at build time and no runtime lookup: the generated code is
//! committed and published, one crate version per Lucide release.

#![warn(missing_docs)]

mod icon;
mod meta;

use core::fmt;
use core::str::FromStr;

pub use icon::Icon;
pub use meta::Category;

/// The class every icon carries, on the span and on the `<svg>`.
const CLASS: &str = "icon";

impl Icon {
    /// The `<svg>` alone, with `class="icon"`.
    ///
    /// ```
    /// # use typed_lucide::Icon;
    /// assert!(Icon::House.svg().ends_with("</svg>"));
    /// ```
    pub fn svg(self) -> String {
        let mut out = String::new();
        self.write_svg(&Attrs::new(), &mut out);
        out
    }

    /// The `<svg>` inside a `<span class="icon">`, which is what a page wants.
    pub fn html(self) -> String {
        self.render(&Attrs::new())
    }

    /// [`html`](Icon::html) with your own classes on the span and your own
    /// attributes on the `<svg>`.
    ///
    /// ```
    /// # use typed_lucide::{Attrs, Icon};
    /// let html = Icon::House.render(&Attrs::new().class("w-4 h-4").attr("aria-label", "Home"));
    /// assert!(html.starts_with(r#"<span class="icon w-4 h-4"><svg class="icon" aria-label="Home""#));
    /// ```
    ///
    /// An `aria-label` replaces the `aria-hidden="true"` an icon otherwise
    /// carries.
    pub fn render(self, attrs: &Attrs) -> String {
        let mut out = String::new();
        out.push_str("<span class=\"");
        out.push_str(CLASS);
        if !attrs.class.is_empty() {
            out.push(' ');
            escape_into(&attrs.class, &mut out);
        }
        out.push_str("\">");
        self.write_svg(attrs, &mut out);
        out.push_str("</span>");
        out
    }

    /// What this icon is for, in Lucide's words.
    ///
    /// ```
    /// # use typed_lucide::Icon;
    /// assert!(Icon::House.tags().contains(&"home"));
    /// ```
    pub fn tags(self) -> &'static [&'static str] {
        meta::TAGS[self as usize]
    }

    /// The categories Lucide files this icon under.
    ///
    /// ```
    /// # use typed_lucide::{Category, Icon};
    /// assert!(Icon::House.categories().contains(&Category::Buildings));
    /// ```
    pub fn categories(self) -> &'static [Category] {
        meta::CATEGORIES[self as usize]
    }

    fn write_svg(self, attrs: &Attrs, out: &mut String) {
        out.push_str("<svg class=\"");
        out.push_str(CLASS);
        out.push('"');
        for (name, value) in &attrs.attrs {
            out.push(' ');
            out.push_str(name);
            out.push_str("=\"");
            escape_into(value, out);
            out.push('"');
        }
        out.push(' ');
        out.push_str(icon::SVG_ATTRS);
        if !attrs.has_aria_label() {
            out.push_str(" aria-hidden=\"true\"");
        }
        out.push('>');
        out.push_str(self.body());
        out.push_str("</svg>");
    }
}

/// Classes for the span and attributes for the `<svg>`.
///
/// ```
/// # use typed_lucide::{Attrs, Icon};
/// Icon::Bell.render(&Attrs::new().class("w-4 h-4").attr("id", "alarm"));
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Attrs {
    class: String,
    attrs: Vec<(String, String)>,
}

impl Attrs {
    /// No classes, no attributes: what [`Icon::html`] renders with.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append classes to the span's `class`. Call it more than once and they
    /// join with a space.
    pub fn class(mut self, class: &str) -> Self {
        if !class.is_empty() {
            if !self.class.is_empty() {
                self.class.push(' ');
            }
            self.class.push_str(class);
        }
        self
    }

    /// Put an attribute on the `<svg>`. The value is HTML-escaped; the name is
    /// yours to get right.
    pub fn attr(mut self, name: &str, value: &str) -> Self {
        self.attrs.push((name.to_string(), value.to_string()));
        self
    }

    fn has_aria_label(&self) -> bool {
        self.attrs
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("aria-label"))
    }
}

/// A name Lucide does not have.
///
/// ```
/// # use typed_lucide::{Icon, Unknown};
/// assert_eq!("arrow-rite".parse::<Icon>(), Err(Unknown("arrow-rite".to_string())));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unknown(pub String);

impl fmt::Display for Unknown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no lucide icon named {:?}", self.0)
    }
}

impl std::error::Error for Unknown {}

impl fmt::Display for Icon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Icon {
    type Err = Unknown;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Icon::from_name(name).ok_or_else(|| Unknown(name.to_string()))
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Every icon carrying this tag, sorted by name; empty if no icon does.
///
/// ```
/// # use typed_lucide::{icons_with_tag, Icon};
/// assert!(icons_with_tag("home").contains(&Icon::House));
/// ```
pub fn icons_with_tag(tag: &str) -> &'static [Icon] {
    let tag = tag.to_lowercase();
    meta::TAG_INDEX
        .binary_search_by(|(known, _)| (*known).cmp(tag.as_str()))
        .map(|i| meta::TAG_INDEX[i].1)
        .unwrap_or(&[])
}

/// Every tag Lucide uses, lowercased and sorted.
pub fn all_tags() -> impl ExactSizeIterator<Item = &'static str> {
    meta::TAG_INDEX.iter().map(|(tag, _)| *tag)
}

fn escape_into(value: &str, out: &mut String) {
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::Icon;
    use serde::de::{Error, Unexpected, Visitor};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for Icon {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(self.name())
        }
    }

    impl<'de> Deserialize<'de> for Icon {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct Name;

            impl Visitor<'_> for Name {
                type Value = Icon;

                fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_str("a lucide icon name")
                }

                fn visit_str<E: Error>(self, value: &str) -> Result<Icon, E> {
                    Icon::from_name(value)
                        .ok_or_else(|| E::invalid_value(Unexpected::Str(value), &self))
                }
            }

            deserializer.deserialize_str(Name)
        }
    }
}
