mod data;
pub use crate::data::{PatentDetails, PatentReferenceType, RegisterConstituents};

mod ops_client;
pub use crate::ops_client::{get_auth_token, get_register_info, get_usage_data, search_register};

mod config;
pub use crate::config::{get_cache_folder, get_epo_credentials, load_config, EpoOpsCredentials};
