use std::{
    fmt::Display,
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use biblatex::Bibliography;
use std::io::Write;

use crate::{
    line::{BlockType, Code, Comment},
    Block, RE_COMMENT,
};

#[derive(Debug, Clone)]
pub struct File {
    filename: PathBuf,
    blocks: Vec<Block>,
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
    pub fn filename(&self) -> &PathBuf {
        &self.filename
    }

    pub fn open(filename: PathBuf) -> Self {
        let file = fs::File::open(filename.clone()).expect("no such file");
        let buf = BufReader::new(file);
        let mut lines = buf
            .lines()
            .map(|l| l.expect("Could not parse line"))
            .peekable();

        let mut blocks = Vec::new();
        let mut current = if RE_COMMENT.is_match(&lines.peek().unwrap()) {
            Block::Comment(Comment::default())
        } else {
            Block::Code(Code::default())
        };

        for line in lines {
            // If we switched from comment to code
            if RE_COMMENT.is_match(&line) {
                if let Block::Code(_) = current {
                    blocks.push(current);
                    current = Block::Comment(Comment::default());
                }
            // If we switched from code to comment
            } else {
                if let Block::Comment(_) = current {
                    blocks.push(current);
                    current = Block::Code(Code::default());
                }
            }

            current.insert(line);
        }

        blocks.push(current);

        Self { blocks, filename }
    }

    pub fn cite(&mut self, bib: &Bibliography) {
        for block in self.blocks.iter_mut() {
            block.cite(bib);
        }
    }

    pub fn save(&self) {
        let mut file = fs::File::create(&self.filename).expect("Could not create file");
        write!(file, "{}", self).expect("Could not write to file");
    }
}
