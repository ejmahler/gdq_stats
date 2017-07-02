use rusqlite::{Connection, Transaction, Statement};
use rusqlite::types::ToSql;
use chrono::{DateTime, UTC};

#[derive(Debug)]
pub struct EventDonation<'a>
{
	pub donation_id: u32,
	pub donor_id: Option<u32>,

	pub amount: i64,

	pub total_before: i64,
	pub total_after: i64,

	pub event_id: &'a str,

	pub timestamp: DateTime<UTC>,
	pub comment: Option<String>
}

pub struct EventDonations<'a> {
	db_connection: Connection,
	event_id: &'a str,
}

impl<'a> EventDonations<'a> {
	pub fn new(event_id: &'a str) -> EventDonations<'a> {
		let conn: Connection = Connection::open("donations.sqlite").unwrap();

		EventDonations {
			db_connection: conn,
			event_id: event_id,
		}
	}

	pub fn count(&self) -> u32 {
		self.db_connection.query_row("SELECT count(donation_id) FROM Donation WHERE event_id=?", &[&self.event_id], |row| {
	        row.get(0)
	    }).unwrap()
	}

	pub fn by_timestamp(&self) -> Vec<EventDonation<'a>>
	{
		self.map_query("
			SELECT
				donation_id,
				donor_id,
				amount,
				(total_after - amount) AS total_before,
				total_after,
				timestamp,
				comment
			FROM Donation
			WHERE event_id=?
			ORDER BY timestamp ASC")
	}

	pub fn by_amount(&self) -> Vec<EventDonation<'a>>
	{
		self.map_query("
			SELECT
				donation_id,
				donor_id,
				amount,
				(total_after - amount) AS total_before,
				total_after,
				timestamp,
				comment
			FROM Donation
			WHERE event_id=?
			ORDER BY amount ASC")
	}

	

	pub fn amount_mode(&self) -> Vec<(i64, i64)>
	{
		self.mode_query(10)
	}

	fn mode_query(&self, limit: i64) -> Vec<(i64, i64)> {
		let mut query_stmt = self.db_connection.prepare("
			SELECT
				amount,
				count(amount)
			FROM donation
			WHERE event_id=?1
			GROUP BY amount
			ORDER BY count(amount) DESC
			LIMIT ?2").unwrap();

		let result = {
			query_stmt.query_map(&[&self.event_id, &limit], |row| {
				(row.get(0), row.get(1))
			})
			.unwrap()
			.map(|entry| entry.unwrap())
			.collect()
		};
		result
	}

	pub fn donation_crossing_threshold(&self, threshold: i64) -> EventDonation<'a> {
		let mut result = self.map_query_with_param("
			SELECT
				donation_id,
				donor_id,
				amount,
				(total_after - amount) AS total_before,
				total_after,
				timestamp,
				comment
			FROM Donation
			WHERE event_id=? AND total_after > ?2 AND total_before < ?2
			ORDER BY amount ASC",
			threshold
			);

		result.pop().unwrap()
	}








	fn map_query(&self, query: &str) -> Vec<EventDonation<'a>>
	{
		let mut query_stmt = self.db_connection.prepare(query).unwrap();

		let result = {
			query_stmt.query_map(&[&self.event_id], |row| {
				EventDonation{
					donation_id: row.get(0),
					donor_id: row.get(1),
					amount: row.get(2),
					total_before: row.get(3),
					total_after: row.get(4),
					event_id: self.event_id,
					timestamp: row.get(5),
					comment: row.get(6)
				}

			}).unwrap().map(|entry| entry.unwrap()).collect()
		};
		result
	}


	fn map_query_with_param<T: ToSql>(&self, query: &str, param: T) -> Vec<EventDonation<'a>>
	{
		let mut query_stmt = self.db_connection.prepare(query).unwrap();

		let result = {
			query_stmt.query_map(&[&self.event_id, &param], |row| {
				EventDonation{
					donation_id: row.get(0),
					donor_id: row.get(1),
					amount: row.get(2),
					total_before: row.get(3),
					total_after: row.get(4),
					event_id: self.event_id,
					timestamp: row.get(5),
					comment: row.get(6)
				}

			}).unwrap().map(|entry| entry.unwrap()).collect()
		};
		result
	}
}