use std::path::Path;
use flat_html::{Element, FlatHtml};
use swb_compiler::compile;

use anyhow::Result;
use less_html::Document;
use less_html::strip::ElementIter;

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
            .collect()
        );
    }


    // Leave IgnoreTag alone
    less_html::keep_unit_element!(IgnoreTag, next);
    // Leave text alone
    less_html::keep_element!(Text, next);
    // Leave EndTag alone
    less_html::keep_element!(EndTag, next);
    // Leave regular tags alone
    less_html::keep_element!(Tag, next);

    match next {
        Element::LineBreak => {
            // Collapse all subsequent linebreaks into one.
            while let Some(Element::LineBreak) = it.peek() { let _ = it.next(); }
            return Some(vec![Element::LineBreak]);
        }
        // All other cases were already handled
        _ => { unreachable!() }
    };
}

fn strip_page(input: &Path) -> Result<FlatHtml> {
    let doc = Document::from_file(input)?;
    let html = less_html::parse(&doc)?;
    let stripped = less_html::strip::oracle_strip(html, &strip)?;
    Ok(stripped)
}

fn main() -> Result<()> {
    let input = strip_page(Path::new("examples/example.html"))?;
    let output = compile(&input)?;
    print!("{}", output);
    Ok(())
}