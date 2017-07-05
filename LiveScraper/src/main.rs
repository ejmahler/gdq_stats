extern crate postgres;
extern crate chrono;
extern crate regex;
extern crate hyper;
extern crate hyper_native_tls;

use std::env;
use std::str;
use std::io::Read;

use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

use postgres::{Connection, TlsMode};
use chrono::{DateTime, Utc};
use regex::Regex;

struct DonationEntry {
    id: i32,
    timestamp: DateTime<Utc>,
    donation_count: i32,
    donation_total: i32,
    event_id: String,
    historic_count: i32,
    historic_total: i32,
}

fn main() {
	let database_uri: String = env::var("GDQ_DATABASE_URI").unwrap();
    let live_id: String =       env::var("GDQ_LIVE_EVENT_ID").unwrap();

    let (count, total) = get_current_donation_stats(&live_id);

    let conn = Connection::connect(database_uri.as_str(), TlsMode::None).unwrap();
    let current_timestamp: DateTime<Utc> = Utc::now();


    //figure out the start time for live donations
    let query_result = conn.query("SELECT timestamp FROM DonationEntry WHERE event_id = $1 ORDER BY timestamp ASC LIMIT 1", &[&live_id]).unwrap();
    let result: Vec<DateTime<Utc>> = query_result.iter().map(|row| row.get(0) ).collect();
    let live_base = result[0];

    //get the corresponding last year's data
    let elapsed = current_timestamp.timestamp() - live_base.timestamp();

    let historic_query = conn.query(
        "SELECT donation_total FROM HistoricDonation WHERE event_id = $1 AND seconds_since_beginning < $2 ORDER BY seconds_since_beginning DESC",
        &[&"sgdq2016", &elapsed]).unwrap();

    let historic_count = historic_query.len() as i32;
    let historic_total = historic_query.get(0).get(0);


    

    let new_entry = DonationEntry {
    	id: 0,
        timestamp: current_timestamp,
        donation_count: count,
        donation_total: total,
        event_id: live_id,
        historic_count: historic_count,
        historic_total: historic_total
    };
    conn.execute("INSERT INTO DonationEntry (timestamp, donation_count, donation_total, event_id, historic_count, historic_total) VALUES ($1, $2, $3, $4, $5, $6)",
                 &[&new_entry.timestamp, &new_entry.donation_count, &new_entry.donation_total, &new_entry.event_id, &new_entry.historic_count, &new_entry.historic_total]).unwrap();

    let result = conn.query("SELECT id, timestamp, donation_count, donation_total, event_id, historic_count, historic_total FROM DonationEntry ORDER BY timestamp DESC LIMIT 1", &[]).unwrap();
    assert_eq!(result.len(), 1);
    for row in &result {
        let entry = DonationEntry {
            id: row.get(0),
            timestamp: row.get(1),
            donation_count: row.get(2),
            donation_total: row.get(3),
            event_id: row.get(4),
            historic_count: row.get(5),
            historic_total: row.get(6),
        };
        println!("Successfuly added row: id: {}, timestamp: {}, count: {}, total: {}, event_id:{}, historic total: {}", entry.id, entry.timestamp, entry.donation_count, entry.donation_total, entry.event_id, entry.historic_total);
    }
}

fn get_current_donation_stats(event_id: &str) -> (i32, i32) {

    //send the request and read the response
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    let scrape_url = format!("https://gamesdonequick.com/tracker/index/{}", event_id);

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