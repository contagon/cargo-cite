use std::{collections::BTreeSet, path::Path};

use hayagriva::{
    archive::{locales, ArchivedStyle},
    citationberg::{IndependentStyle, LocaleCode, Style},
    io::from_biblatex_str,
    BibliographyDriver, BibliographyRequest, CitationItem, CitationRequest, Library,
};

/// Convert set of keys to formatted citations
pub fn keys_to_citations(
    keys: &BTreeSet<String>,
    bib: &Library,
    style: &IndependentStyle,
) -> Vec<String> {
    let mut driver = BibliographyDriver::new();
    let locales = locales();

    let entries = keys
        .iter()
        .map(|key| CitationItem::with_entry(bib.get(key).unwrap()))
        .collect();

    driver.citation(CitationRequest::new(
        entries,
        &style,
        Some(LocaleCode::en_us()),
        &locales,
        None,
    ));

    let rendered = driver.finish(BibliographyRequest {
        style: &style,
        locale: Some(LocaleCode::en_us()),
        locale_files: &locales,
    });

    rendered
        .bibliography
        .unwrap()
        .items
        .iter()
        .map(|item| format!("{:#}", item.content))
        .collect()
}

/// Load a style by name
pub fn load_style(name: &str) -> Result<IndependentStyle, String> {
    let style = ArchivedStyle::by_name(name).ok_or("Invalid Style")?.get();
    if let Style::Independent(i) = style {
        Ok(i)
    } else {
        panic!("Could not load style");
    }
}

/// Load a bibliography from a file
pub fn load_bib(path: &str) -> Result<Library, String> {
    let bib = std::fs::read_to_string(path).map_err(|_| "Could not read file")?;
    let bib = from_biblatex_str(&bib).map_err(|_| "Couldn't load bibtex")?;
    Ok(bib)
}
