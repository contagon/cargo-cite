use std::{
    collections::{BTreeSet, HashMap},
    fmt::Display,
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use std::io::Write;

use crate::{
    block::{BlockType, Code, Comment, RE_COMMENT},
    Block, Key,
};

#[derive(Debug, Clone)]
pub struct File {
    filename: PathBuf,
    blocks: Vec<Block>,
    keys: BTreeSet<Key>,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for block in &self.blocks {
            write!(f, "{}", block)?;
        }
        Ok(())
    }
}

impl File {
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn filename(&self) -> &PathBuf {
        &self.filename
    }

    pub fn open(filename: PathBuf) -> Self {
        let file = fs::File::open(filename.clone()).expect("no such file");
        let buf = BufReader::new(file);
        let lines = buf.lines().map(|l| l.expect("Could not parse line"));

        Self::open_from_lines(lines, filename)
    }

    pub fn open_from_lines(lines: impl Iterator<Item = String>, filename: PathBuf) -> Self {
        let mut lines = lines.peekable();

        // Handle the case of empty files
        let first = match lines.peek() {
            Some(first) => first,
            None => {
                return Self {
                    blocks: Vec::new(),
                    filename,
                    keys: BTreeSet::new(),
                }
            }
        };

        let mut blocks = Vec::new();
        let mut current = if RE_COMMENT.is_match(first) {
            Block::Comment(Comment::default())
        } else {
            Block::Code(Code::default())
        };

        for line in lines {
            // If we switched from code to comment
            if RE_COMMENT.is_match(&line) {
                if let Block::Code(_) = current {
                    blocks.push(current);
                    current = Block::Comment(Comment::default());
                }
            // If we switched from comment to code
            } else {
                if let Block::Comment(_) = current {
                    blocks.push(current);
                    current = Block::Code(Code::default());
                }
            }

            current.insert(line);
        }

        blocks.push(current);
        let keys = blocks
            .iter()
            .map(|b| b.keys())
            .flatten()
            .flatten()
            .cloned()
            .collect();

        Self {
            blocks,
            filename,
            keys,
        }
    }

    pub fn cite(&mut self, citations: &HashMap<Key, String>) {
        for block in self.blocks.iter_mut() {
            block.cite(citations);
        }
    }

    pub fn keys(&self) -> &BTreeSet<Key> {
        &self.keys
    }

    pub fn save(&self) {
        let mut file = fs::File::create(&self.filename).expect("Could not create file");
        write!(file, "{}", self).expect("Could not write to file");
    }
}
