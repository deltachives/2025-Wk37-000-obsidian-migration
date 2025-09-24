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
    Identical { name: &'a str, data: String },
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
    ]
}

#[test]
fn test_obsidian_patch_writeback() {
    init();

    let tds = get_test_data();

    for td in tds {
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
        }
    }
}
