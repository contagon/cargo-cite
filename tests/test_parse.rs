use cargo_cite::{keys_to_citations, load_bib, load_style, scan_for_key, Block, BlockType, File};

#[test]
fn parse_no_cites() {
    let lines = r#"
    /// This is a comment
    /// that spans multiple lines
    fn main() {
        color = v_color;
    };
"#;
    let lines = lines.lines().skip(1).map(|l| l.to_string());
    let file = File::open_from_lines(lines, "test.rs".into());

    let blocks = file.blocks();

    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].len(), 2);
    assert_eq!(blocks[1].len(), 3);

    if let Block::Comment(comment) = &blocks[0] {
        assert_eq!(comment.keys().len(), 0);
    } else {
        panic!("First block is not a comment");
    }

    if let Block::Comment(_) = &blocks[1] {
        panic!("Second block is not a code block");
    }
}

#[test]
fn parse_basic_cite() {
    let raw_lines = r#"
    /// This is a comment
    /// that spans multiple lines
    /// And has a citation [^@simple]
    /// And another citation [^@another] that's not in the bib file
    /// And another footnote [^footnote] that's not a citation
    ///
    /// [^@simple]: Doe, John, and Jane Smith. “A Very Simple Title.” The Journal of Rust, May 2013
    fn main() {
        color = v_color;
    };
"#;
    let lines = raw_lines.lines().skip(1).map(|l| l.to_string());
    let bib = load_bib("tests/ref.bib").unwrap();
    let style = load_style("ieee").unwrap();

    let keys = scan_for_key(raw_lines);
    let citations = keys_to_citations(keys, &bib, &style);

    let mut file = File::open_from_lines(lines, "test.rs".into());
    file.cite(&citations);

    if let Block::Comment(comment) = &file.blocks()[0] {
        println!("{}", comment);
        assert_eq!(comment.keys().len(), 2);
        assert_eq!(comment.len(), 7);
    }
}

#[test]
fn parse_extra_line() {
    let raw_lines = r#"
    /// This is a comment
    /// that spans multiple lines
    /// And has a citation [^@simple]
    /// And another footnote [^footnote] that's not a citation
    /// 
    /// [^footnote]: This is a footnote
    /// [^@simple]: Doe, John, and Jane Smith. “A Very Simple Title.” The Journal of Rust, May 2013
    fn main() {
        color = v_color;
    };
"#;
    let lines = raw_lines.lines().skip(1).map(|l| l.to_string());
    let bib = load_bib("tests/ref.bib").unwrap();
    let style = load_style("ieee").unwrap();

    let keys = scan_for_key(raw_lines);
    let citations = keys_to_citations(keys, &bib, &style);

    let mut file = File::open_from_lines(lines, "test.rs".into());
    file.cite(&citations);

    if let Block::Comment(comment) = &file.blocks()[0] {
        println!("{}", comment);
        assert_eq!(comment.keys().len(), 1);
        assert_eq!(comment.len(), 7);
    }
}
