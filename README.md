# typed-lucide

Every [Lucide](https://lucide.dev) icon as a Rust enum variant, with its inline
SVG, its tags and categories, a scored search, and an `icon-search` CLI.

A name that does not exist is a compile error. No build step, no network at
build time, no runtime lookup: the generated code is committed and published,
one crate version per Lucide release.

```rust
use typed_lucide::Icon;

let house = Icon::House;

house.name();   // "house"
house.svg();    // <svg class="icon" …><path d="…"/>…</svg>
house.html();   // <span class="icon"><svg …>…</svg></span>
house.body();   // <path d="…"/>… — the inner elements only, for your own <svg>
```

The sibling of [`riclib/icon`](https://github.com/riclib/icon) for Go: same
Lucide release, same kebab names, same `icon` class, so a page served by either
stack draws the same glyph and one `icon-search` answers for both.

## Install

```toml
[dependencies]
typed-lucide = "=1.40.0"
```

The crate version **is** the Lucide version. `1.40.0` is Lucide `1.40.0`.
Lucide renames and removes icons between releases, so a bump can break a build:
pin it exactly, and moving to a new Lucide is a manifest edit you make rather
than a `cargo update` you did not.

Zero dependencies. The CLI is behind a feature:

```bash
cargo install typed-lucide --features cli   # puts icon-search on PATH
```

## Use

### In a template

`Icon` implements `Display` as its kebab name. The markup comes from a method, so
a template says what it means:

```jinja
{# askama #}
<button class="ad-icon-btn" title="Settings">{{ Icon::Settings.html()|safe }}</button>
```

```rust
// maud, or any format!
html! { span { (PreEscaped(Icon::Bell.html())) } }
```

### With extra classes and attributes

```rust
Icon::House.render(&Attrs::new().class("w-4 h-4").attr("aria-label", "Home"))
// <span class="icon w-4 h-4"><svg class="icon" aria-label="Home" …>…</svg></span>
```

The `icon` class is always present on the span and on the `<svg>`; classes you
add go on the span, attributes you add go on the `<svg>`, in the order you gave
them. `aria-hidden="true"` is on the `<svg>` unless you set an `aria-label`.

### From a string

```rust
let icon: Icon = "arrow-right".parse()?;       // Err(Unknown("arrow-rite")) otherwise
Icon::from_name("house")                         // Option<Icon>
```

### Every icon, and what it is for

```rust
Icon::ALL.len();                                 // 1799
Icon::House.tags();                              // ["home", "living", "building", …]
Icon::House.categories();                        // [Category::Buildings, …]
Category::Navigation.icons();                    // &[Icon]
Category::ALL;
icons_with_tag("arrow");                         // &[Icon]
all_tags();                                      // every tag, sorted
```

### Search

The same scoring as the Go library: an exact name is 100, a tag hit above a
category hit above a substring, stable order for equal scores.

```rust
use typed_lucide::search;

for hit in search("arrow").take(5) {
    println!("{} {} {:?}", hit.icon, hit.relevance, hit.matched); // Matched::Exact | Tag | Category | Partial
}

search_with(SearchOptions { query: "edit", max: 10, min_relevance: 50, categories: &[Category::Text] });
```

The query is one string, matched whole: `"arrow"` finds icons, `"arrow right"`
finds only what carries those two words side by side. An empty query is every
icon, at relevance 50 and `Matched::All`.

### `serde`

With the `serde` feature, `Icon` serializes as its name and refuses an unknown
one on the way in.

## The CLI

```bash
icon-search search "home"                 # scored hits, name · relevance · why
icon-search search "arrow" --limit 5
icon-search info house                    # tags, categories, and ready-to-paste Rust
icon-search categories
icon-search list --category navigation
icon-search list --tag arrow --limit 10
icon-search list --category navigation --tag arrow
icon-search svg house                     # the markup, for a quick look or a fixture
```

`info` prints the snippet an agent pastes, so nobody guesses a name:

```
house  ·  buildings, home, navigation
tags: home, living, building, residence, architecture

  Icon::House.html()                      the span and the svg
  Icon::House.svg()                       the svg alone
  Icon::House.body()                      the paths, for your own <svg>
  {{ Icon::House.html()|safe }}           askama
```

`skill/SKILL.md` is a Claude Code skill that teaches an agent to use the CLI
instead of grepping for icon names. Install it beside the binary.

## How it is built

```
typed-lucide/
  src/lib.rs          the API: Icon, Category, Attrs, search, errors
  src/icon.rs         GENERATED — the enum, names, bodies
  src/meta.rs         GENERATED — tags, categories
  src/search.rs       scoring, hand-written
  src/bin/icon-search.rs   the CLI (feature "cli")
  generate/           the generator, a workspace member, never published
  skill/SKILL.md      the Claude Code skill
```

`cargo run -p generate -- 1.40.0` downloads that Lucide tag's source archive,
reads `icons/*.svg` and `icons/*.json` (tags, categories), writes `src/icon.rs`
and `src/meta.rs`, and sets the crate version. Generated files carry a
`// GENERATED by typed-lucide/generate from lucide <tag> — do not edit` line.
The consumer's build touches none of this: no `build.rs`, no network, no
`include!` of anything outside `src/`.

A scheduled workflow watches Lucide's releases; a new tag regenerates, runs the
tests, and opens a pull request titled with the tag. Merging it and tagging is a
person's act, and so is `cargo publish`.

### Naming

Kebab to PascalCase, digits kept: `a-arrow-down` → `AArrowDown`, `building-2`
→ `Building2`, `trash-2` → `Trash2`. A name that starts with a digit gets an
underscore. The mapping is a function in the generator, and a test pins it in
both directions for every icon: every kebab name spells its variant, and every
variant name belongs to exactly one icon. The way back is that table rather
than a second function — `arrow-down-0-1` → `ArrowDown01` and `grid-2x2` →
`Grid2x2` are not rules you can run backwards.

### Markup

What `html()` emits, exactly, for `Icon::House`:

```html
<span class="icon"><svg class="icon" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .709-1.528l7-6a2 2 0 0 1 2.582 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg></span>
```

Lucide's own attributes, in Lucide's order, `class="icon"` first. The body is
Lucide's SVG with its outer element gone and the whitespace between the
remaining elements closed up: same elements, same attributes in the same order,
the path data byte for byte.

## Tests

- every generated name round-trips `name()` ↔ `parse()`, and every variant
  name round-trips against its kebab name;
- every body is well-formed XML and contains no `<svg`;
- the markup of one icon is pinned byte for byte (the block above);
- the search's scoring is pinned rung by rung, 100 down to 30, on the Go
  library's own cases;
- the generator emits the same bytes for the same tag, in name order, whatever
  order the archive hands it its icons.

## License

The crate is MIT. The icons are Lucide's, ISC; `LICENSE-LUCIDE` is theirs.
