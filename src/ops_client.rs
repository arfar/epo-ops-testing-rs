use crate::{EpoOpsCredentials, get_epo_credentials};
use crate::{TokenResponse, Usage};
use chrono::naive::NaiveDate;
use chrono::prelude::*;
use log::{debug, error};
use reqwest::StatusCode;
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::fmt;
use std::sync::Mutex;

static TOKEN: Mutex<TokenMutex> = Mutex::new(new_token_mutex());

enum NeedsAuth {
    Yes,
    No,
}

fn do_get_request(url: &str, mut headers: HeaderMap, needs_auth: NeedsAuth) -> String {
    match needs_auth {
        NeedsAuth::Yes => {
            let epo_credentials = get_epo_credentials();
            let auth_token = get_auth_token(epo_credentials);
            headers.insert(
                "Authorization",
                format!("Bearer {}", auth_token).parse().unwrap(),
            );
        }
        NeedsAuth::No => {}
    }
    let http_client = reqwest::blocking::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let response = http_client.get(url).headers(headers).send();
    handle_request_errors(response)
}

fn do_post_request(
    url: &str,
    data: Option<String>,
    mut headers: HeaderMap,
    needs_auth: NeedsAuth,
) -> String {
    match needs_auth {
        NeedsAuth::Yes => {
            let epo_credentials = get_epo_credentials();
            let auth_token = get_auth_token(epo_credentials);
            headers.insert(
                "Authorization",
                format!("Bearer {}", auth_token).parse().unwrap(),
            );
        }
        NeedsAuth::No => {}
    }
    let http_client = reqwest::blocking::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let mut response = http_client.post(url).headers(headers);
    if let Some(d) = data {
        response = response.body(d);
    }
    let response = response.send();
    handle_request_errors(response)
}

fn handle_request_errors(response: Result<reqwest::blocking::Response, reqwest::Error>) -> String {
    // Rather unhappy with alllll of this. I need to learn how to better reason about
    //  and work with errors etc - even just to make it look better.
    //  And obviously make these not panics if/when I implement good errors
    match response {
        Ok(resp) => match resp.status() {
            StatusCode::OK => resp
                .text()
                .expect("Error decoding body - very odd - weird charset? missing body?"),
            _ => {
                dbg!(&resp);
                dbg!(resp.text().unwrap());
                panic!("Received error in response - can't go forward");
            }
        },
        Err(e) => {
            dbg!(e);
            todo!("Error sending packet. Internet issues?");
        }
    }
}

#[derive(Debug)]
struct TokenMutex {
    access_token: String, // Would like to use a type alias here - not sure how though
    issued_at: i64,
    expires_in: u32,
}

const fn new_token_mutex() -> TokenMutex {
    TokenMutex {
        access_token: String::new(),
        issued_at: 0,
        expires_in: 0,
    }
}

pub fn get_auth_token(epo_credentials: EpoOpsCredentials) -> String {
    let mut token_mutex = TOKEN.lock().unwrap();
    let current_time = Utc::now().timestamp();
    let current_token_expired =
        (current_time - 60) > (token_mutex.issued_at + token_mutex.issued_at);

    if current_token_expired {
        debug!(target: "authentication", "Need new authentication token");
        let formatted_credentials = epo_credentials.format_credentials();

        let auth_url = "https://ops.epo.org/3.2/auth/accesstoken";
        let data = Some("grant_type=client_credentials".to_string());
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Basic {}", formatted_credentials).parse().unwrap(),
        );
        headers.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );

        let response_body = do_post_request(auth_url, data, headers, NeedsAuth::No);

        let response_body_json: Result<TokenResponse, serde_json::Error> =
            serde_json::from_str(&response_body);
        let json_token = match response_body_json {
            Ok(json_token) => json_token,
            Err(e) => {
                dbg!(e);
                dbg!(response_body);
                panic!("Error decoding the JSON - the data layout might've changed?");
            }
        };
        token_mutex.access_token = json_token.access_token.to_string();
        token_mutex.expires_in = json_token.expires_in.parse().unwrap();
        token_mutex.issued_at = json_token.issued_at.parse().unwrap();
        token_mutex.access_token.clone()
    } else {
        debug!(target: "authentication", "Used previous authentication token");
        token_mutex.access_token.clone()
    }
}

pub fn get_publication_bulk(
    patents_details: &Vec<PatentDetails>,
    constituents: &Vec<PublicationConstituents>,
) -> String {
    assert!(patents_details.len() > 1);
    // All of the PatentDetails need to be the same - I could figure out how to convert between the 2
    //  optionally using the number-service API in 3.3 of the EPO OPS doc
    let first_patent = patents_details.first().unwrap();
    #[cfg(debug_assertions)]
    {
        for patent_details in patents_details {
            assert_eq!(patent_details.reference_type, first_patent.reference_type);
        }
    }

    let mut url = match first_patent.reference_type {
        PatentReferenceType::Publication => {
            "http://ops.epo.org/rest-services/published-data/publication/epodoc/".to_string()
        }
        PatentReferenceType::Application => {
            "http://ops.epo.org/rest-services/published-data/application/epodoc/".to_string()
        }
        _ => unimplemented!("Only supporting publication and application numbers"),
    };
    if constituents.is_empty() {
        url.push_str(&PublicationConstituents::Biblio.to_string());
    } else {
        for constituent in constituents {
            url.push_str(&constituent.to_string());
            url.push(',');
        }
        url.pop();
    }
    let mut data = String::new();
    for patent_details in patents_details {
        data.push_str(&patent_details.to_string());
        data.push('\n');
    }
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/plain".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());
    do_post_request(&url, Some(data), headers, NeedsAuth::Yes)
}

pub fn get_publication(
    patent_details: &PatentDetails,
    constituents: &Vec<PublicationConstituents>,
) -> String {
    // This function is 90% the same as the get_register... refactor?
    let mut url = match patent_details.reference_type {
        PatentReferenceType::Publication => {
            "http://ops.epo.org/rest-services/published-data/publication/epodoc/".to_string()
        }
        PatentReferenceType::Application => {
            "http://ops.epo.org/rest-services/published-data/application/epodoc/".to_string()
        }
        _ => unimplemented!("Only supporting publication and application numbers"),
    };
    if constituents.is_empty() {
        url.push_str(&PublicationConstituents::Biblio.to_string());
    } else {
        for constituent in constituents {
            url.push_str(&constituent.to_string());
            url.push(',');
        }
        url.pop();
    }
    let data = Some(patent_details.to_string());
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/plain".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());
    do_post_request(&url, data, headers, NeedsAuth::Yes)
}
// See input construction rule 3 of EPO OPS 3.2 doc
fn encode_input(unquoted_string: &str) -> String {
    let mut quoted_string = String::new();
    // I'm sure there's a more efficient way to do this - but I think it'll work.
    //  The strings will only be like 20 chars max, so it'll probably be fine.
    for c in unquoted_string.chars() {
        match c {
            '?' => quoted_string.push_str("%3F"),
            '@' => quoted_string.push_str("%40"),
            '#' => quoted_string.push_str("%23"),
            '%' => quoted_string.push_str("%25"),
            '$' => quoted_string.push_str("%24"),
            '&' => quoted_string.push_str("%26"),
            '+' => quoted_string.push_str("%2B"),
            ',' => quoted_string.push_str("%2C"),
            ':' => quoted_string.push_str("%3A"),
            ';' => quoted_string.push_str("%3B"),
            '=' => quoted_string.push_str("%3D"),
            ' ' => quoted_string.push_str("%20"),
            '"' => quoted_string.push_str("%22"),
            '<' => quoted_string.push_str("%3C"),
            '>' => quoted_string.push_str("%3E"),
            '{' => quoted_string.push_str("%7B"),
            '}' => quoted_string.push_str("%7D"),
            '|' => quoted_string.push_str("%7C"),
            '^' => quoted_string.push_str("%5E"),
            '~' => quoted_string.push_str("%7E"),
            '[' => quoted_string.push_str("%5B"),
            ']' => quoted_string.push_str("%5D"),
            '`' => quoted_string.push_str("%60"),
            _ => quoted_string.push(c),
        }
    }
    quoted_string
}

pub fn search_register(query_string: &str) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    let url = "http://ops.epo.org/rest-services/register/search";
    let mut headers = HeaderMap::new();
    headers.insert("Range", "1-100".parse().unwrap());
    headers.insert("Content-Type", "text/plain".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());
    let mut data = "q=".to_string();
    data.push_str(&encode_input(query_string));
    let data = data;
    debug!(target: "register", "Running search with query: {}", data);
    // Should probably correct the lifetimes stuff with the query_string
    let result = do_post_request(url, Some(data.clone()), headers, NeedsAuth::Yes);
    let result_json: Result<Value, serde_json::Error> = serde_json::from_str(&result);
    let num_results: u32;
    match result_json {
        Err(e) => {
            error!(target: "register", "Couldn't decode JSON from EPO OPS");
            dbg!(e);
            panic!("Couldn't decode JSON from EPO OPS");
        }
        Ok(json) => match json.get("ops:world-patent-data") {
            Some(ops_world_patent_data) => match ops_world_patent_data.get("ops:register-search") {
                Some(ops_register_search) => {
                    let result = ops_register_search
                        .get("@total-result-count")
                        .unwrap()
                        .as_str()
                        .unwrap();
                    num_results = result.parse().unwrap();
                }
                None => {
                    error!(target: "register", "Couldn't find ops:register-search - malformed JSON");
                    panic!("Couldn't decode JSON from EPO OPS");
                }
            },
            None => {
                error!(
                    target: "register",
                    "Couldn't find ops:world-patent-data - malformed JSON"
                );
                panic!("Couldn't decode JSON from EPO OPS");
            }
        },
    }
    results.push(result);
    debug!(target: "register", "Found {} results", num_results);
    let results_left = num_results - 100;
    let remainder = num_results % 100;
    let num_loops = results_left / 100 + { if remainder > 0 { 1 } else { 0 } };
    for loop_number in 0..num_loops {
        let range_str = format!(
            "{}-{}",
            (loop_number + 1) * 100 + 1,
            (loop_number + 2) * 100
        );
        debug!(target: "register", "Getting items {} of the current search", range_str);
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "text/plain".parse().unwrap());
        headers.insert("Accept", "application/json".parse().unwrap());
        headers.insert("Range", range_str.parse().unwrap());
        results.push(do_post_request(
            url,
            Some(data.clone()),
            headers,
            NeedsAuth::Yes,
        ));
    }

    results
}

pub fn get_register_info(
    patent_details: &PatentDetails,
    constituents: &Vec<RegisterConstituents>,
) -> String {
    let mut url = match patent_details.reference_type {
        PatentReferenceType::Publication => {
            "https://ops.epo.org/rest-services/register/publication/epodoc/".to_string()
        }
        PatentReferenceType::Application => {
            "https://ops.epo.org/rest-services/register/application/epodoc/".to_string()
        }
        _ => unimplemented!("Only supporting publication and application numbers"),
    };

    if constituents.is_empty() {
        url.push_str(&RegisterConstituents::Biblio.to_string());
    } else {
        for constituent in constituents {
            url.push_str(&constituent.to_string());
            url.push(',');
        }
        url.pop();
    }
    let data = Some(patent_details.to_string());
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/plain".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());
    do_post_request(&url, data, headers, NeedsAuth::Yes)
}

pub fn get_usage_data(from: NaiveDate, to: NaiveDate) -> Usage {
    let url = format!(
        "https://ops.epo.org/3.2/developers/me/stats/usage?timeRange={}~{}",
        from.format("%d/%m/%Y"),
        to.format("%d/%m/%Y")
    );
    let mut headers = HeaderMap::new();
    headers.insert("Accept", "application/json".parse().unwrap());
    let usage = do_get_request(&url, headers, NeedsAuth::Yes);
    let response_body_json: Result<Usage, serde_json::Error> = serde_json::from_str(&usage);
    match response_body_json {
        Ok(good_usage) => good_usage,
        Err(e) => {
            dbg!(e);
            dbg!(usage);
            panic!("Error decoding the JSON - the data layout might've changed?");
        }
    }
}

pub enum RegisterConstituents {
    Biblio,
    ProceduralSteps,
    Events,
    Upp,
}

impl fmt::Display for RegisterConstituents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RegisterConstituents::Biblio => write!(f, "biblio"),
            RegisterConstituents::Events => write!(f, "events"),
            RegisterConstituents::ProceduralSteps => write!(f, "procedural-steps"),
            RegisterConstituents::Upp => write!(f, "upp"),
        }
    }
}

pub enum PublicationConstituents {
    Biblio,
    Abstract,
    FullCycle,
}

impl fmt::Display for PublicationConstituents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PublicationConstituents::Biblio => write!(f, "biblio"),
            PublicationConstituents::Abstract => write!(f, "abstract"),
            PublicationConstituents::FullCycle => write!(f, "full-cycle"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum PatentReferenceType {
    Publication,
    Application,
    Priority,
    Unknown,
}

#[derive(Debug)]
pub struct PatentDetails {
    pub country_code: String,
    pub number: String,
    pub kind_code: Option<String>,
    pub date: Option<NaiveDate>,
    pub reference_type: PatentReferenceType,
}

/* There is this "docdb" format which requires '.'s between the different
 * components. I'm not quite sure how/where it's used right now (the EPO
 * docdb website bulk downloadload themselves don't even seem to use
 * it. I'm not going to implement it right now. This doesn't handle
 * escaping.
 */

impl fmt::Display for PatentDetails {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.country_code, self.number)?;
        if let Some(pub_type) = &self.kind_code {
            write!(f, "{}", pub_type)?;
        }
        if let Some(date) = &self.date {
            write!(f, ".{}", date.format("%Y%m%d"))?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_some_quoting() {
        assert_eq!("pa%3DIBM", encode_input("pa=IBM"));
        assert_eq!("applicant%3DIBM", encode_input("applicant=IBM"));

        // This example is from page 15 of the EPO OPS docs - I think their example is
        //  wrong because they don't encode the last space. I could be missing something
        //  though. Also, I'm not doing the appropriate "original" here - this is about
        //  the escape.
        assert_eq!(
            "DE20%202007%20016%20308.8",
            encode_input("DE20 2007 016 308.8")
        );
    }
}
