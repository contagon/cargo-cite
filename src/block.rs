use std::{collections::BTreeSet, fmt::Display, path::Path};

use hayagriva::{citationberg::IndependentStyle, Library};
use regex::Regex;

use crate::keys_to_citations;
use lazy_static::lazy_static;

lazy_static! {
    // Matches citations inline
    pub static ref RE_CITATION: Regex = Regex::new(r"\[\^@(.*?)\]").unwrap();
    // Matches comments (ie line begins with //! or ///)
    pub static ref RE_COMMENT: Regex = Regex::new(r"^[ \t]*//[/!]").unwrap();
    // Matches footnotes (ie line begins with /// [^@citekey]:)
    pub static ref RE_FOOTNOTE: Regex = Regex::new(r"^[ \t]*//[/!]\s*\[\^@(.*?)\]:").unwrap();
}

pub trait BlockType {
    fn len(&self) -> usize;
    fn insert(&mut self, line: String);
    fn cite(&mut self, bib: &Library, style: &IndependentStyle, file: impl AsRef<Path>);
}

// ------------------------- Comment block ------------------------- //

#[derive(Debug, Clone)]
pub struct Comment {
    lines: Vec<String>,
    citations: BTreeSet<String>,
}

impl Default for Comment {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            citations: BTreeSet::new(),
        }
    }
}

impl Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl BlockType for Comment {
    fn len(&self) -> usize {
        self.lines.len()
    }

    fn insert(&mut self, line: String) {
        // Check if the line contains a reference
        if RE_CITATION.is_match(&line) {
            // If it's from the footnote, don't keep the line at all
            if RE_FOOTNOTE.is_match(&line) {
                return;
            // Otherwise, cache the citation
            } else {
                for cite in RE_CITATION.captures_iter(&line) {
                    self.citations.insert(cite[1].to_string());
                }
            }
        }
        self.lines.push(line);
    }

    fn cite(&mut self, bib: &Library, style: &IndependentStyle, file: impl AsRef<Path>) {
        // Get indentation
        let start = RE_COMMENT
            .captures(&self.lines[0])
            .expect("Comment found without delimiter");
        let start = start[0].to_string();

        // Create citation lines
        let citations = keys_to_citations(&self.citations, bib, style, file);

        // Check if need a blank line before citations
        let last_line = self.lines.last().expect("Comment found without any lines");
        if !citations.is_empty() && *last_line != *start {
            self.lines.push(start.clone());
        }

        // Insert citations
        for (key, cite) in self.citations.iter().zip(citations) {
            self.lines.push(format!("{} [^@{}]: {}", start, key, cite));
        }
    }
}

// ------------------------- Code block ------------------------- //
#[derive(Debug, Clone)]
pub struct Code {
    lines: Vec<String>,
}

impl Default for Code {
    fn default() -> Self {
        Self { lines: Vec::new() }
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl BlockType for Code {
    fn len(&self) -> usize {
        self.lines.len()
    }

    fn insert(&mut self, line: String) {
        self.lines.push(line);
    }

    fn cite(&mut self, _bib: &Library, _style: &IndependentStyle, _path: impl AsRef<Path>) {}
}

// ------------------------- Block ------------------------- //
#[derive(Debug, Clone)]
pub enum Block {
    Comment(Comment),
    Code(Code),
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Block::Comment(c) => write!(f, "{}", c),
            Block::Code(c) => write!(f, "{}", c),
        }
    }
}

// This could be done with enum dispatch...
// but it's simple enough to do by hand
impl BlockType for Block {
    fn len(&self) -> usize {
        match self {
            Block::Comment(c) => c.len(),
            Block::Code(c) => c.len(),
        }
    }

    fn insert(&mut self, line: String) {
        match self {
            Block::Comment(c) => c.insert(line),
            Block::Code(c) => c.insert(line),
        }
    }

    fn cite(&mut self, bib: &Library, style: &IndependentStyle, file: impl AsRef<Path>) {
        match self {
            Block::Comment(c) => c.cite(bib, style, file),
            Block::Code(c) => c.cite(bib, style, file),
        }
    }
}
