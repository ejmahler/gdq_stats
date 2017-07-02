use regex::Regex;

use chrono::{DateTime, UTC};
use kuchiki::{NodeDataRef, ElementData};

use selectors::tree::Element;
use selectors::Element;

use common::{Donation, IncompleteDonation};
use web;

lazy_static! {
	static ref PAGECOUNT_REGEX: Regex = Regex::new(r#"<option>(?P<pagenum>[\d]+)</option>"#).unwrap();
	static ref DONOR_ID_REGEX: Regex = Regex::new(r#"/tracker/donor/(?P<donor_id>\d+)(?:/\d+)?"#).unwrap();
	static ref DONATION_ID_REGEX: Regex = Regex::new(r#"/tracker/donation/(?P<donation_id>\d+)"#).unwrap();
	static ref DONATION_AMOUNT_REGEX: Regex = Regex::new(r#"Amount:\s*[$](?P<amount_text>[\d,.]+)"#).unwrap();
}




pub fn scrape_donation_list_pagecount(event_id: &str) -> u32 {
	let response_string = web::fetch_string(&format!("https://gamesdonequick.com/tracker/donations/{}", event_id));

    //search through <option> tags for page numbers
    let mut max_pagenum = 0;
    for capture in PAGECOUNT_REGEX.captures_iter(&response_string) {
    	let matched_pagenum = capture.name("pagenum").unwrap().as_str();

    	let parsed_pagenum = matched_pagenum.parse::<u32>().unwrap();

    	if parsed_pagenum > max_pagenum {
    		max_pagenum = parsed_pagenum;
    	}
    }

    assert!(max_pagenum > 0, "The parser couldn't determine how many pages of donors there are");

    max_pagenum
}


pub fn scrape_donation_list(event_id: &str, page_id: u32) -> Vec<IncompleteDonation> {
	let response_dom = web::fetch_html(&format!("https://gamesdonequick.com/tracker/donations/{}?page={}", event_id, page_id));

	let mut parsed_donations = Vec::new();
	for row in response_dom.select("body>div.container-fluid>table>tbody>tr").unwrap() {
		parsed_donations.push(parse_donation_row(row));
	}
	parsed_donations
}

pub fn scrape_donation_page(donation_id: u32) -> Donation {
	let response_dom = web::fetch_html(&format!("https://gamesdonequick.com/tracker/donation/{}", donation_id));

	//most of the data we need can be found in a h2 element
	let donation_header = response_dom.select("div.container-fluid > h2.text-center").unwrap().next().unwrap();

	let donor_id = parse_donor_id(&donation_header);
	let timestamp = parse_timestamp(donation_header.as_node().select(".datetime").unwrap().next().unwrap());

	let amount = parse_donation_amount_from_text(&donation_header.text_contents());

	//if the comment exists, it's the only textcontent in the tbody of the first table on the page
	//if no comment exists, there are no tables on the page
	let comment = if let Some(comment_container) = response_dom.select("table > tbody").unwrap().next() {
		let trimmed_text = comment_container.text_contents().trim().to_owned();

		//if the comment was rejected or is still pending approval, count it as None
		match trimmed_text.as_str() {
			"(Comment rejected)" => None,
			"(Comment pending approval)" => None,
			_ => Some(trimmed_text),
		}
	} else {
		None
	};

	Donation {
		donation_id: donation_id,
		donor_id: donor_id,
		amount: amount,
		timestamp: timestamp,
		comment: comment
	}
}







fn parse_donation_row(row: NodeDataRef<ElementData>) -> IncompleteDonation {
	let mut td_iter = row.as_node().select("td").unwrap();


	//the first td will contain the donor id (or it will contain the text "(anonymous)"")
	let donor_id = parse_donor_id(&td_iter.next().unwrap());

	//the next cell contains the timestamp
	let timestamp = parse_timestamp(td_iter.next().unwrap());

	//the next cell contains both the donation id and donation amount
	let donation_cell = td_iter.next().unwrap();
	let donation_id = parse_donation_id(&donation_cell);
	let donation_amount = parse_donation_amount(&donation_cell);

	//the fourth cell has a yes/no indicating whether or not the donation has a comment
	let comment_text = td_iter.next().unwrap().text_contents();
	let has_comment = match comment_text.trim() {
		"Yes" => true,
		"No" => false,
		other => panic!("Unexpected value in 'has comment field': {}", other)
	};

	//there shouldbe no more rows
	assert!(td_iter.count() == 0, "Got too many columns in a donation row");

	IncompleteDonation {
		donation_id: donation_id,
		donor_id: donor_id, 
		amount: donation_amount,
		timestamp: timestamp, 
		has_comment: has_comment
	}
}


fn parse_donor_id(parent: &NodeDataRef<ElementData>) -> Option<u32> {
	if let Some(child_element) = parent.as_node().select("a").unwrap().next() {
		let borrowed_attributes = child_element.attributes.borrow();
		let href = borrowed_attributes.get("href").unwrap();

		let captures = DONOR_ID_REGEX.captures(href).unwrap();
		let parsed_id = captures.name("donor_id").unwrap().as_str().parse::<u32>().unwrap();
		Some(parsed_id)

	} else {
		None
	}
}

fn parse_timestamp(parent: NodeDataRef<ElementData>) -> DateTime<UTC> {
	DateTime::parse_from_str(parent.text_contents().trim(), "%m/%d/%Y %H:%M:%S %z").unwrap().with_timezone(&UTC)
}

fn parse_donation_id(td: &NodeDataRef<ElementData>) -> u32 {
	let child_element = td.first_child_element().unwrap();
	let borrowed_attributes = child_element.attributes.borrow();
	let href = borrowed_attributes.get("href").unwrap();

	let captures = DONATION_ID_REGEX.captures(href).unwrap();

	captures.name("donation_id").unwrap().as_str().parse::<u32>().unwrap()
}


fn parse_donation_amount(td: &NodeDataRef<ElementData>) -> i64 {
	let stripped_string = td.text_contents().trim().replace("$","").replace(",","").replace(".","");
	stripped_string.parse::<i64>().unwrap()
}

fn parse_donation_amount_from_text(text: &str) -> i64 {
	let captures = DONATION_AMOUNT_REGEX.captures(text).unwrap();

	let captured_amount = captures.name("amount_text").unwrap().as_str();
	let stripped_amount = captured_amount.replace(",","").replace(".","");

	stripped_amount.parse::<i64>().unwrap()
}
