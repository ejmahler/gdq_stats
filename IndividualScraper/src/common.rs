use chrono::{DateTime, UTC};

pub struct IncompleteDonation
{
	pub donation_id: u32,

	pub donor_id: Option<u32>,

	pub amount: i64,

	pub timestamp: DateTime<UTC>,

	pub has_comment: bool
}


pub struct Donation
{
	pub donation_id: u32,

	pub donor_id: Option<u32>,

	pub amount: i64,

	pub timestamp: DateTime<UTC>,

	pub comment: Option<String>
}

impl Donation {
	pub fn without_comment(donation: &IncompleteDonation) -> Donation {
		Donation {
			donation_id: donation.donation_id,
			donor_id: donation.donor_id,
			amount: donation.amount,
			timestamp: donation.timestamp,
			comment: None
		}
	}
}