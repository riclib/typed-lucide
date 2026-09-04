//! The scoring, rung by rung, as `riclib/icon` scores it:
//!
//! | what matched                  | relevance |
//! |-------------------------------|-----------|
//! | the name, exactly             | 100       |
//! | a tag, exactly                | 90        |
//! | a category, exactly           | 80        |
//! | the name, from the start      | 70        |
//! | the name, anywhere            | 60        |
//! | a tag, from the start         | 50        |
//! | a tag, anywhere               | 40        |
//! | a category, anywhere          | 30        |

use typed_lucide::{Category, Hit, Icon, Matched, SearchOptions, search, search_with};

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
fn the_name_exactly_is_a_hundred() {
    assert_eq!(rung("house", Icon::House), (100, Matched::Exact));
    assert_eq!(
        search("house").next().unwrap().icon,
        Icon::House,
        "and it comes first"
    );
    assert_eq!(
        rung("  HOUSE  ", Icon::House),
        (100, Matched::Exact),
        "trimmed and folded"
    );
}

#[test]
fn a_tag_exactly_is_ninety() {
    assert_eq!(rung("morning", Icon::AlarmClock), (90, Matched::Tag));
    assert_eq!(rung("wheelchair", Icon::Accessibility), (90, Matched::Tag));
}

#[test]
fn a_category_exactly_is_eighty() {
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
fn the_start_of_a_name_is_seventy_and_the_middle_is_sixty() {
    assert_eq!(
        rung("arrow-big", Icon::ArrowBigRight),
        (70, Matched::Partial)
    );
    assert_eq!(
        rung("big-right", Icon::ArrowBigRight),
        (60, Matched::Partial)
    );
    assert_eq!(rung("house", Icon::HousePlug), (70, Matched::Partial));
    assert_eq!(rung("house", Icon::Warehouse), (60, Matched::Partial));
}

#[test]
fn the_start_of_a_tag_is_fifty_and_the_middle_is_forty() {
    // air-vent is tagged "climate-control": the query starts a tag.
    assert_eq!(rung("climate", Icon::AirVent), (50, Matched::Partial));
    // caravan is tagged "mobile home": the query sits inside a tag.
    assert_eq!(rung("home", Icon::Caravan), (40, Matched::Partial));
}

#[test]
fn the_middle_of_a_category_is_thirty() {
    assert_eq!(rung("avigatio", Icon::Barrel), (30, Matched::Partial));
}

#[test]
fn hits_come_back_best_first_and_ties_keep_their_order() {
    let hits: Vec<Hit> = search("house").collect();
    assert!(hits.windows(2).all(|w| w[0].relevance >= w[1].relevance));

    let tied: Vec<&str> = hits
        .iter()
        .filter(|h| h.relevance == 70)
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
}

#[test]
fn a_query_nothing_carries_finds_nothing() {
    assert_eq!(search("qwertyuiop").count(), 0);
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
        assert!(search("arrow").count() > 0);
    }
    let each = started.elapsed() / 20;
    assert!(
        each < std::time::Duration::from_millis(50),
        "a search took {each:?}"
    );
}
