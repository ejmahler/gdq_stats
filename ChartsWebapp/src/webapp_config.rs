use rocket::config;

static DEFAULT_STATIC_BASE: &'static str = "/static";

pub fn get_static_base() -> String {
	config::active().map(|config| {
    	config.get_str("static_base").unwrap_or(DEFAULT_STATIC_BASE).to_string()
	}).unwrap_or(DEFAULT_STATIC_BASE.to_string())
}

pub fn use_local_static_handler() -> bool {
	get_static_base() == DEFAULT_STATIC_BASE
}