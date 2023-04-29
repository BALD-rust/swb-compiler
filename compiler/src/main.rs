use flat_html::{Element, FlatHtml, TagKind};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use swb_compiler::compile;

use anyhow::Result;
use less_html::strip::ElementIter;
use less_html::Document;

fn strip(next: &Element, it: &mut ElementIter) -> Option<Vec<Element>> {
    // We split our text on newlines, and delete any lines that are only whitespace
    if let Element::Text(str) = next {
        // Otherwise, we will split our text on newlines
        return Some(
            str.split_terminator('\n')
                .flat_map(|line| {
                    if line.chars().all(|c| c.is_whitespace()) {
                        None
                    } else {
                        // Trim leading and trailing whitespace
                        Some(Element::Text(line.trim().to_string()))
                    }
                })
                .collect(),
        );
    }  

    match next {
        Element::Tag(TagKind::Script) => {
            while let Some(child) = it.next() {
                if let Element::EndTag(TagKind::Script) = *child {
                    return None;
                }
            }
        }
        _ => {}
    };

    // Leave IgnoreTag alone, this is only used while stripping
    less_html::keep_unit_element!(IgnoreTag, next);
    // Leave EndTag alone
    less_html::keep_element!(EndTag, next);
    // Leave regular tags alone
    less_html::keep_element!(Tag, next);

    match next {
        Element::LineBreak => {
            // Collapse all subsequent linebreaks into one.
            while let Some(Element::LineBreak) = it.peek() {
                let _ = it.next();
            }
            return Some(vec![Element::LineBreak]);
        }
        // All other cases were already handled
        _ => {
            unreachable!()
        }
    };
}

fn strip_page(input: &Path) -> Result<FlatHtml> {
    let doc = Document::from_file(input)?;
    let html = less_html::parse(&doc)?;
    let stripped = less_html::strip::oracle_strip(html, &strip)?;
    Ok(stripped)
}

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("usage: swb [input] [--text]");
        std::process::exit(1);
    }
    let path = Path::new(&args[1]);
    let input = strip_page(path)?;
    let output = compile(&input)?;
    let out_path = path.with_extension("swb");
    let mut file = File::create(&out_path)?;

    if args.len() == 3 && args[2] == "--text" {
        write!(&mut file, "{}", output)?;
    } else {
        let binary = output.binary().into_byte_buffer();
        file.write(binary.as_slice())?;
    }
   
    Ok(())
}
