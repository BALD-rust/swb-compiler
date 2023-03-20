#![feature(type_alias_impl_trait)]
#![feature(async_closure)]

use std::future::poll_fn;
use std::task::Poll;
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embedded_graphics::geometry::{Point, Size};
use env_logger::Env;
use toekomst::display::disp;
use toekomst::key::Accel;
use toekomst::label::{label_once, label_once_bold, label_once_on};
use toekomst::notify::Notify;
use toekomst::{label, request_redraw};
use toekomst::widget::clean_space_on;

use flat_html::Element;
use less_html::strip::ElementIter;
use less_html::Document;
use toekomst::layout::Vertical;
use swb_compiler::CompilationOutput;
use swb_compiler::instruction::{Instruction, StyleVar};

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

struct StyleVarStack {
    state: u32,
}

impl StyleVarStack {
    pub fn new() -> Self {
        Self {
            state: 0,
        }
    }
    
    pub fn push(&mut self) {
        self.state += 1;
    }

    pub fn pop(&mut self) {
        self.state -= 1;
    }

    pub fn is_enabled(&self) -> bool {
        self.state > 0
    }
}

async fn ui(page: &CompilationOutput) {
    let mut v = Vertical::new(Point::new(10, 10), 2);
    let mut bold = StyleVarStack::new();
    for instr in &page.instructions {
        match instr {
            Instruction::Text(address) => {
                // TODO: add helper for this
                let str = page.text_buf.get(address.base.0 as usize..(address.base.offset(address.range as i32).0 as usize)).unwrap();
                if bold.is_enabled() {
                    label_once_bold(str, v.push(label::FONT.character_size)).await;
                } else {
                    label_once(str, v.push(label::FONT.character_size)).await;
                }
            }
            Instruction::Push(StyleVar::Bold) => {
                bold.push();
            }
            Instruction::Push(_) => {}
            Instruction::Pop(StyleVar::Bold) => {
                bold.pop();
            }
            Instruction::Pop(_) => {}
            Instruction::Endl => {
                v.push(label::FONT.character_size);
            }
            Instruction::Stop => {
                break;
            }
        }
    }

    {
        let dt = &mut *disp().await;
        request_redraw();
    }

    poll_fn(|_| Poll::Pending).await
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let env = Env::default().filter_or("RUST_LOG", "info");

    env_logger::Builder::from_env(env)
        .format_timestamp_millis()
        .init();

    toekomst::display::init_disp(Size::new(400, 240));

    let input = include_str!("../../examples/example.html");
    let doc = less_html::Document::from_string(input.to_string()).unwrap();
    let html = less_html::parse(&doc).unwrap();
    let stripped = less_html::strip::oracle_strip(html, &strip).unwrap();
    let swb = swb_compiler::compile(&stripped).unwrap();

    select(toekomst::display::run_disp(), ui(&swb)).await;

    std::process::exit(0);
}
