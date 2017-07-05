#![feature(plugin)]
#![plugin(rocket_codegen)]

#![feature(custom_derive)]

extern crate chrono;
extern crate rocket;
extern crate postgres;
extern crate serde_json;

extern crate rocket_contrib;

#[macro_use] extern crate serde_derive;

mod webapp_config;
mod date_field;

use std::env;
use std::path::{Path, PathBuf};

use postgres::{Connection, TlsMode};
use chrono::{DateTime, UTC};

use rocket_contrib::JSON;
use rocket::response::{content, NamedFile};

#[macro_use]
extern crate lazy_static;

lazy_static!{
	static ref DATABASE_URI: String = env::var("GDQ_DATABASE_URI").unwrap();
	static ref CURRENT_EVENT_ID: String = env::var("GDQ_LIVE_EVENT_ID").unwrap();
}

static DONATAION_DATA_QUERY: &'static str = "SELECT id, timestamp, donation_count, donation_total, historic_total FROM DonationEntry WHERE event_id = $1 ORDER BY timestamp ASC";
static DONATAION_DATA_UPDATE_QUERY: &'static str = "SELECT id, timestamp, donation_count, donation_total, historic_total FROM DonationEntry WHERE event_id = $1 AND timestamp > $2 ORDER BY timestamp ASC";




#[derive(Serialize)]
struct DonationEntry {
	timestamp: DateTime<UTC>,
	count: i32,
	total: i32,
	total_2016: i32,
}

#[derive(Serialize)]
struct DataResponse(Vec<DonationEntry>);

#[derive(FromForm)]
struct DonationQuery {
	since: date_field::DateField,
}

#[get("/")]
fn index() -> content::HTML<String> {
	let response_str = format!(include_str!("index.html"), static_base=webapp_config::get_static_base());

	content::HTML(response_str)
}

#[get("/donation_data")]
fn get_donation_data() -> JSON<DataResponse>  {
	let database_uri: &str = &*DATABASE_URI;
	let db_connection = Connection::connect(database_uri, TlsMode::None).unwrap();

	let query_result = db_connection.query(DONATAION_DATA_QUERY, &[&*CURRENT_EVENT_ID]).unwrap();

	let result: Vec<DonationEntry> = query_result.iter().map(|row| DonationEntry { timestamp: row.get(1), count: row.get(2), total: row.get(3), total_2016: row.get(4) }).collect();
	JSON(DataResponse(result))
}

#[get("/donation_data/update?<update_form>")]
fn get_donation_data_update(update_form: DonationQuery) -> JSON<DataResponse>  {
	let database_uri: &str = &*DATABASE_URI;
	let db_connection = Connection::connect(database_uri, TlsMode::None).unwrap();

	let date_field::DateField(since_date) = update_form.since;
	let query_result = db_connection.query(DONATAION_DATA_UPDATE_QUERY, &[&*CURRENT_EVENT_ID, &since_date]).unwrap();

	let result: Vec<DonationEntry> = query_result.iter().map(|row| DonationEntry { timestamp: row.get(1), count: row.get(2), total: row.get(3), total_2016: row.get(4) }).collect();
	JSON(DataResponse(result))
}

#[get("/static/<file..>")]
fn static_files(file: PathBuf) -> Option<NamedFile> {
	NamedFile::open(Path::new("static/").join(file)).ok()
}


fn main() {
	if webapp_config::use_local_static_handler() {
		rocket::ignite().mount("/", routes![index, get_donation_data, get_donation_data_update, static_files]).launch()
	} else {
		rocket::ignite().mount("/", routes![index, get_donation_data, get_donation_data_update]).launch()
	}
}