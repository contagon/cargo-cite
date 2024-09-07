use cargo_cite::{load_bib, load_style, Block, BlockType, File};

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
        assert_eq!(comment.citations().len(), 0);
    } else {
        panic!("First block is not a comment");
    }

    if let Block::Comment(_) = &blocks[1] {
        panic!("Second block is not a code block");
    }
}

#[test]
fn parse_basic_cite() {
    let lines = r#"
    /// This is a comment
    /// that spans multiple lines
    /// And has a citation [^@simple]
    /// And another citation [^@another] that's not in the bib file
    /// And another footnote [^footnote] that's not a citation
    ///
    /// [^@another]: J. Doe and J. Smith, “A very simple title,” The Journal of Rust, May 2013. (should be dropped)
    fn main() {
        color = v_color;
    };
"#;
    let lines = lines.lines().skip(1).map(|l| l.to_string());
    let bib = load_bib("tests/ref.bib").unwrap();
    let style = load_style("ieee").unwrap();

    let mut file = File::open_from_lines(lines, "test.rs".into());
    file.cite(&bib, &style);

    if let Block::Comment(comment) = &file.blocks()[0] {
        println!("{}", comment);
        assert_eq!(comment.citations().len(), 2);
        assert_eq!(comment.len(), 7);
    }
}

#[test]
fn parse_extra_line() {
    let lines = r#"
    /// This is a comment
    /// that spans multiple lines
    /// And has a citation [^@simple]
    /// And another footnote [^footnote] that's not a citation
    /// 
    /// [^footnote]: This is a footnote
    /// [^@simple]: J. Doe and J. Smith, “A very simple title,” The Journal of Rust, May 2013.
    fn main() {
        color = v_color;
    };
"#;
    let lines = lines.lines().skip(1).map(|l| l.to_string());
    let bib = load_bib("tests/ref.bib").unwrap();
    let style = load_style("ieee").unwrap();

    let mut file = File::open_from_lines(lines, "test.rs".into());
    file.cite(&bib, &style);

    if let Block::Comment(comment) = &file.blocks()[0] {
        println!("{}", comment);
        assert_eq!(comment.citations().len(), 1);
        assert_eq!(comment.len(), 7);
    }
}
