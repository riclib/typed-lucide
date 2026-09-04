//! Finding an icon by what it is, not by what it is called.
//!
//! A query is cut into words on whitespace and hyphens, and so is an icon's
//! name and each of its tags. **Every query word has to land somewhere**, so
//! `"arrow right"` finds the arrows that also point right and nothing else.
//! Each word scores the best rung it reaches:
//!
//! | the word …                          | rung |
//! |-------------------------------------|------|
//! | (the whole query names the icon)    | 100  |
//! | is a word of the name               | 90   |
//! | is a word of a tag                  | 90   |
//! | is a category                       | 80   |
//! | sits inside the name                | 60   |
//! | sits inside a tag                   | 50   |
//! | sits inside a category              | 30   |
//!
//! The icon's relevance is the mean of its words' rungs, except that a query
//! naming the icon outright is 100. Equal scores keep name order, so the same
//! query always gives the same list, and word order never matters:
//! `search("right arrow")` and `search("arrow right")` are the same search.

use std::cmp::Reverse;

use crate::meta::{SEARCH_TEXT, TAG_WORDS};
use crate::{Category, Icon};

/// Why an icon came back.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Matched {
    /// The query names the icon, or its best word is a word of the name.
    Exact,
    /// Its best word is a tag, or sits inside one.
    Tag,
    /// Its best word is a category, or sits inside one.
    Category,
    /// Its best word only sits inside the name.
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
    /// What the icon's best word matched.
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
    /// The words to look for; every one of them has to land. Empty means every
    /// icon.
    pub query: &'a str,
    /// At most this many hits; 0 for all of them.
    pub max: usize,
    /// Drop hits scoring below this.
    pub min_relevance: u8,
    /// Keep only icons in one of these categories; empty keeps them all.
    pub categories: &'a [Category],
}

/// Icons matching every word of `query`, best first.
///
/// ```
/// # use typed_lucide::{search, Icon, Matched};
/// let first = search("arrow right").next().unwrap();
/// assert_eq!((first.icon, first.relevance, first.matched), (Icon::ArrowRight, 100, Matched::Exact));
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

/// Whitespace and hyphens part words, in a query and in what it is matched
/// against alike.
fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| c.is_whitespace() || c == '-')
        .filter(|word| !word.is_empty())
}

fn scored(query: &str) -> Vec<Hit> {
    let query = query.to_lowercase();
    let query: Vec<&str> = words(&query).collect();

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

    // What the query would be as a name: only a name of that length can be it.
    let named_length = query.iter().map(|word| word.len()).sum::<usize>() + query.len() - 1;

    // Icon::ALL is in name order and the sort below is stable, so equal scores
    // come back in name order.
    let mut hits: Vec<Hit> = Icon::ALL
        .iter()
        .enumerate()
        .filter_map(|(index, &icon)| score(icon, index, &query, named_length))
        .collect();
    hits.sort_by_key(|hit| Reverse(hit.relevance));
    hits
}

fn score(icon: Icon, index: usize, query: &[&str], named_length: usize) -> Option<Hit> {
    if icon.name().len() == named_length && names_the_icon(icon, query) {
        return Some(Hit {
            icon,
            relevance: 100,
            matched: Matched::Exact,
        });
    }

    let mut total = 0u32;
    let mut best = (0u8, Matched::Partial);
    for &word in query {
        // One word nobody carries, and the icon is not a hit at all.
        let (rung, matched) = self::rung(icon, index, word)?;
        total += u32::from(rung);
        // A better rung wins; on a tie the better kind does, so the answer does
        // not depend on the order the words were typed in.
        if rung > best.0 || (rung == best.0 && matched < best.1) {
            best = (rung, matched);
        }
    }

    let count = query.len() as u32;
    Some(Hit {
        icon,
        relevance: ((total + count / 2) / count) as u8,
        matched: best.1,
    })
}

/// The query is the icon's name, whatever order its words came in:
/// `"arrow right"`, `"arrow-right"` and `"right arrow"` all name `arrow-right`.
fn names_the_icon(icon: Icon, query: &[&str]) -> bool {
    let name = icon.name();
    words(name).count() == query.len()
        && words(name).all(|word| query.contains(&word))
        && query
            .iter()
            .all(|word| words(name).any(|part| part == *word))
}

fn rung(icon: Icon, index: usize, word: &str) -> Option<(u8, Matched)> {
    // Every rung below needs the word to appear somewhere in the icon's name,
    // tags or categories, and this is one search over all three.
    if !SEARCH_TEXT[index].contains(word) {
        return None;
    }

    let name = icon.name();
    if words(name).any(|part| part == word) {
        return Some((90, Matched::Exact));
    }
    if TAG_WORDS[index].contains(&word) {
        return Some((90, Matched::Tag));
    }
    if icon
        .categories()
        .iter()
        .any(|category| words(category.name()).any(|part| part == word))
    {
        return Some((80, Matched::Category));
    }
    if name.contains(word) {
        return Some((60, Matched::Partial));
    }
    if TAG_WORDS[index].iter().any(|tag| tag.contains(word)) {
        return Some((50, Matched::Tag));
    }
    if icon
        .categories()
        .iter()
        .any(|category| category.name().contains(word))
    {
        return Some((30, Matched::Category));
    }
    None
}
