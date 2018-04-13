#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]

extern crate futures;
extern crate html5ever;
extern crate hyper;
extern crate hyper_tls;
extern crate kuchiki;
extern crate regex;
extern crate rocket;
extern crate tokio_core;

use futures::future::Executor;
use futures::{future, stream, Future, Stream};
use html5ever::serialize::serialize;
use hyper::Client;
use kuchiki::traits::*;
use kuchiki::NodeRef;
use regex::{Captures, Regex, RegexBuilder};
use std::default::Default;
use std::io;
use tokio_core::reactor::Core;
mod colors;
use rocket::response::content;

use hyper::{Response, StatusCode};

use tokio_core::reactor::Handle;

const DNS_WORKER_THREADS: usize = 4;

fn create_colors_regex() -> Regex {
    let mut color_groupings = Vec::new();
    for color_name in colors::CSS4_COLORS.iter() {
        color_groupings.push(format!("(\\b{}\\b)", color_name));
    }

    let joined_groups = &color_groupings.join("|")[..];

    RegexBuilder::new(&["(?P<color>", joined_groups, ")"].concat()[..])
        .case_insensitive(true)
        .build()
        .unwrap()
}

fn replace_color_names_in_text_child_nodes(node_ref: &NodeRef) {
    let colors_regex = create_colors_regex();
    for child_node in node_ref.children() {
        child_node.as_text().and_then(|text_node| {
            let original_text = text_node.borrow().clone();

            Some(
                text_node.replace(
                    colors_regex
                        .replace_all(&original_text[..], "purple")
                        .to_string(),
                ),
            )
        });
    }
}

fn fetch_and_mutate_url(url: &String) -> String {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());
    let handle = &core.handle();

    let client = hyper::Client::configure()
        .connector(hyper_tls::HttpsConnector::new(DNS_WORKER_THREADS, handle).unwrap())
        .build(handle);

    let uri = url.parse().unwrap();

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
        })
        .and_then(|parsed_dom| {
            for css_match in parsed_dom.select("li, b, a, p, td, div, span, h1, h2, h3, h4").unwrap() {
                let as_node = css_match.as_node();

                replace_color_names_in_text_child_nodes(as_node);
            }

            let mut bytes = vec![];
            serialize(&mut bytes, &parsed_dom, Default::default()).unwrap();
            let result = String::from_utf8(bytes).unwrap();
            Ok(result)
        });
    core.run(work).unwrap()
}

#[derive(FromForm)]
struct Task {
    url: String,
}

#[get("/purplize?<task>")]
fn purplize(task: Task) -> content::Html<String> {
    content::Html(fetch_and_mutate_url(&task.url))
}

fn main() {
    rocket::ignite().mount("/", routes![purplize]).launch();
}
