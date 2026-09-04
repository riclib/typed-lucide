//! Finding an icon by what it is, not by what it is called.
//!
//! The scoring is [`riclib/icon`](https://github.com/riclib/icon)'s, so the Go
//! and the Rust library answer a query the same way: an exact name is 100, an
//! exact tag 90, an exact category 80, and a substring is 70 down to 30
//! depending on where it lands. Equal scores keep their order — name first,
//! then tag, then category, then substring, each group sorted by name.

use crate::meta::{SEARCH_TEXT, TAG_INDEX};
use crate::{Category, Icon};

/// Why an icon came back.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Matched {
    /// The query is the icon's name.
    Exact,
    /// The query is one of the icon's tags.
    Tag,
    /// The query is one of the icon's categories.
    Category,
    /// The query appears inside the name, a tag or a category.
    Partial,
    /// The query was empty, so every icon came back.
    All,
}

/// One icon the search found, and how well.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Hit {
    /// The icon.
    pub icon: Icon,
    /// 0 to 100, higher is better.
    pub relevance: u8,
    /// What matched.
    pub matched: Matched,
}

/// What to search for, and what to keep.
///
/// ```
/// # use typed_lucide::{search_with, Category, SearchOptions};
/// let hits = search_with(SearchOptions {
///     query: "edit",
///     max: 10,
///     min_relevance: 50,
///     categories: &[Category::Text],
/// });
/// assert!(hits.count() <= 10);
/// ```
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchOptions<'a> {
    /// The words to look for. Empty means every icon.
    pub query: &'a str,
    /// At most this many hits; 0 for all of them.
    pub max: usize,
    /// Drop hits scoring below this.
    pub min_relevance: u8,
    /// Keep only icons in one of these categories; empty keeps them all.
    pub categories: &'a [Category],
}

/// Icons matching `query`, best first.
///
/// ```
/// # use typed_lucide::{search, Icon, Matched};
/// let first = search("house").next().unwrap();
/// assert_eq!((first.icon, first.relevance, first.matched), (Icon::House, 100, Matched::Exact));
/// ```
pub fn search(query: &str) -> impl ExactSizeIterator<Item = Hit> {
    scored(query).into_iter()
}

/// [`search`] with the hits filtered and cut.
pub fn search_with(options: SearchOptions<'_>) -> impl ExactSizeIterator<Item = Hit> {
    let mut hits = scored(options.query);

    if !options.categories.is_empty() {
        hits.retain(|hit| {
            hit.icon
                .categories()
                .iter()
                .any(|category| options.categories.contains(category))
        });
    }
    if options.min_relevance > 0 {
        hits.retain(|hit| hit.relevance >= options.min_relevance);
    }
    if options.max > 0 {
        hits.truncate(options.max);
    }

    hits.into_iter()
}

fn scored(query: &str) -> Vec<Hit> {
    let query = query.trim().to_lowercase();

    if query.is_empty() {
        return Icon::ALL
            .iter()
            .map(|&icon| Hit {
                icon,
                relevance: 50,
                matched: Matched::All,
            })
            .collect();
    }

    let mut hits = Vec::new();
    let mut seen = vec![false; Icon::COUNT];
    let push = |hits: &mut Vec<Hit>, seen: &mut Vec<bool>, icon: Icon, relevance, matched| {
        if !seen[icon as usize] {
            seen[icon as usize] = true;
            hits.push(Hit {
                icon,
                relevance,
                matched,
            });
        }
    };

    if let Some(icon) = Icon::from_name(&query) {
        push(&mut hits, &mut seen, icon, 100, Matched::Exact);
    }
    if let Ok(i) = TAG_INDEX.binary_search_by(|(tag, _)| (*tag).cmp(query.as_str())) {
        for &icon in TAG_INDEX[i].1 {
            push(&mut hits, &mut seen, icon, 90, Matched::Tag);
        }
    }
    if let Some(category) = Category::from_name(&query) {
        for &icon in category.icons() {
            push(&mut hits, &mut seen, icon, 80, Matched::Category);
        }
    }
    for (i, &icon) in Icon::ALL.iter().enumerate() {
        if seen[i] || !SEARCH_TEXT[i].contains(&query) {
            continue;
        }
        let relevance = partial_relevance(icon, &query);
        if relevance > 0 {
            hits.push(Hit {
                icon,
                relevance,
                matched: Matched::Partial,
            });
        }
    }

    // A stable sort, so equal scores keep the order above.
    hits.sort_by_key(|hit| std::cmp::Reverse(hit.relevance));
    hits
}

/// Where the substring landed, in the order the Go library asks.
fn partial_relevance(icon: Icon, query: &str) -> u8 {
    let name = icon.name();
    if name.starts_with(query) {
        return 70;
    }
    if name.contains(query) {
        return 60;
    }
    if icon
        .tags()
        .iter()
        .any(|tag| tag.to_lowercase().starts_with(query))
    {
        return 50;
    }
    if icon
        .tags()
        .iter()
        .any(|tag| tag.to_lowercase().contains(query))
    {
        return 40;
    }
    if icon
        .categories()
        .iter()
        .any(|category| category.name().contains(query))
    {
        return 30;
    }
    0
}
