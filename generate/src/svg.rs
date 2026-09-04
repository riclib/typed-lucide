//! Lucide's SVG in, the icon body out.
//!
//! The body is the outer `<svg>` element's children, in order, with every
//! attribute in its source order and every value byte for byte. Only the
//! whitespace *between* elements goes: Lucide pretty-prints its files, and the
//! body is markup to paste, not a file to read.

use anyhow::{Result, bail, ensure};

/// What one icon's SVG file yields: the outer element's attributes (minus
/// `class`) and the serialized children.
pub struct Parsed {
    pub attrs: String,
    pub body: String,
}

/// The shape of an element tree: names and attributes, in order. Two trees with
/// the same shape carry the same drawing.
#[derive(PartialEq, Eq)]
struct Shape {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<Shape>,
}

pub fn parse(source: &str) -> Result<Parsed> {
    let doc = roxmltree::Document::parse(source)?;
    let root = doc.root_element();
    ensure!(
        root.tag_name().name() == "svg",
        "root element is <{}>, not <svg>",
        root.tag_name().name()
    );

    // roxmltree hands namespace declarations back separately, and Lucide writes
    // xmlns first, so they go first here too.
    let namespaces = root.namespaces().map(|ns| match ns.name() {
        Some(prefix) => format!("xmlns:{prefix}=\"{}\"", escape(ns.uri())),
        None => format!("xmlns=\"{}\"", escape(ns.uri())),
    });
    let attrs = namespaces
        .chain(
            root.attributes()
                .filter(|a| a.name() != "class")
                .map(|a| format!("{}=\"{}\"", a.name(), escape(a.value()))),
        )
        .collect::<Vec<_>>()
        .join(" ");

    let mut body = String::new();
    write_children(root, &mut body)?;

    ensure!(!body.contains("<svg"), "body still holds an <svg> element");
    ensure!(!body.is_empty(), "icon has an empty body");

    // Verify by parsing, not by looking: the body wrapped back in an <svg> must
    // have exactly the element tree the source had.
    let wrapped = format!("<svg>{body}</svg>");
    let round = roxmltree::Document::parse(&wrapped)?;
    ensure!(
        shape(round.root_element()) == shape(root),
        "the body does not parse back to the source's elements"
    );

    Ok(Parsed { attrs, body })
}

fn write_children(node: roxmltree::Node<'_, '_>, out: &mut String) -> Result<()> {
    for child in node.children() {
        if child.is_element() {
            out.push('<');
            out.push_str(child.tag_name().name());
            for a in child.attributes() {
                out.push(' ');
                out.push_str(a.name());
                out.push_str("=\"");
                out.push_str(&escape(a.value()));
                out.push('"');
            }
            if child.has_children() {
                out.push('>');
                write_children(child, out)?;
                out.push_str("</");
                out.push_str(child.tag_name().name());
                out.push('>');
            } else {
                out.push_str("/>");
            }
        } else if child.is_text() {
            let text = child.text().unwrap_or_default();
            if !text.trim().is_empty() {
                out.push_str(&escape_text(text));
            }
        } else if child.is_comment() {
            continue;
        } else {
            bail!("unexpected node in the icon: {child:?}");
        }
    }
    Ok(())
}

fn shape(node: roxmltree::Node<'_, '_>) -> Vec<Shape> {
    node.children()
        .filter(|c| c.is_element())
        .map(|c| Shape {
            name: c.tag_name().name().to_string(),
            attrs: c
                .attributes()
                .map(|a| (a.name().to_string(), a.value().to_string()))
                .collect(),
            children: shape(c),
        })
        .collect()
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_text(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;")
}

#[cfg(test)]
mod tests {
    use super::parse;

    const HOUSE: &str = r#"<svg
  xmlns="http://www.w3.org/2000/svg"
  width="24"
  height="24"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
>
  <path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8" />
  <circle cx="12" cy="12" r="10" />
</svg>
"#;

    #[test]
    fn strips_the_outer_element_and_the_whitespace_between_children() {
        let parsed = parse(HOUSE).unwrap();
        assert_eq!(
            parsed.body,
            r#"<path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><circle cx="12" cy="12" r="10"/>"#
        );
        assert_eq!(
            parsed.attrs,
            r#"xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round""#
        );
        assert!(!parsed.attrs.contains("class"));
    }

    #[test]
    fn drops_the_lucide_classes_from_the_outer_element() {
        let with_class = HOUSE.replace("<svg\n", "<svg class=\"lucide lucide-house\"\n");
        assert_eq!(
            parse(&with_class).unwrap().attrs,
            parse(HOUSE).unwrap().attrs
        );
    }
}
