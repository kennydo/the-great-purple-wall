extern crate futures;
extern crate gotham;
extern crate html5ever;
extern crate hyper;
extern crate hyper_tls;
extern crate kuchiki;
extern crate tokio_core;
extern crate regex;

use std::io;
use std::default::Default;
use futures::{Future, Stream};
use html5ever::serialize::serialize;

use kuchiki::traits::*;
use kuchiki::NodeRef;
use regex::{ Captures, Regex};
mod colors;

fn create_colors_regex() -> Regex {
    let mut color_groupings = Vec::new();
    for color_name in colors::CSS4_COLORS.iter() {
        color_groupings.push(format!("(\\b{}\\b)", color_name));
    }

    let joined_groups = &color_groupings.join("|")[..];

    Regex::new(&[
        "(?P<color>",
        joined_groups,
        ")"
    ].concat()[..]).unwrap()
}

fn replace_color_names_in_text_child_nodes(node_ref: &NodeRef) {
    let colors_regex = create_colors_regex();
    for child_node in node_ref.children() {
        child_node.as_text().and_then(|text_node| {
            let original_text = text_node.borrow().clone();

            Some(text_node.replace(
                colors_regex.replace(&original_text[..], |caps: &Captures| {
                    format!("lol {} asd", &caps[1])
                }).to_string()
            ))
        });
    }
}

fn main() {


    let mut core = tokio_core::reactor::Core::new().unwrap();
    const DNS_WORKER_THREADS: usize = 4;

    let client = hyper::Client::configure()
        .connector(hyper_tls::HttpsConnector::new(DNS_WORKER_THREADS, &core.handle()).unwrap())
        .build(&core.handle());

    let uri = "https://en.wikipedia.org/wiki/Teal".parse().expect("asd");

    let work = client
        .get(uri)
        .and_then(|res| {
            println!("Response: {}", res.status());
            println!("Headers: \n{}", res.headers());

            res.body().concat2().and_then(|chunk| {
                let v = chunk.to_vec();
                // Google does not actually return UTF-8, so instead of carefully parsing the
                // HTTP header to figure out the encoding, we just drop stuff
                Ok(String::from_utf8_lossy(&v).to_string())
            })
        })
        .and_then(|html| {
            let document = kuchiki::parse_html().one(html);
            Ok(document)
        });

    let parsed_dom = core.run(work).unwrap();

    for css_match in parsed_dom.select("li, b, a, p, td").unwrap() {
        let as_node = css_match.as_node();

        replace_color_names_in_text_child_nodes(as_node);
    }


    serialize(&mut io::stdout(), &parsed_dom, Default::default()).unwrap();
}
