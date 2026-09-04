---
name: typed-lucide
description: Find Lucide icons for Rust code with the icon-search CLI from the typed-lucide crate. Use when writing or editing Rust/askama/maud code that needs an icon. NEVER grep the crate source, src/icon.rs or the registry for icon names — ask icon-search.
---

# typed-lucide

1799 Lucide icons as Rust enum variants, from the `typed-lucide` crate. A name
that does not exist is a compile error, so get the name right first.

## Commands

The binary is `icon-search`, on PATH after
`cargo install typed-lucide --features cli`.

### Search for icons by keyword

```bash
icon-search search "home"
icon-search search "arrow" --limit 5
```

Each line is `name · relevance · why`, best first. `why` is `exact`, `tag`,
`category` or `partial`.

### Get full info for one icon

```bash
icon-search info house
icon-search info arrow-right
```

Tags, categories, and the Rust to paste. An unknown name exits 1 and suggests
near misses.

### List all categories

```bash
icon-search categories
```

### List and filter icons

```bash
icon-search list --category navigation
icon-search list --tag arrow --limit 10
icon-search list --category navigation --tag arrow
```

### Look at the markup

```bash
icon-search svg house
```

## Using icons in Rust

### The dependency

```toml
[dependencies]
typed-lucide = "=1.40.0"
```

```rust
use typed_lucide::{Attrs, Icon};
```

### The three usages

```rust
Icon::House.html()   // <span class="icon"><svg class="icon" …>…</svg></span>
Icon::House.svg()    // the <svg> alone
Icon::House.body()   // the <path> elements, for an <svg> you write yourself
```

### With classes and attributes

```rust
Icon::House.render(&Attrs::new().class("w-4 h-4").attr("aria-label", "Home"))
```

Classes land on the span, attributes on the `<svg>`. An icon carries
`aria-hidden="true"` unless you give it an `aria-label`.

### In an askama template

```jinja
<button class="ad-icon-btn" title="Home">{{ Icon::House.html()|safe }}</button>
```

The template's context needs the icon; `html()` returns a `String`, so pipe it
through `|safe` (or `PreEscaped` in maud) or the markup will be escaped.

### From a name at runtime

```rust
let icon: Icon = "arrow-right".parse()?;   // Err(Unknown(name)) if Lucide has no such icon
Icon::from_name("house")                   // Option<Icon>
```

## Naming convention

Icon names are kebab-case; the variant is the same name in PascalCase.

- Lucide name: `arrow-big-right` → variant: `Icon::ArrowBigRight`
- `house` → `Icon::House`
- `building-2` → `Icon::Building2`
- `grid-2x2` → `Icon::Grid2x2`
- `arrow-down-0-1` → `Icon::ArrowDown01`

The last two are why you ask `icon-search info <name>` for the variant instead
of spelling it out yourself.
