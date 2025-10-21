mod ops_client;
pub use crate::ops_client::{
    PatentDetails, PatentReferenceType, PublicationConstituents, RegisterConstituents,
    get_auth_token, get_publication, get_publication_bulk, get_register_info, get_usage_data,
    search_register,
};

mod config;
pub use crate::config::{EpoOpsCredentials, get_cache_folder, get_epo_credentials, load_config};

mod deser;
pub use crate::deser::{
    RegApplicationReferenceOneOrMany, RegOpsRegisterResult, RegSearchOpsSearchResults,
    TokenResponse, Usage,
};
