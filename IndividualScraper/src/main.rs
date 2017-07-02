#![feature(conservative_impl_trait)]

extern crate rusqlite;
extern crate chrono;
extern crate hyper;
extern crate hyper_native_tls;
extern crate regex;
extern crate kuchiki;
extern crate selectors;
extern crate pbr;

#[macro_use]
extern crate lazy_static;


mod common;
mod scrape;
mod db;
mod web;

use pbr::ProgressBar;
use db::{PageQueue, DonationQueue, DonationList, FinalizedDonationList};

const EVENT_ID: &'static str = "sgdq2016";

fn main() {
	println!("Beginning of main");
    
	let mut page_queue = PageQueue::new(EVENT_ID);
	let mut donation_queue = DonationQueue::new(EVENT_ID);
	let donation_list = DonationList::new(EVENT_ID);
	let mut finalized_list = FinalizedDonationList::new();

	if page_queue.count() == 0 && donation_queue.count() == 0 && donation_list.count() == 0 && finalized_list.count(EVENT_ID) == 0{
		init_pages(&mut page_queue);
	}
	
	if page_queue.count() > 0 {
		process_donation_pages(&mut page_queue);
	}

	if donation_queue.count() > 0 {
		process_donations(&mut donation_queue);
	}

	if donation_list.count() > 0 && finalized_list.count(EVENT_ID) == 0 {
		finalized_list.finalize(&donation_list);
	}

	println!("Done");
}





fn init_pages(page_queue: &mut PageQueue) {
	let pagecount = scrape::scrape_donation_list_pagecount(page_queue.event_id());
	
	let pages: Vec<u32> = (1..(pagecount+1)).collect();

	page_queue.enqueue(&pages);

	println!("Called init and queued {} pages", page_queue.count());
}



fn process_donation_pages(page_queue: &mut PageQueue) {
	let count = page_queue.count();
	let mut progress_bar = ProgressBar::new(count as u64);
	progress_bar.message("Scanning donation list: ");

	for _ in 0..count {
		let page_id = page_queue.peek();

		let donation_list = scrape::scrape_donation_list(page_queue.event_id(), page_id);

		page_queue.dequeue(page_id, &donation_list);
		progress_bar.inc();
	}

	progress_bar.finish();
}

fn process_donations(donation_queue: &mut DonationQueue) {
	let count = donation_queue.count();
	let mut progress_bar = ProgressBar::new(count as u64);
	progress_bar.message("Processing donations: ");

	for _ in 0..count {
		let donation = scrape::scrape_donation_page(donation_queue.peek());
		donation_queue.dequeue(donation);
		progress_bar.inc();
	}

	progress_bar.finish();
}
