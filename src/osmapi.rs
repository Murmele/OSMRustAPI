//! A simple library use the osm api
//!
//!
//! # Example
//!
//! ```no_run
//! ```

use reqwest;
use url::Url;
use xmltree::{Element, ParseError, XMLNode, EmitterConfig};
use chrono::{DateTime, Utc};
use std::fs::File;

const API_VERSION: &str = "0.6";
const OSM_DEVEL_API_URL: &str = "https://master.apis.dev.openstreetmap.org";
const OSM_API_URL: &str = "https://api.openstreetmap.org";
const GENERATOR: &str = "Rust OSM API";

pub struct OsmAccount {
    username: String,
    password: String,
    dev_api: bool,

    changeset_id: String,
    changeset: Element,
}

impl OsmAccount {
    pub fn new(username: String, password: String, dev_api: bool) -> Self {
        let mut changeset = Element::new("osmChange");
        changeset.attributes.insert("version".to_string(), API_VERSION.to_string());
        changeset.attributes.insert("generator".to_string(), GENERATOR.to_string());
        let now: DateTime<Utc> = Utc::now();
        changeset.attributes.insert("date".to_string(), now.to_rfc3339());
        changeset.attributes.insert("author".to_string(), "Martin Marmsoler".to_string());  

        Self {
            username,
            password,
            dev_api,
            changeset_id: "".to_string(),
            changeset: changeset,
        }
    }

    pub fn add_modify_node_changeset(&mut self, mut node: Element, version: &str) {
        // Osm Change: https://wiki.openstreetmap.org/wiki/OsmChange
        node.attributes.insert("changeset".to_string(), self.changeset_id.clone());
        node.attributes.insert("version".to_string(), version.to_string());

        match self.changeset.get_mut_child("modify") {
            Some(e) => {
                e.children.push(XMLNode::Element(node));
            },
            None => {
                let mut e = Element::new("modify");
                e.children.push(XMLNode::Element(node));
                self.changeset.children.push(XMLNode::Element(e));
            }
        }
    }

    pub fn add_create_node_changeset(&mut self, mut node: Element) {
        node.attributes.insert("changeset".to_string(), self.changeset_id.clone());
        node.attributes.insert("version".to_string(), "1".to_string());

        match self.changeset.get_mut_child("create") {
            Some(e) => {
                e.children.push(XMLNode::Element(node));
            },
            None => {
                let mut e = Element::new("create");
                e.children.push(XMLNode::Element(node));
                self.changeset.children.push(XMLNode::Element(e));
            }
        }
    }

    pub fn changeset_id(&mut self) -> Result<&String, ()>{
        if self.changeset_id.is_empty() {
            return Err(());
        }
        Ok(&self.changeset_id)
    }


    pub fn write_changeset_to_file(&self, filename: &str) -> Result<(), xmltree::Error> {
        // Write changeset to file for caching!
        let mut config = EmitterConfig::new();
        config.perform_indent = true;
        config.perform_escaping = true;
        self.changeset.write_with_config(File::create(filename).unwrap(), config)
    }

    // Needs always authentication
    // https://help.openstreetmap.org/questions/17694/create-a-changeset-using-api
    pub fn put(&self, sub_url: &str, body: &str) -> Result<String , Box<dyn std::error::Error>> {
        let url = create_url(self.dev_api, sub_url)?;

        let client = reqwest::blocking::Client::new();
        let request_builder = client.put(url);
        let request_builder = request_builder.basic_auth(&self.username, Some(&self.password));
        let request_builder = request_builder.body(body.to_string().clone()); // TODO:remove "to_string().clone()" fix lifetime issue
        let response = request_builder.send()?;
        Ok(response.text()?)
    }

    pub fn put_xml(&self, sub_url: &str, body: &str) -> Result<Element, Box<dyn std::error::Error>> {
        Ok(parse_response(&self.put(sub_url, body)?)?)
    }

    pub fn get(&self, sub_url: &str) -> Result<Element, Box<dyn std::error::Error>> {
        get(self.dev_api, sub_url)
    }

    pub fn createChangeSet(&mut self, comment: &str) -> Result<(), Box<dyn std::error::Error>> {

        // create changeset body according to
        // https://wiki.openstreetmap.org/w/images/6/67/OSM_API0.6_Changeset_successful_creation_V0.1.png
        
        // XMLtree is not able to print to string?? 
        // let mut e = Element::new("osm");
        // let mut changeset = Element::new("changeset");
        // let mut created_by = Element::new("tag");
        // created_by.attributes.insert("created_by".to_string(), "Rust OSM API".to_string());
        // let mut c = Element::new("tag");
        // c.attributes.insert("comment".to_string(), comment.to_string());
        // changeset.children.push(XMLNode::Element(created_by));
        // changeset.children.push(XMLNode::Element(c));
        // e.children.push(XMLNode::Element(changeset));
        let s = format!("<osm>
                            <changeset>
                                <tag k=\"created_by\" v=\"{}\"/>
                                <tag k=\"comment\" v=\"{}\"/>
                            </changeset>
                        </osm>
                        ", "RUST OSM API", comment);

        let res = self.put("changeset/create", &s);
        match res {
            Ok(v) => {
                self.changeset_id = v;
                return Ok(())
            },
            Err(e) => {
                println!("Error occured: {:?}", e);
                return Err(e)
            }
        }
    }
}

fn create_url(devel: bool, sub_url: &str) -> Result<Url, Box<dyn std::error::Error>> {
    let mut url;
    if devel {
        url = Url::parse(OSM_DEVEL_API_URL)?;
    } else {
        url = Url::parse(OSM_API_URL)?;
    }
    url = url.join("api/")?;
    url = url.join(&(API_VERSION.to_owned() + "/"))?;
    return Ok(url.join(sub_url)?);
}

pub fn get_tag(devel: bool, tag: &str) {
    
}

pub fn get(devel: bool, sub_url: &str) -> Result<Element, Box<dyn std::error::Error>> {
    let url = create_url(devel, sub_url)?;

    // https://blog.logrocket.com/making-http-requests-rust-reqwest/
    let resp = reqwest::blocking::get(url.as_str())?;
    match resp.status() {
        reqwest::StatusCode::OK => {
            let element = parse_response(&resp.text()?)?;
            return Ok(element)
        },
        _ => {
            return Err(format!("{}", resp.status().as_u16()).into());
        },
    };
}

fn parse_response(resp: &str) -> Result<Element, ParseError> {
    Element::parse(resp.as_bytes())
}

#[cfg(test)]
mod tests {

    #[test]
    fn account() {
        // let dev = true;
        // let account = super::OsmAccount::new(String::from("Username"), String::from("Password"), dev);

        // assert!(account.username == "Username");
        // assert!(account.password == "Password");
        // assert!(account.api_url == "https://master.apis.dev.openstreetmap.org");

        // let dev = false;
        // let account = super::OsmAccount::new(String::from("Username"), String::from("Password"), dev);

        // assert!(account.username == "Username");
        // assert!(account.password == "Password");
        // assert!(account.api_url == "https://api.openstreetmap.org/");
    }

    #[test]
    fn getNode() {
        // let node_id = "885094266";

        // super::get(false, "node/".to_owned() + node_id);
    }
}