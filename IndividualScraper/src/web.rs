use std::io::Read;
use std::{thread, time};

use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use hyper::Ok;

use kuchiki;
use kuchiki::traits::TendrilSink;

fn fetch_url(url: &str) -> impl Read {
	//we're going to be nice to the maintainers of the donation tracker
	//to avoid DDOSing them, add a delay between each request
	let delay_milliseconds = 100;
	thread::sleep(time::Duration::from_millis(delay_milliseconds));

	//perform the request and read the repsonse into a string
	let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    let response = client.get(url).send().unwrap();
    assert_eq!(response.status, Ok);

    response
}

pub fn fetch_html(url: &str) -> kuchiki::NodeRef {
	let mut response = fetch_url(url);
	
	kuchiki::parse_html()
		.from_utf8()
		.read_from(&mut response)
		.unwrap()
}

pub fn fetch_string(url: &str) -> String {
	let mut response = fetch_url(url);

	let mut response_string = String::new();
    response.read_to_string(&mut response_string).unwrap();

    response_string
}