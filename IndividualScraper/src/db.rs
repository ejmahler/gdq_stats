use rusqlite::{Connection, Transaction};

use common::{IncompleteDonation, Donation};

static SETUP_PROGRESS_SQL : &'static str = "
			BEGIN;

			CREATE TABLE IF NOT EXISTS DonationPageQueue( page_id INT PRIMARY KEY NOT NULL, event_id TEXT NOT NULL );
			CREATE TABLE IF NOT EXISTS DonationQueue( donation_id INT PRIMARY KEY NOT NULL, event_id TEXT NOT NULL );
			CREATE TABLE IF NOT EXISTS Donation( 
				donation_id INT PRIMARY KEY NOT NULL,
				event_id TEXT NOT NULL,
				donor_id INT,
				amount BIGINT NOT NULL,
				timestamp TEXT NOT NULL,
				comment TEXT
				);

			COMMIT;
			";

static SETUP_FINAL_SQL : &'static str = "
			BEGIN;

			CREATE TABLE IF NOT EXISTS Donation( 
				donation_id INT PRIMARY KEY NOT NULL,
				event_id TEXT NOT NULL,
				donor_id INT,
				amount BIGINT NOT NULL,
				total_after BIGINT NOT NULL,
				timestamp TEXT NOT NULL,
				comment TEXT
				);

			COMMIT;
			";

fn make_progress_connection() -> Connection {
	Connection::open("donations_in_progress.sqlite").unwrap()
}



fn insert_in_progress_donation(tx: &Transaction, donation: Donation, event_id: &str) {
	tx.execute("INSERT INTO Donation (donation_id, event_id, donor_id, amount, timestamp, comment) VALUES(?1,?2,?3,?4,?5,?6)",
		&[
			&donation.donation_id,
			&event_id,
			&donation.donor_id,
			&donation.amount,
			&donation.timestamp,
			&donation.comment
		]).unwrap();
}



pub struct PageQueue<'a> {
	db_connection: Connection,
	event_id: &'a str,
}

impl<'a> PageQueue<'a> {
	pub fn new(event_id: &'a str) -> PageQueue<'a> {
		let conn: Connection = make_progress_connection();

		//make sure the donor page table exists
		conn.execute_batch(SETUP_PROGRESS_SQL).unwrap();

		PageQueue {
			db_connection: conn,
			event_id: event_id,
		}
	}

	pub fn count(&self) -> u32 {
		self.db_connection.query_row("SELECT count(page_id) FROM DonationPageQueue WHERE event_id=?", &[&self.event_id], |row| {
	        row.get(0)
	    }).unwrap()
	}

	pub fn peek(&self) -> u32 {
		self.db_connection.query_row("SELECT page_id FROM DonationPageQueue WHERE event_id=? LIMIT 1", &[&self.event_id], |row| {
	        row.get(0)
	    }).unwrap()
	}

	pub fn event_id(&self) -> &'a str {
		self.event_id
	}

	pub fn enqueue(&mut self, pages: &[u32])
	{
		let tx = self.db_connection.transaction().unwrap();

		//we need to create a scope so that the insert statement wil lbe dropped before we commit the transaction
		{
			let mut stmt = tx.prepare("INSERT INTO DonationPageQueue (page_id, event_id) VALUES (?,?)").unwrap();

			for page_id in pages {
				stmt.execute(&[page_id, &self.event_id]).unwrap();
			}
		}

	    tx.commit().unwrap();
	}

	pub fn dequeue(&mut self, page_id: u32, donations: &[IncompleteDonation])
	{
		let tx = self.db_connection.transaction().unwrap();

		tx.execute("DELETE FROM DonationPageQueue WHERE page_id=?", &[&page_id]).unwrap();

		//we need to create a scope so that the insert statement wil lbe dropped before we commit the transaction
		{
			let mut insert_queue_stmt = tx.prepare("INSERT INTO DonationQueue (donation_id, event_id) VALUES (?, ?)").unwrap();

			for donation in donations {
				//id this donation has a comment, we need to add it to the donation queue. if it doesn't, we can insert the donation directly
				if donation.has_comment {
					insert_queue_stmt.execute(&[&donation.donation_id, &self.event_id]).unwrap();
				} else {
					insert_in_progress_donation(&tx, Donation::without_comment(donation), self.event_id);
				}
				
			}
		}

	    tx.commit().unwrap();
	}
}



pub struct DonationQueue<'a> {
	db_connection: Connection,
	event_id: &'a str,
}

impl<'a> DonationQueue<'a> {
	pub fn new(event_id: &'a str) -> DonationQueue<'a> {
		let conn: Connection = make_progress_connection();

		//make sure the donor page table exists
		conn.execute_batch(SETUP_PROGRESS_SQL).unwrap();

		DonationQueue {
			db_connection: conn,
			event_id: event_id,
		}
	}

	pub fn count(&self) -> u32 {
		self.db_connection.query_row("SELECT count(donation_id) FROM DonationQueue WHERE event_id=?", &[&self.event_id], |row| {
	        row.get(0)
	    }).unwrap()
	}

	pub fn peek(&self) -> u32 {
		self.db_connection.query_row("SELECT donation_id FROM DonationQueue WHERE event_id=? LIMIT 1", &[&self.event_id], |row| {
	        row.get(0)
	    }).unwrap()
	}

	pub fn dequeue(&mut self, donation: Donation)
	{
		let tx = self.db_connection.transaction().unwrap();

		tx.execute("DELETE FROM DonationQueue WHERE donation_id=?", &[&donation.donation_id]).unwrap();
		insert_in_progress_donation(&tx, donation, self.event_id);

	    tx.commit().unwrap();
	}
}

pub struct DonationList<'a> {
	db_connection: Connection,
	event_id: &'a str,
}

impl<'a> DonationList<'a> {
	pub fn new(event_id: &'a str) -> DonationList<'a> {
		let conn: Connection = make_progress_connection();

		//make sure the donor page table exists
		conn.execute_batch(SETUP_PROGRESS_SQL).unwrap();

		DonationList {
			db_connection: conn,
			event_id: event_id,
		}
	}

	pub fn count(&self) -> u32 {
		self.db_connection.query_row("SELECT count(donation_id) FROM Donation WHERE event_id=?", &[&self.event_id], |row| {
	        row.get(0)
	    }).unwrap()
	}

	pub fn event_id(&self) -> &'a str {
		self.event_id
	}


	pub fn all_donations(&self) -> Vec<Donation> {
		let mut query_stmt = self.db_connection.prepare("SELECT donation_id, donor_id, amount, timestamp, comment FROM Donation WHERE event_id=? ORDER BY timestamp ASC").unwrap();

		let result = {
			query_stmt.query_map(&[&self.event_id], |row| {
				Donation{
					donation_id: row.get(0),
					donor_id: row.get(1),
					amount: row.get(2),
					timestamp: row.get(3),
					comment: row.get(4)
				}

			}).unwrap().map(|entry| entry.unwrap()).collect()
		};
		result
	}
}






pub struct FinalizedDonationList {
	db_connection: Connection,
}

impl FinalizedDonationList {
	pub fn new() -> FinalizedDonationList {
		let conn: Connection = Self::make_final_connection();

		//make sure the donor page table exists
		conn.execute_batch(SETUP_FINAL_SQL).unwrap();

		FinalizedDonationList {
			db_connection: conn,
		}
	}

	fn make_final_connection() -> Connection {
		Connection::open("donations.sqlite").unwrap()
	}

	pub fn count(&self, event_id: &str) -> u32 {
		self.db_connection.query_row("SELECT count(donation_id) FROM Donation WHERE event_id=?", &[&event_id], |row| {
	        row.get(0)
	    }).unwrap()
	}

	pub fn finalize(&mut self, donation_list: &DonationList) {
		let tx = self.db_connection.transaction().unwrap();

		let mut total = 0;
		for donation in donation_list.all_donations() {
			total += donation.amount;

			Self::insert_finalized_donation(&tx, donation, total, donation_list.event_id());
		}

	    tx.commit().unwrap();
	}

	fn insert_finalized_donation(tx: &Transaction, donation: Donation, total_after: i64, event_id: &str) {
		tx.execute("INSERT INTO Donation (donation_id, event_id, donor_id, amount, total_after, timestamp, comment) VALUES(?1,?2,?3,?4,?5,?6,?7)",
		&[
			&donation.donation_id,
			&event_id,
			&donation.donor_id,
			&donation.amount,
			&total_after,
			&donation.timestamp,
			&donation.comment
		]).unwrap();
	}
}