use std::{
    collections::{BTreeSet, HashMap},
    fmt::Display,
};

use regex::Regex;

use crate::Key;
use lazy_static::lazy_static;

lazy_static! {
    // Matches citations inline
    pub static ref RE_CITATION: Regex = Regex::new(r"\[\^@(.*?)\]").unwrap();
    // Matches comments (ie line begins with //! or ///)
    pub static ref RE_COMMENT: Regex = Regex::new(r"^[ \t]*//[/!]").unwrap();
    // Matches citation footnote (ie line begins with /// [^@citekey]:)
    pub static ref RE_CITE_FOOTNOTE: Regex = Regex::new(r"^[ \t]*//[/!]\s*\[\^@(.*?)\]:").unwrap();
    // Matches any footnote (ie line begins with /// [^footnote]:)
    pub static ref RE_FOOTNOTE: Regex = Regex::new(r"^[ \t]*//[/!]\s*\[\^.*\]:").unwrap();
}

pub trait BlockType {
    fn len(&self) -> usize;
    fn insert(&mut self, line: String);
    fn cite(&mut self, citations: &HashMap<Key, String>);
    fn keys(&self) -> Option<&BTreeSet<Key>>;
}

// ------------------------- Comment block ------------------------- //

#[derive(Debug, Clone)]
pub struct Comment {
    lines: Vec<String>,
    keys: BTreeSet<Key>,
}

impl Default for Comment {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            keys: BTreeSet::new(),
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
            if RE_CITE_FOOTNOTE.is_match(&line) {
                return;
            // Otherwise, cache the citation
            } else {
                for cite in RE_CITATION.captures_iter(&line) {
                    log::info!("Citation found: {}", &cite[1]);
                    self.keys.insert(Key(cite[1].to_string()));
                }
            }
        }
        self.lines.push(line);
    }

    fn cite(&mut self, citations: &HashMap<Key, String>) {
        // Get indentation
        let start = RE_COMMENT
            .captures(&self.lines[0])
            .expect("Comment found without delimiter");
        let start = start[0].to_string();

        // Check if need a blank line before citations
        let last_line = self.lines.last().expect("Comment found without any lines");
        // If there is a citation & the last line is not a normal footnote & the last line isn't already a blank line
        // then add a blank line
        if !citations.is_empty() && !RE_FOOTNOTE.is_match(last_line) && *last_line != start {
            self.lines.push(start.clone());
        }

        // Insert citations
        for key in self.keys.iter() {
            if let Some(cite) = citations.get(key) {
                self.lines
                    .push(format!("{} [^@{}]: {}", start, key.0, cite));
            }
        }
    }

    fn keys(&self) -> Option<&BTreeSet<Key>> {
        Some(&self.keys)
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

    fn cite(&mut self, _citations: &HashMap<Key, String>) {}

    fn keys(&self) -> Option<&BTreeSet<Key>> {
        None
    }
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

    fn cite(&mut self, citations: &HashMap<Key, String>) {
        match self {
            Block::Comment(c) => c.cite(citations),
            Block::Code(c) => c.cite(citations),
        }
    }

    fn keys(&self) -> Option<&BTreeSet<Key>> {
        match self {
            Block::Comment(c) => c.keys(),
            Block::Code(c) => c.keys(),
        }
    }
}
