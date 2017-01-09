extern crate postgres;
extern crate chrono;
extern crate hyper;
extern crate regex;

use std::env;
use std::io::Read;

use postgres::{Connection, TlsMode};
use chrono::{DateTime, UTC};
use regex::Regex;

struct DonationEntry {
    id: i32,
    timestamp: DateTime<UTC>,
    donation_count: i32,
    donation_total: i32,
}

fn main() {
	let database_uri: String = env::var("GDQ_DATABASE_URI").unwrap();

    let conn = Connection::connect(database_uri.as_str(), TlsMode::None).unwrap();
    /*conn.execute("DROP TABLE DonationEntry", &[]).unwrap();
    conn.execute("CREATE TABLE DonationEntry (
                    id              SERIAL PRIMARY KEY,
                    timestamp       TIMESTAMP WITH TIME ZONE NOT NULL,
                    donation_count           INT,
                    donation_total           INT
                  )", &[]).unwrap();
    */
    

    let (count, total) = get_current_donation_stats();



    let new_entry = DonationEntry {
    	id: 0,
        timestamp: UTC::now(),
        donation_count: count,
        donation_total: total
    };
    conn.execute("INSERT INTO DonationEntry (timestamp, donation_count, donation_total) VALUES ($1, $2, $3)",
                 &[&new_entry.timestamp, &new_entry.donation_count, &new_entry.donation_total]).unwrap();

    for row in &conn.query("SELECT id, timestamp, donation_count, donation_total FROM DonationEntry", &[]).unwrap() {
        let entry = DonationEntry {
            id: row.get(0),
            timestamp: row.get(1),
            donation_count: row.get(2),
            donation_total: row.get(3),
        };
        println!("id: {}, timestamp: {}, count: {}, total: {}", entry.id, entry.timestamp, entry.donation_count, entry.donation_total);
    }
}

fn get_current_donation_stats() -> (i32, i32) {

    //send the request and read the response
    let client = hyper::Client::new();

    let scrape_url: String = env::var("GDQ_SCRAPE_URL").unwrap();
    //let scrape_url = "https://gamesdonequick.com/tracker/19";

    let mut response = client.get(scrape_url.as_str()).send().unwrap();
    assert_eq!(response.status, hyper::Ok);

    let mut response_string = String::new();
    response.read_to_string(&mut response_string).unwrap();

    //pull out the donation total and donation count text
    let parse_regex = Regex::new(r"Donation Total:\s*\$(?P<total>[\d,]+)(?:[.\d]+)?\s*\((?P<count>\d+)\)\s*\&mdash;").unwrap();
    let captures = parse_regex.captures(&response_string).unwrap();

    let matched_total = captures.name("total").unwrap().as_str();
    let matched_count = captures.name("count").unwrap().as_str();

    //remove any comma separators before parsing
    let parsed_total = matched_total.replace(",", "").parse::<i32>().unwrap();
    let parsed_count = matched_count.replace(",", "").parse::<i32>().unwrap();

	(parsed_count, parsed_total)
}