use crate::PatentReferenceType;
use chrono::NaiveDate;
use serde::Deserialize;
use serde::de::Deserializer;

use crate::PatentDetails;

#[derive(Debug, Deserialize)]
pub struct UsageValue {
    pub value: String,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize)]
pub struct UsageMetric {
    pub name: String,
    pub values: Vec<UsageValue>,
}

#[derive(Debug, Deserialize)]
pub struct UsageDimension {
    pub name: String,
    pub metrics: Vec<UsageMetric>,
    #[serde(rename(deserialize = "individualNames"))]
    pub individual_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsageEnvironment {
    pub name: String,
    pub dimensions: Vec<UsageDimension>,
}

#[derive(Debug, Deserialize)]
pub struct UsageMetadata {
    pub notices: Option<Vec<String>>,
    pub errors: Option<Vec<String>>,
    #[serde(rename(deserialize = "failedEnvs"))]
    pub failed_envs: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub environments: Vec<UsageEnvironment>,
    #[serde(rename(deserialize = "metaData"))]
    pub metadata: UsageMetadata,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TokenResponse {
    pub access_token: String,
    pub api_product_list: String,
    pub api_product_list_json: Vec<String>,
    pub application_name: String,
    pub client_id: String,
    #[serde(rename(deserialize = "developer.email"))]
    pub developer_email: String,
    pub expires_in: String,
    pub issued_at: String,
    pub organization_name: String,
    pub refresh_count: String,
    pub refresh_token_expires_in: String,
    pub scope: String,
    pub status: String,
    pub token_type: String,
}

fn parse_application_number<'de, D>(deserializer: D) -> Result<PatentDetails, D::Error>
where
    D: Deserializer<'de>,
{
    parse_doc_id(deserializer, PatentReferenceType::Application)
}

fn parse_doc_id<'de, D>(
    deserializer: D,
    refernce_type: PatentReferenceType,
) -> Result<PatentDetails, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    pub struct RegDocumentId {
        #[serde(rename(deserialize = "reg:country"))]
        pub reg_country: DollarValue,
        #[serde(rename(deserialize = "reg:doc-number"))]
        pub reg_doc_number: DollarValue,
        #[serde(rename(deserialize = "reg:date"))]
        pub reg_date: Option<DollarValue>,
    }
    #[derive(Debug, Deserialize)]
    pub struct DollarValue {
        #[serde(rename(deserialize = "$"))]
        pub value: String,
    }

    let reg_doc_id = RegDocumentId::deserialize(deserializer)?;

    let date = reg_doc_id
        .reg_date
        .map(|d| NaiveDate::parse_from_str(&d.value, "%Y%m%d").unwrap());
    Ok(PatentDetails {
        country_code: reg_doc_id.reg_country.value.clone(),
        number: reg_doc_id.reg_doc_number.value.clone(),
        date,
        kind_code: None,
        reference_type: refernce_type,
    })
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RegApplicationReferenceOneOrMany {
    One(RegApplicationReference),
    Many(Vec<RegApplicationReference>),
}

#[derive(Debug, Deserialize)]
pub struct RegApplicationReference {
    #[serde(
        rename(deserialize = "reg:document-id"),
        deserialize_with = "parse_application_number"
    )]
    pub reg_document_id: PatentDetails,
}

/* Register Search structures
 *  These are for deserialising the results from a register search - section 3.4.2 of the EPO OPS doc
 * TODO: Consider what structures overlap 100% with the other JSON structures (e.g. is reg:bibliographic-data
 *       from section 3.4.2 the same as one from section 3.4.1?)
 */

#[derive(Debug, Deserialize)]
pub struct RegSearchRegBibliographicData {
    #[serde(rename(deserialize = "reg:application-reference"))]
    pub reg_application_reference: RegApplicationReference,
}

#[derive(Debug, Deserialize)]
pub struct RegSearchRegRegisterDocument {
    #[serde(rename(deserialize = "reg:bibliographic-data"))]
    pub reg_bibliographic_data: RegSearchRegBibliographicData,
}

#[derive(Debug, Deserialize)]
pub struct RegSearchRegRegisterDocuments {
    // Feels weird to me that the things is called register-document when it's an array of documents...
    //  oh well. I'm going to keep it close to the EPO OPS value names
    #[serde(rename(deserialize = "reg:register-document"))]
    pub reg_register_document: Vec<RegSearchRegRegisterDocument>,
}

#[derive(Debug, Deserialize)]
pub struct RegSearchOpsRegisterSearch {
    #[serde(rename(deserialize = "reg:register-documents"))]
    pub reg_register_documents: RegSearchRegRegisterDocuments,
}

#[derive(Debug, Deserialize)]
pub struct RegSearchOpsWorldPatentData {
    #[serde(rename(deserialize = "ops:register-search"))]
    pub ops_register_search: RegSearchOpsRegisterSearch,
}

#[derive(Debug, Deserialize)]
pub struct RegSearchOpsSearchResults {
    // NOTE: This is not complete - I'm just deserializing values I care about for the moment
    #[serde(rename(deserialize = "ops:world-patent-data"))]
    pub ops_world_patent_data: RegSearchOpsWorldPatentData,
}

/* Register details structs
 * These are structures for deserializing the JSON from register retreival: Section 3.4.1 of the EPO OPS doc
 */

#[derive(Debug, Deserialize)]
pub struct RegRegBibliographicData {
    #[serde(rename(deserialize = "reg:application-reference"))]
    pub reg_application_reference: RegApplicationReferenceOneOrMany,
}

#[derive(Debug, Deserialize)]
pub struct RegRegRegisterDocument {
    #[serde(rename(deserialize = "reg:bibliographic-data"))]
    pub reg_bibliographic_data: RegRegBibliographicData,
}

#[derive(Debug, Deserialize)]
pub struct RegRegRegisterDocuments {
    #[serde(rename(deserialize = "reg:register-document"))]
    pub reg_register_document: RegRegRegisterDocument,
}

#[derive(Debug, Deserialize)]
pub struct RegOpsRegisterSearch {
    #[serde(rename(deserialize = "reg:register-documents"))]
    pub reg_register_documents: RegRegRegisterDocuments,
}

#[derive(Debug, Deserialize)]
pub struct RegOpsWorldPatentData {
    #[serde(rename(deserialize = "ops:register-search"))]
    pub ops_register_search: RegOpsRegisterSearch,
}

#[derive(Debug, Deserialize)]
pub struct RegOpsRegisterResult {
    // Note: This is not a deserialization of the whole structure - only values I care about for the moment
    #[serde(rename(deserialize = "ops:world-patent-data"))]
    pub ops_world_patent_data: RegOpsWorldPatentData,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_register_deser() {
        let path = "test/example_register.json";
        let file = File::open(path).expect("Couldn't open file");
        let reader = BufReader::new(file);
        let json: Result<RegOpsRegisterResult, serde_json::Error> = serde_json::from_reader(reader);
        assert!(json.is_ok());
    }

    #[test]
    fn test_search_page() {
        let path = "test/example_page_from_register_search.json";
        let file = File::open(path).expect("Couldn't open file");
        let reader = BufReader::new(file);
        let json: Result<RegSearchOpsSearchResults, serde_json::Error> =
            serde_json::from_reader(reader);
        assert!(json.is_ok());
    }
}
