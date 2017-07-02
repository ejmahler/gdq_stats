extern crate chrono;
extern crate rusqlite;

use std::collections::HashMap;

mod db;

fn format_dollars(amount: i64) -> String
{
	let cents = amount%100;
	let dollars = amount/100;

	format!("${}.{}", dollars, cents)
}



fn main() {
	let donation_db = db::EventDonations::new("sgdq2016");

	let count = donation_db.count() as usize;
    let donations_by_amount = donation_db.by_amount();

    println!("Median");
    println!("upper 75% {}", format_dollars(donations_by_amount[count*3/4].amount));
    println!("median:   {}", format_dollars(donations_by_amount[count/2].amount));
    println!("lower 25% {}", format_dollars(donations_by_amount[count/4].amount));

    println!();
    println!("Mode:");
    let mode_raw = donation_db.amount_mode();
    for (i, &(amount, count)) in mode_raw.iter().enumerate() {
    	println!("{}: {} (Count: {})", i, format_dollars(amount), count);
    }
    
    let interval = 10000000;
    for i in 1..13 {
    	let threshold = i * interval;
    	let lower = threshold - interval / 10;
    	let upper = threshold + interval / 10;

    	let lower_result = donation_db.donation_crossing_threshold(lower);
    	let mid_result =   donation_db.donation_crossing_threshold(threshold);
    	let upper_result = donation_db.donation_crossing_threshold(upper);

    	println!("Crossing threshold {}: lower to mid: {:?}, mid to upper {:?}", format_dollars(threshold), mid_result.timestamp - lower_result.timestamp, upper_result.timestamp - mid_result.timestamp);
    }
    
}
