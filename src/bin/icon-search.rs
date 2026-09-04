//! `icon-search` — find a Lucide icon without grepping for one.

use std::io::Write;

use clap::{Parser, Subcommand};
use typed_lucide::{Category, Icon, Matched, SearchOptions, icons_with_tag, search_with};

#[derive(Parser)]
#[command(
    name = "icon-search",
    about = "Query the typed-lucide icon library",
    version,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scored hits for a query: name, relevance, why.
    Search {
        /// The words to look for.
        #[arg(required = true)]
        query: Vec<String>,
        /// At most this many hits; 0 for all of them.
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Tags, categories and ready-to-paste Rust for one icon.
    Info {
        /// The icon's kebab name, e.g. `arrow-big-right`.
        name: String,
    },
    /// Every Lucide category, with how many icons it holds.
    Categories,
    /// Icon names, filtered.
    List {
        /// Only icons in this category.
        #[arg(long)]
        category: Option<String>,
        /// Only icons carrying this tag.
        #[arg(long)]
        tag: Option<String>,
        /// At most this many names; 0 for all of them.
        #[arg(long, default_value_t = 0)]
        limit: usize,
    },
    /// The `<svg>` markup of one icon.
    Svg {
        /// The icon's kebab name.
        name: String,
    },
}

/// Print a line, and stop quietly when the reader has gone (`| head`).
macro_rules! line {
    ($($arg:tt)*) => {
        if writeln!(std::io::stdout(), $($arg)*).is_err() {
            return;
        }
    };
}

fn main() {
    match Cli::parse().command {
        Command::Search { query, limit } => search(&query.join(" "), limit),
        Command::Info { name } => info(&icon(&name)),
        Command::Categories => categories(),
        Command::List {
            category,
            tag,
            limit,
        } => list(category.as_deref(), tag.as_deref(), limit),
        Command::Svg { name } => svg(&icon(&name)),
    }
}

fn search(query: &str, limit: usize) {
    let hits: Vec<_> = search_with(SearchOptions {
        query,
        max: limit,
        ..Default::default()
    })
    .collect();
    if hits.is_empty() {
        eprintln!("nothing matches {query:?}");
        std::process::exit(1);
    }
    for hit in hits {
        line!(
            "{:<30} · {:>3} · {}",
            hit.icon.name(),
            hit.relevance,
            why(hit.matched)
        );
    }
}

fn svg(icon: &Icon) {
    line!("{}", icon.svg());
}

fn info(icon: &Icon) {
    let categories = icon
        .categories()
        .iter()
        .map(|c| c.name())
        .collect::<Vec<_>>()
        .join(", ");
    line!("{}  ·  {}", icon.name(), categories);
    line!("tags: {}", icon.tags().join(", "));
    line!();
    let variant = format!("Icon::{icon:?}");
    line!("  {:<40}the span and the svg", format!("{variant}.html()"));
    line!("  {:<40}the svg alone", format!("{variant}.svg()"));
    line!(
        "  {:<40}the paths, for your own <svg>",
        format!("{variant}.body()")
    );
    line!("  {:<40}askama", format!("{{{{ {variant}.html()|safe }}}}"));
}

fn categories() {
    for category in Category::ALL {
        line!("{:<20} {:>4}", category.name(), category.icons().len());
    }
}

fn list(category: Option<&str>, tag: Option<&str>, limit: usize) {
    let mut icons: Vec<Icon> = match category {
        Some(name) => match Category::from_name(&name.to_lowercase()) {
            Some(category) => category.icons().to_vec(),
            None => {
                eprintln!("no such category: {name}");
                std::process::exit(1);
            }
        },
        None => Icon::ALL.to_vec(),
    };
    if let Some(tag) = tag {
        let tagged = icons_with_tag(tag);
        icons.retain(|icon| tagged.contains(icon));
    }
    if limit > 0 {
        icons.truncate(limit);
    }
    for icon in icons {
        line!("{:<30} Icon::{icon:?}", icon.name());
    }
}

fn icon(name: &str) -> Icon {
    match Icon::from_name(&name.to_lowercase()) {
        Some(icon) => icon,
        None => {
            eprintln!("no lucide icon named {name:?}");
            let near = search_with(SearchOptions {
                query: name,
                max: 5,
                ..Default::default()
            })
            .map(|hit| hit.icon.name())
            .collect::<Vec<_>>();
            if !near.is_empty() {
                eprintln!("did you mean: {}", near.join(", "));
            }
            std::process::exit(1);
        }
    }
}

fn why(matched: Matched) -> &'static str {
    match matched {
        Matched::Exact => "exact",
        Matched::Tag => "tag",
        Matched::Category => "category",
        Matched::Partial => "partial",
        Matched::All => "all",
    }
}
