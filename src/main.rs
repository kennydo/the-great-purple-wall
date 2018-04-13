extern crate futures;
extern crate gotham;
extern crate html5ever;
extern crate hyper;
extern crate hyper_tls;
extern crate kuchiki;
extern crate tokio_core;

use std::io;
use std::default::Default;
use futures::{Future, Stream};
use html5ever::serialize::serialize;

use kuchiki::traits::*;

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

    let parsed_dom = core.run(work).expect("couldnt run loop");
    serialize(&mut io::stdout(), &parsed_dom, Default::default()).unwrap();
}
