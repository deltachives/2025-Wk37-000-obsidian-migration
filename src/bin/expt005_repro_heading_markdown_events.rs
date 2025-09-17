use pulldown_cmark::{Parser, TextMergeStream};

fn print_events(s: &str) {
    let parser = Parser::new(s);

    let iter = TextMergeStream::new(parser);

    println!();

    for event in iter {
        println!("Event {event:?}");
    }
}

fn main() {
    /*
       Event Start(Heading { level: H1, id: None, classes: [], attrs: [] })
       Event Text(Borrowed("Some Heading"))
       Event End(Heading(H1))
    */
    print_events(
        &r#"
        @ # Some Heading
        @"#
        .replace("@", "")
        .replace("        ", ""),
    );

    /*
       Event Rule
       Event Start(Heading { level: H2, id: None, classes: [], attrs: [] })
       Event Text(Boxed("deprecates: \"[[001 Looking into heading level graph views]]\""))
       Event SoftBreak
       Event Text(Boxed("breaks: \"[[000 Setting up time logging in Obsidian]]\""))
       Event End(Heading(H2))
    */
    print_events(
        &r#"
        @ ---
        @ deprecates: "[[001 Looking into heading level graph views]]"
        @ breaks: "[[000 Setting up time logging in Obsidian]]"
        @ ---
        @ "#
        .replace("@ ", "")
        .replace("        ", ""),
    );

    /*
       Event Start(Heading { level: H2, id: None, classes: [], attrs: [] })
       Event Text(Borrowed("This heading has "))
       Event Start(Link { link_type: Inline, dest_url: Borrowed("https://www.google.com"), title: Borrowed(""), id: Borrowed("") })
       Event Text(Borrowed("links"))
       Event End(Link)
       Event Text(Borrowed(" and "))
       Event Code(Borrowed("code"))
       Event Text(Borrowed(" in it! "))
       Event Code(Borrowed("Many"))
       Event Text(Borrowed(" "))
       Event Code(Borrowed("of"))
       Event Text(Borrowed(" "))
       Event Code(Borrowed("them"))
       Event Text(Borrowed("!"))
       Event End(Heading(H2))
    */
    print_events(
        &r#"
        @ ## This heading has [links](https://www.google.com) and `code` in it! `Many` `of` `them`!
        @ "#
        .replace("@ ", "")
        .replace("        ", ""),
    );

    // At least it doesn't let code blocks in...
    /*
        Event Start(Heading { level: H2, id: None, classes: [], attrs: [] })
        Event Text(Boxed("This heading has a whole code block! ```rust"))
        Event End(Heading(H2))
        Event Start(Paragraph)
        Event Text(Borrowed("hiii"))
        Event End(Paragraph)
        Event Start(CodeBlock(Fenced(Borrowed(""))))
        Event End(CodeBlock)
    */
    print_events(
        &r#"
        @ ## This heading has a whole code block! ```rust
        @ hiii
        @ ```
        @ "#
        .replace("@ ", "")
        .replace("        ", ""),
    );

    /*
       Event Start(Heading { level: H2, id: None, classes: [], attrs: [] })
       Event Text(Borrowed("This heading has a whole code block! "))
       Event Code(Borrowed("rust hiii"))
       Event End(Heading(H2))
    */
    print_events(
        &r#"
        @ ## This heading has a whole code block! ```rust hiii```
        @ "#
        .replace("@ ", "")
        .replace("        ", ""),
    );
}
