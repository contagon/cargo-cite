use core::fmt;
use std::{collections::BTreeSet, path::Path};

use hayagriva::{
    archive::{locales, ArchivedStyle},
    citationberg::{IndependentStyle, LocaleCode, Style},
    io::from_biblatex_str,
    BibliographyDriver, BibliographyRequest, CitationItem, CitationRequest, Library,
};
use thiserror::Error;

// ------------------------- Keys -> Citations ------------------------- //
/// Convert set of keys to formatted citations
// TODO: Return Nones for missing keys?? Then can process w/ exact line number or something
pub fn keys_to_citations(
    keys: &BTreeSet<String>,
    bib: &Library,
    style: &IndependentStyle,
    file: impl AsRef<Path>,
) -> Vec<String> {
    if keys.is_empty() {
        return Vec::new();
    }

    let mut driver = BibliographyDriver::new();
    let locales = locales();

    let entries = keys
        .iter()
        .filter_map(|key| {
            let cite = bib.get(key);
            if cite.is_none() {
                log::warn!(
                    "{:?}: Key @{} not found in the bib file",
                    file.as_ref(),
                    key
                );
            }
            cite
        })
        .map(CitationItem::with_entry)
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
        .expect("Bibliography failed to render")
        .items
        .iter()
        .map(|item| format!("{:#}", item.content))
        .collect()
}

// ------------------------- Loading Style ------------------------- //
/// Error type for loading a style
#[derive(Debug, Error)]
pub enum StyleError {
    #[error("Invalid style name, see hayagriva::archive::ArchivedStyle")]
    InvalidStyle,
    #[error("Style is not independent")]
    DependentStyle,
}

/// Load a style by name
pub fn load_style(name: &str) -> Result<IndependentStyle, StyleError> {
    let style = ArchivedStyle::by_name(name)
        .ok_or(StyleError::InvalidStyle)?
        .get();
    if let Style::Independent(i) = style {
        Ok(i)
    } else {
        Err(StyleError::DependentStyle)
    }
}

// ------------------------- Loading bibliography ------------------------- //
/// Error type for loading a bibliography
#[derive(Debug, Error)]
pub enum BibError {
    Io(#[from] std::io::Error),
    Bib(Vec<hayagriva::io::BibLaTeXError>),
}

impl From<Vec<hayagriva::io::BibLaTeXError>> for BibError {
    fn from(e: Vec<hayagriva::io::BibLaTeXError>) -> Self {
        BibError::Bib(e)
    }
}

impl fmt::Display for BibError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BibError::Io(e) => write!(f, "{}", e),
            BibError::Bib(e) => {
                for e in e {
                    write!(f, "{}", e)?;
                }
                Ok(())
            }
        }
    }
}

/// Load a bibliography from a file
pub fn load_bib(path: &str) -> Result<Library, BibError> {
    let bib = std::fs::read_to_string(path)?;
    let bib = from_biblatex_str(&bib)?;
    Ok(bib)
}
