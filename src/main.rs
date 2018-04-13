extern crate futures;
extern crate gotham;
extern crate html5ever;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;

use std::io::{self, Write};
use std::default::Default;
use futures::{Future, Stream};
use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use futures::future;
use hyper::{Method, Error};

use html5ever::{parse_document, serialize};
use html5ever::driver::ParseOpts;
use html5ever::rcdom::RcDom;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;

fn main() {
    let mut core = Core::new().unwrap();

    let client = hyper::Client::configure()
        .connector(HttpsConnector::new(4, &core.handle()).unwrap())
        .build(&core.handle());

    let opts = ParseOpts {
        ..Default::default()
    };

    let uri = "https://www.google.com/".parse().expect("asd");

    let work = client.get(uri)
          .and_then(|res| {
              println!("Response: {}", res.status());
              println!("Headers: \n{}", res.headers());

              res.body().concat2().and_then(|chunk| {
                  let v = chunk.to_vec();
                  Ok(String::from_utf8_lossy(&v).to_string())
              })
          });

    let foo = core.run(work).expect("couldnt run loop");
    println!("result: {:?}", foo)
}
