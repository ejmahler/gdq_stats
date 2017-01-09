#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;
extern crate rocket;
extern crate postgres;

use std::env;

use postgres::{Connection, TlsMode};
use chrono::{DateTime, UTC};

struct DonationEntry {
    id: i32,
    timestamp: DateTime<UTC>,
    donation_count: i32,
    donation_total: i32,
}

#[get("/")]
fn index() -> String {
	let database_uri: String = env::var("GDQ_DATABASE_URI").unwrap();
    let db_connection = Connection::connect(database_uri.as_str(), TlsMode::None).unwrap();

    let query_result = db_connection.query("SELECT id, timestamp, donation_count, donation_total FROM DonationEntry ORDER BY timestamp DESC", &[]).unwrap();

    let mut result = String::new();
    for row in &query_result {
        let entry = DonationEntry {
            id: row.get(0),
            timestamp: row.get(1),
            donation_count: row.get(2),
            donation_total: row.get(3),
        };
        result += format!("<p>id: {}, timestamp: {}, count: {}, total: {}</p>", entry.id, entry.timestamp, entry.donation_count, entry.donation_total).as_str();
    }

    result
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch()
}