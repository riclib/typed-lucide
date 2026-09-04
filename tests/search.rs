//! Every word of the query has to land, and each lands on its best rung:
//!
//! | the word …                       | rung |
//! |----------------------------------|------|
//! | (the whole query names the icon) | 100  |
//! | is a word of the name            | 90   |
//! | is a word of a tag               | 90   |
//! | is a category                    | 80   |
//! | sits inside the name             | 60   |
//! | sits inside a tag                | 50   |
//! | sits inside a category           | 30   |
//!
//! The relevance is the mean of the words' rungs. `riclib/icon` for Go instead
//! matches the query as one string, which finds nothing for two words that are
//! not adjacent in an icon's data; the rungs it does reach are pinned here too.

use typed_lucide::{Category, Hit, Icon, Matched, SearchOptions, search, search_with};

fn names(query: &str, count: usize) -> Vec<&'static str> {
    search(query)
        .take(count)
        .map(|hit| hit.icon.name())
        .collect()
}

fn hit(query: &str, icon: Icon) -> Hit {
    search(query)
        .find(|hit| hit.icon == icon)
        .unwrap_or_else(|| panic!("{query:?} did not find {icon}"))
}

fn rung(query: &str, icon: Icon) -> (u8, Matched) {
    let hit = hit(query, icon);
    (hit.relevance, hit.matched)
}

#[test]
fn two_words_find_the_icon_that_carries_both() {
    assert_eq!(
        names("arrow right", 4),
        [
            "arrow-right",
            "arrow-big-right",
            "arrow-big-right-dash",
            "arrow-down-right"
        ],
        "the icon the query names comes first, then the family"
    );
    assert_eq!(rung("arrow right", Icon::ArrowRight), (100, Matched::Exact));
    assert_eq!(
        rung("arrow right", Icon::ArrowBigRight),
        (90, Matched::Exact)
    );
    assert!(
        !search("arrow right").any(|hit| hit.icon == Icon::ArrowLeft),
        "an arrow that does not point right misses a word"
    );
}

#[test]
fn a_two_word_tag_is_found_by_its_words() {
    // "font size" is one tag of a-arrow-down; the Go library finds it only
    // because those two words sit side by side, and finds nothing when they do
    // not.
    assert_eq!(
        names("font size", 4),
        [
            "a-arrow-down",
            "a-arrow-up",
            "a-large-small",
            "text-initial"
        ]
    );
    assert_eq!(rung("font size", Icon::AArrowDown), (90, Matched::Tag));
    assert_eq!(rung("size font", Icon::AArrowDown), (90, Matched::Tag));
}

#[test]
fn word_order_is_not_a_thing() {
    let one: Vec<Hit> = search("arrow right").collect();
    let other: Vec<Hit> = search("right arrow").collect();
    assert_eq!(one, other);
    assert_eq!(
        search("arrow-right").collect::<Vec<_>>(),
        one,
        "hyphens part words too"
    );
}

#[test]
fn a_word_nobody_carries_ends_the_search() {
    assert_eq!(search("arrow qwertyuiop").count(), 0);
    assert_eq!(search("qwertyuiop").count(), 0);
}

#[test]
fn the_name_exactly_is_a_hundred() {
    assert_eq!(rung("house", Icon::House), (100, Matched::Exact));
    assert_eq!(names("house", 1), ["house"], "and it leads");
    assert_eq!(
        rung("  HOUSE  ", Icon::House),
        (100, Matched::Exact),
        "trimmed and folded"
    );
    assert_eq!(
        rung("a-arrow-down", Icon::AArrowDown),
        (100, Matched::Exact)
    );
}

#[test]
fn a_word_of_the_name_or_of_a_tag_is_ninety() {
    assert_eq!(rung("house", Icon::HousePlug), (90, Matched::Exact));
    assert_eq!(rung("morning", Icon::AlarmClock), (90, Matched::Tag));
    assert_eq!(rung("wheelchair", Icon::Accessibility), (90, Matched::Tag));
    // A tag written with a hyphen ("climate-control") or a space ("mobile
    // home") gives up its words like anything else.
    assert_eq!(rung("climate", Icon::AirVent), (90, Matched::Tag));
    assert_eq!(rung("home", Icon::Caravan), (90, Matched::Tag));
}

#[test]
fn a_category_is_eighty() {
    assert_eq!(rung("navigation", Icon::Barrel), (80, Matched::Category));
    assert_eq!(
        rung("navigation", Icon::Navigation),
        (100, Matched::Exact),
        "the name wins"
    );
    assert_eq!(
        rung("navigation", Icon::Map),
        (90, Matched::Tag),
        "and a tag beats a category"
    );
}

#[test]
fn sitting_inside_the_name_is_sixty_a_tag_fifty_a_category_thirty() {
    assert_eq!(rung("ouse", Icon::House), (60, Matched::Partial));
    assert_eq!(rung("limate", Icon::AirVent), (50, Matched::Tag));
    assert_eq!(rung("avigatio", Icon::Barrel), (30, Matched::Category));
}

#[test]
fn the_relevance_is_the_mean_of_the_words() {
    // apple: "food" is one of its tag words (90), "beverage" only a word of its
    // food-beverage category (80).
    assert_eq!(rung("food beverage", Icon::Apple), (85, Matched::Tag));
    assert_eq!(rung("food", Icon::Apple), (90, Matched::Tag));
    assert_eq!(rung("beverage", Icon::Apple), (80, Matched::Category));
}

#[test]
fn hits_come_back_best_first_and_ties_keep_name_order() {
    let hits: Vec<Hit> = search("house").collect();
    assert!(hits.windows(2).all(|w| w[0].relevance >= w[1].relevance));

    let tied: Vec<&str> = hits
        .iter()
        .filter(|h| h.relevance == 90)
        .map(|h| h.icon.name())
        .collect();
    let mut sorted = tied.clone();
    sorted.sort_unstable();
    assert_eq!(tied, sorted, "equal scores stay in name order");

    // Twice over, the same answer.
    assert_eq!(search("house").collect::<Vec<_>>(), hits);
}

#[test]
fn an_icon_is_named_once_however_many_ways_it_matches() {
    let hits: Vec<Hit> = search("home").collect();
    let mut seen: Vec<Icon> = hits.iter().map(|h| h.icon).collect();
    let before = seen.len();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), before);
    assert_eq!(
        rung("home", Icon::House),
        (90, Matched::Tag),
        "home is a tag of house, not its name"
    );
}

#[test]
fn an_empty_query_is_every_icon() {
    let hits: Vec<Hit> = search("").collect();
    assert_eq!(hits.len(), Icon::COUNT);
    assert!(
        hits.iter()
            .all(|h| h.relevance == 50 && h.matched == Matched::All)
    );
    assert_eq!(search("   ").count(), Icon::COUNT);
    assert_eq!(search("-").count(), Icon::COUNT);
}

#[test]
fn options_filter_and_cut() {
    let all = search("edit").count();
    assert!(all > 10);

    let capped = search_with(SearchOptions {
        query: "edit",
        max: 3,
        ..Default::default()
    });
    assert_eq!(capped.count(), 3);

    let strict: Vec<Hit> = search_with(SearchOptions {
        query: "edit",
        min_relevance: 90,
        ..Default::default()
    })
    .collect();
    assert!(!strict.is_empty());
    assert!(strict.iter().all(|h| h.relevance >= 90));

    let text: Vec<Hit> = search_with(SearchOptions {
        query: "edit",
        max: 10,
        min_relevance: 50,
        categories: &[Category::Text],
    })
    .collect();
    assert!(!text.is_empty());
    assert!(
        text.iter()
            .all(|h| h.icon.categories().contains(&Category::Text))
    );

    let unlimited = search_with(SearchOptions {
        query: "edit",
        ..Default::default()
    });
    assert_eq!(unlimited.count(), all, "max 0 keeps them all");
}

#[test]
fn a_search_of_every_icon_is_quick() {
    let started = std::time::Instant::now();
    for _ in 0..20 {
        assert!(search("arrow right").count() > 0);
    }
    let each = started.elapsed() / 20;
    assert!(
        each < std::time::Duration::from_millis(50),
        "a search took {each:?}"
    );
}
