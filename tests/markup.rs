//! The markup, byte for byte. This is the README's Markup section.

use typed_lucide::{Attrs, Icon};

const HOUSE: &str = r#"<span class="icon"><svg class="icon" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .709-1.528l7-6a2 2 0 0 1 2.582 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg></span>"#;

#[test]
fn house_is_pinned() {
    assert_eq!(Icon::House.html(), HOUSE);
    assert_eq!(
        Icon::House.svg(),
        HOUSE
            .trim_start_matches("<span class=\"icon\">")
            .trim_end_matches("</span>")
    );
    assert_eq!(
        Icon::House.body(),
        r#"<path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .709-1.528l7-6a2 2 0 0 1 2.582 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>"#
    );
}

#[test]
fn html_is_render_with_nothing_added() {
    for &icon in Icon::ALL {
        assert_eq!(icon.html(), icon.render(&Attrs::new()));
        assert!(
            icon.html()
                .starts_with("<span class=\"icon\"><svg class=\"icon\" ")
        );
        assert!(icon.html().ends_with("</svg></span>"));
        assert!(icon.svg().contains(icon.body()));
    }
}

#[test]
fn classes_go_on_the_span_and_attributes_on_the_svg() {
    assert_eq!(
        Icon::House.render(&Attrs::new().class("w-4 h-4").attr("aria-label", "Home")),
        HOUSE
            .replace(r#"<span class="icon">"#, r#"<span class="icon w-4 h-4">"#)
            .replace(
                r#"<svg class="icon" "#,
                r#"<svg class="icon" aria-label="Home" "#
            )
            .replace(r#" aria-hidden="true""#, "")
    );
}

#[test]
fn classes_add_up_and_empty_ones_do_not() {
    let html = Icon::Bell.render(&Attrs::new().class("w-4").class("h-4").class(""));
    assert!(html.starts_with(r#"<span class="icon w-4 h-4">"#), "{html}");
    assert!(
        Icon::Bell
            .render(&Attrs::new().class(""))
            .starts_with(r#"<span class="icon">"#)
    );
}

#[test]
fn an_icon_is_hidden_from_a_screen_reader_unless_it_is_labelled() {
    assert!(Icon::Bell.html().contains(r#"aria-hidden="true""#));
    assert!(
        Icon::Bell
            .render(&Attrs::new().attr("id", "x"))
            .contains(r#"aria-hidden="true""#)
    );
    let labelled = Icon::Bell.render(&Attrs::new().attr("aria-label", "Alerts"));
    assert!(!labelled.contains("aria-hidden"));
    let shouted = Icon::Bell.render(&Attrs::new().attr("ARIA-LABEL", "Alerts"));
    assert!(
        !shouted.contains("aria-hidden"),
        "the check is case-insensitive"
    );
}

#[test]
fn attribute_values_are_escaped() {
    let html = Icon::Bell.render(
        &Attrs::new()
            .class(r#"a"b"#)
            .attr("data-note", r#"<a href="x">&"#),
    );
    assert!(html.contains(r#"<span class="icon a&quot;b">"#), "{html}");
    assert!(
        html.contains(r#"data-note="&lt;a href=&quot;x&quot;&gt;&amp;""#),
        "{html}"
    );
}

#[test]
fn attributes_keep_the_order_they_were_given() {
    let html = Icon::Bell.render(&Attrs::new().attr("id", "a").attr("data-x", "b"));
    assert!(
        html.contains(r#"<svg class="icon" id="a" data-x="b" xmlns=""#),
        "{html}"
    );
}
