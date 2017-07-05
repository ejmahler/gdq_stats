extern crate chrono;
extern crate postgres;
extern crate rusqlite;

use std::env;
use chrono::{DateTime, UTC};

static HISTORIC_EVENT_ID: &'static str = "sgdq2016";
static LIVE_EVENT_ID: &'static str = "sgdq2017";

struct Donation {
	timestamp: DateTime<UTC>,
	total: i32,
}

fn load_historic_donations() {
	let rusqlite_connection = rusqlite::Connection::open("../IndividualScraper/donations.sqlite").unwrap();

	let mut query_stmt = rusqlite_connection.prepare("SELECT timestamp, total_after FROM Donation WHERE event_id=? ORDER BY timestamp DESC").unwrap();


	let historic_donations: Vec<Donation> = {
		query_stmt.query_map(&[&HISTORIC_EVENT_ID], |row| {
			Donation{
				timestamp: row.get(0),
				total: row.get(1)
			}

		}).unwrap().map(|entry| entry.unwrap()).collect()
	};

	let postgres_uri = env::var("GDQ_DATABASE_URI").unwrap();
	let postgres_connection = postgres::Connection::connect(postgres_uri, postgres::TlsMode::None).unwrap();

	let historic_base: DateTime<UTC> = "2016-07-03T15:00:00+00:00".parse().unwrap();
	for item in historic_donations {
		let elapsed = item.timestamp.timestamp() - historic_base.timestamp();

		let count = postgres_connection.query("SELECT id FROM HistoricDonation WHERE event_id = $1 AND seconds_since_beginning = $2", &[&HISTORIC_EVENT_ID, &elapsed]).unwrap().len();

		if count == 0 {
			postgres_connection.execute("INSERT INTO HistoricDonation (donation_total, event_id, seconds_since_beginning) VALUES ($1, $2, $3)",
	                 &[&(item.total / 100), &HISTORIC_EVENT_ID, &elapsed]).unwrap();
		}
	}
}

fn apply_to_live_data() {
	let postgres_uri = env::var("GDQ_DATABASE_URI").unwrap();
	let postgres_connection = postgres::Connection::connect(postgres_uri, postgres::TlsMode::None).unwrap();

	//figure out the start time for live donations
	let query_result = postgres_connection.query("SELECT timestamp FROM DonationEntry WHERE event_id = $1 ORDER BY timestamp ASC LIMIT 1", &[&LIVE_EVENT_ID]).unwrap();
	let result: Vec<DateTime<UTC>> = query_result.iter().map(|row| row.get(0) ).collect();
	let live_base = result[0];

	//iterate through all the live donations
	let query_result = postgres_connection.query("SELECT id, timestamp FROM DonationEntry WHERE event_id = $1 ORDER BY timestamp ASC", &[&LIVE_EVENT_ID]).unwrap();
	for row in query_result.iter() {
		let id: i32 = row.get(0);
		let live_timestamp: DateTime<UTC> = row.get(1);
		let elapsed = live_timestamp.timestamp() - live_base.timestamp();

		let historic_query = postgres_connection.query(
			"SELECT donation_total FROM HistoricDonation WHERE event_id = $1 AND seconds_since_beginning < $2 ORDER BY seconds_since_beginning DESC",
			&[&HISTORIC_EVENT_ID, &elapsed]).unwrap();
		
		let count = historic_query.len() as i32;
		let total: i32 = historic_query.get(0).get(0);

		postgres_connection.execute("UPDATE DonationEntry SET historic_total = $1, historic_count = $2 WHERE id = $3",
	                 &[&total, &count, &id]).unwrap();

	}
}

fn main() {
	load_historic_donations();
	apply_to_live_data();
}


