use std::{collections::BTreeSet, fmt::Display};

use hayagriva::{citationberg::IndependentStyle, Library};

use crate::{keys_to_citations, RE_CITATION, RE_COMMENT, RE_FOOTNOTE};

pub trait BlockType {
    fn len(&self) -> usize;
    fn insert(&mut self, line: String);
    fn cite(&mut self, bib: &Library, style: &IndependentStyle);
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

    fn cite(&mut self, bib: &Library, style: &IndependentStyle) {
        // Get indentation
        let start = RE_COMMENT.captures(&self.lines[0]).unwrap();
        let start = start[0].to_string();

        // Create citation lines
        let citations = keys_to_citations(&self.citations, bib, style);

        // Check if need a blank line before citations
        let last_line = self.lines.last().unwrap();
        if *last_line != *start {
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

    fn cite(&mut self, _bib: &Library, _style: &IndependentStyle) {}
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

    fn cite(&mut self, bib: &Library, style: &IndependentStyle) {
        match self {
            Block::Comment(c) => c.cite(bib, style),
            Block::Code(c) => c.cite(bib, style),
        }
    }
}
