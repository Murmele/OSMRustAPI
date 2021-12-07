//! A simple library to fetch osm data with the overpass api
//! Inspired by python overpass api
//!
//!
//! # Example
//!
//! ```no_run
//!  let text: String;
//!  let query = "node[\"sport\"=\"free_flying\"];";
//!  let api = overpass::API::new("https://lz4.overpass-api.de/api/interpreter", 200);
//!  let result = api.get(query, overpass::Responseformat::xml,  "body", true);
//!  match result {
//!      Err(e) => println!("Something went wrong: {}", e),
//!      Ok(t) => {
//!          export_free_flying_to_file(&t, free_flying_cache_file);
//!          text = t;
//!      }
//!  }
//!  println!("Result: {}", text);
//! ```
use std::time;

pub struct API {
    url: String,
    timeout: u32,
}

macro_rules! QUERY_TEMPLATE { () => { "[out:{out}];{query}out {verbosity};" }; }

impl API {

    // Instances can be found at: https://wiki.openstreetmap.org/wiki/Overpass_API#Public_Overpass_API_instances
    pub fn new(url: &str, timeout: u32) -> Self {
        Self {
            url: url.to_string(),
            timeout
        }
    }

    pub fn get(&self, query: &str, responseformat: Responseformat, verbosity: &str, pure_query: bool) -> Result<String, Box<dyn std::error::Error>>{
        let full_query = if pure_query {
            API::construct_ql_query(query, responseformat, verbosity)
        } else {
            query.to_string()
        };

        println!("Full query: {}", full_query);

        let client = reqwest::blocking::Client::new();
        let mut request_builder = client.post(&self.url).timeout(time::Duration::from_secs(self.timeout.into()));
        request_builder = request_builder.body(full_query);
        let response = request_builder.send()?;

        if response.status().is_success() {
            println!("success!");
        } else if response.status().is_server_error() {
            println!("server error!");
        } else {
            println!("Something else happened. Status: {:?}", response.status());
        }

        Ok(response.text()?)
    }

    fn construct_ql_query(query: &str, responseformat: Responseformat, verbosity: &str) -> String {
        let mut ql_query = query.to_string().trim().to_string();
         if !ql_query.ends_with(";") {
             ql_query = ql_query + ";";
         }

        format!(QUERY_TEMPLATE!(), out=API::response_format_to_string(responseformat), query=ql_query, verbosity=verbosity)
    }

    fn response_format_to_string (f: Responseformat) -> String {
        match f {
            Responseformat::GEOJSON => "geojson".to_string(),
            Responseformat::JSON => "json".to_string(),
            Responseformat::XML => "xml".to_string(),
            Responseformat::CSV => "csv".to_string()
        }
    }
}

pub enum Responseformat {
    GEOJSON,
    JSON,
    XML,
    CSV
}