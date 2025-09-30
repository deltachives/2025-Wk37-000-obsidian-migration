//! Testing that our obsidian patching of rendered markdown works correctly

use migration_rs::*;
use tap::prelude::*;

use std::sync::Once;

static G_INIT_ONCE: Once = Once::new();

pub fn init() {
    G_INIT_ONCE.call_once(|| {
        drivers::init_logging_with_level(log::LevelFilter::Trace);
    });
}

#[derive(Debug)]
enum TestData<'a> {
    Identical {
        name: &'a str,
        data: String,
    },
    Different {
        name: &'a str,
        data: String,
        expected: String,
    },
}

fn get_test_data<'a>() -> Vec<TestData<'a>> {
    vec![
        TestData::Identical {
            name: "frontmatter-000",
            data: r#"
                    @ ---
                    @ status: todo
                    @ ---
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
        },
        TestData::Identical {
            name: "frontmatter-001",
            data: r#"
                    @ ---
                    @ parent: "[[000 Implement the Event Accumulator]]"
                    @ spawned_by: "[[000 Implement the Event Accumulator]]"
                    @ context_type: entry
                    @ ---
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
        },
        TestData::Identical {
            name: "table-000",
            data: r#"
                    @ | Simple | Table | Example |
                    @ | ------ | ----- | ------- |
                    @ | 0      | 1     | 2       |
                    @ | A      | B     | C       |
                    @ | `D`    | `E`   | `F`     |
                    @ | ⊕      | ☆     | ◯       |
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
        },
        TestData::Identical {
            name: "table-001",
            data: r#"
                    @ | Heading        | Meaning                                                                                                                                                                                                                                                                                                                                                                                     |
                    @ | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
                    @ | Objective      | Scope and purpose of the entire note file                                                                                                                                                                                                                                                                                                                                                   |
                    @ | Journal        | Mostly sequential logs of progress towards our objective                                                                                                                                                                                                                                                                                                                                    |
                    @ | Tasks          | Entries with a clear scope and measure of completion. Think "do X". They're operative.                                                                                                                                                                                                                                                                                                      |
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
        },
        TestData::Identical {
            name: "table-002",
            data: r#"
                    @ | Week | Day | Date           | 💫                                  | Highlight                                                                |
                    @ | ---- | --- | -------------- | ----------------------------------- | ------------------------------------------------------------------------ |
                    @ | 36   | Mon | 2025-09-01     | [[Summary-2025-09-01\|🕸️ </>]]     | Learn about wasmer                                                       |
                    @ | 36   | Tue | [[2025-09-02]] | [[Summary-2025-09-02\|🕸️ </>]]<br> | Deploy obsidian notes to web                                             |
                    @ | 36   | Wed | 2025-09-03     | ` `                                 |                                                                          |
                    @ | 36   | Thu | [[2025-09-04]] | 🕹️ `</>`                           | Writing and research                                                     |
                    @ | 36   | Fri | [[2025-09-05]] | <br>🕹️ `</>`                       | Learn about rust db ORM [diesel](https://docs.rs/diesel/latest/diesel/). |
                    @ | 36   | Sat | [[2025-09-06]] | [[Summary-2025-09-06\|📚📈🎲]]      | Study probability theory                                                 |
                    @ | 36   | Sun | 2025-09-07     | ` `                                 |                                                                          |
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
        },
        TestData::Different {
            name: "list-000",
            data: r#"
                    @ There is some text here.
                    @
                    @ *   **Some Text**: Some Description
                    @ *   More Text
                    @
                    @ Something new
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
            expected: r#"
                    @ There is some text here.
                    @
                    @ - **Some Text**: Some Description
                    @ - More Text
                    @
                    @ Something new
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
        },

        // Inner bullets are set to be 3 spaces with pulldown_cmark_to_cmark
        TestData::Identical {
            name: "list-001",
            data: r#"
                    @ 1. Some Item
                    @    - And its description
                    @ 2. Another Item
                    @    - And another description
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
        },

        // Nested bullets are expected to visually align after the "{n}. " of their parent numbered bullets
        // This has expected rules in CommonMarkdown spec. See [list-items](https://spec.commonmark.org/0.31.2/#list-items).
        TestData::Identical {
            name: "list-002",
            data: r#"
                    @ 1. Item
                    @    - Desc
                    @ 2. Item
                    @ 3. Item
                    @ 4. Item
                    @ 5. Item
                    @ 6. Item
                    @ 7. Item
                    @ 8. Item
                    @ 9. Item
                    @ 10. Item
                    @     - Desc
                    @ 11. Item
                    @ 12. Item
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
        },

        TestData::Identical {
            name: "quote-000",
            data: r#"
                    @ > Quotes should be preserved!
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
        },

        // Note that pulldown_cmark does not give an event for the [^l2] line likely because it's unused.
        TestData::Different {
            name: "refs-000",
            data: r#"
                    @ # 1 Refs [^l1]
                    @ [^l1]: https://www.google.com
                    @ [^l2]: https://www.duckduckgo.com
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
            expected: r#"
                    @ # 1 Refs [^l1]
                    @ [^l1]: https://www.google.com
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
        },

        // If the [^l1] line doesn't have a link, it will still be given as an event
        TestData::Identical {
            name: "refs-001",
            data: r#"
                    @ # 1 Refs
                    @ [^l1]: Some Text
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
        },

        // Unnecessary spacing at the end is trimmed, but lines should not collapse. We ran into an issue where
        // the middle new line would be removed.
        TestData::Different {
            name: "spacing-000",
            data: r#"
                    @ Spawned by: [[Some Note]]
                    @
                    @ Spawned in: [[Some Note#^spawn-entry-000000|^spawn-entry-000000]]
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
            expected: r#"
                    @ Spawned by: [[Some Note]]
                    @
                    @ Spawned in: [[Some Note#^spawn-entry-000000|^spawn-entry-000000]]
                "#
            .trim()
            .replace("@ ", "")
            .replace("                    ", ""),
        },
    ]
}

#[test]
fn test_obsidian_patch_writeback() {
    init();

    let tds = get_test_data();

    for td in tds {
        let manual_test_case_filtering_enabled = false;
        let manual_test_case = "";

        if manual_test_case_filtering_enabled {
            match td {
                TestData::Identical { name, .. } | TestData::Different { name, .. } => {
                    if name != manual_test_case {
                        continue;
                    }
                }
            };
        }

        match td {
            TestData::Identical { name, data } => {
                let events = common::parse_markdown_file(&data);

                let new_data = common::render_events_to_common_markdown(&events)
                    .expect("Failed to render back to common markdown")
                    .pipe(|new_data| {
                        common::adhoc_fix_rendered_markdown_output_for_obsidian(&data, &new_data)
                    });

                if data != new_data {
                    println!("failed with test data: {name}");

                    println!("\n<events>");
                    for event in events.iter() {
                        println!("{event:?}");
                    }
                    println!("</events>\n");

                    println!("\n<old>");
                    println!("{data}");
                    println!("</old>\n");

                    println!("\n<new>");
                    println!("{new_data}");
                    println!("</new>\n");

                    drivers::display_diff(&data, &new_data, drivers::DisplayDiffFrom::default());

                    panic!("data does not match");
                }
            }
            TestData::Different {
                name,
                data,
                expected,
            } => {
                let events = common::parse_markdown_file(&data);

                let new_data = common::render_events_to_common_markdown(&events)
                    .expect("Failed to render back to common markdown")
                    .pipe(|new_data| {
                        common::adhoc_fix_rendered_markdown_output_for_obsidian(&data, &new_data)
                    });

                if expected != new_data {
                    println!("failed with test data: {name}");

                    println!("\n<events>");
                    for event in events.iter() {
                        println!("{event:?}");
                    }
                    println!("</events>\n");

                    println!("\n<old>");
                    println!("{data}");
                    println!("</old>\n");

                    println!("\n<expected>");
                    println!("{expected}");
                    println!("</expected>\n");

                    println!("\n<new>");
                    println!("{new_data}");
                    println!("</new>\n");

                    drivers::display_diff(&data, &new_data, drivers::DisplayDiffFrom::default());

                    panic!("data does not match");
                }
            }
        }
    }
}
