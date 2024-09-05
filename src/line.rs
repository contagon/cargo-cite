use std::{collections::BTreeSet, fmt::Display};

use enum_dispatch::enum_dispatch;

use crate::{RE_CITATION, RE_COMMENT, RE_FOOTNOTE};

#[enum_dispatch]
pub trait BlockType {
    fn len(&self) -> usize;
    fn insert(&mut self, line: String);
    fn cite(&mut self, bib: &crate::Bibliography);
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
            // If it's from the footnote, don't keep it
            if RE_FOOTNOTE.is_match(&line) {
                return;
            // Otherwise, keep them
            } else {
                for cite in RE_CITATION.captures_iter(&line) {
                    self.citations.insert(cite[1].to_string());
                }
            }
        }
        self.lines.push(line);
    }

    fn cite(&mut self, bib: &crate::Bibliography) {
        // Get indentation
        let start = RE_COMMENT.captures(&self.lines[0]).unwrap();
        let start = start[0].to_string();
        println!("Start: {}", start);

        // Create citation lines
        for cite in &self.citations {
            if let Some(entry) = bib.get(cite) {
                // TODO: Proper citation
                let citation = format!("{} [^@{}]: {:?}", start, cite, entry);
                self.lines.push(citation);
            } else {
                println!("@{} citekey not in bib file", cite);
            }
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

    fn cite(&mut self, _bib: &crate::Bibliography) {}
}

// ------------------------- Block ------------------------- //
#[enum_dispatch(BlockType)]
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
