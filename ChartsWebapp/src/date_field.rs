use chrono::{DateTime, UTC};
use chrono::format::ParseError;
use rocket::request::FromFormValue;

pub struct DateField(pub DateTime<UTC>);

/// Expects ISO formatted date
impl<'v> FromFormValue<'v> for DateField {
	type Error = ParseError;

	fn from_form_value(form_value: &'v str) -> Result<DateField, ParseError> {
		let decoded = String::from_form_value(form_value).unwrap();
		match decoded.parse::<DateTime<UTC>>() {
			Ok(date) => Ok(DateField(date)),
			Err(error) => {println!("{}", error); Err(error)}
		}
	}
}