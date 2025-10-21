use chrono::NaiveDate;
use epo_ops_testing::{
    get_cache_folder, get_register_info, get_usage_data, load_config, search_register,
    PatentDetails, PatentReferenceType, RegisterConstituents,
};
use std::fs;

fn main() {
    env_logger::init();

    load_config("conf.ini");
    let cache_folder = get_cache_folder();

    let constituents = vec![
        RegisterConstituents::Biblio,
        RegisterConstituents::ProceduralSteps,
        RegisterConstituents::Events,
    ];

    let patent_details = PatentDetails {
        country_code: "EP".to_string(),
        number: "4451611".to_string(),
        kind_code: None,
        date: None,
        reference_type: PatentReferenceType::Publication,
    };
    let info = get_register_info(&patent_details, &constituents);
    let filename = format!("{}/{}.register.biblio.json", cache_folder, patent_details);
    fs::write(&filename, info)
        .unwrap_or_else(|_| panic!("Should be able to write to {}", filename));

    let patent_details = PatentDetails {
        country_code: "EP".to_string(),
        number: "24198419.1".to_string(),
        kind_code: None,
        date: None,
        reference_type: PatentReferenceType::Application,
    };
    let info = get_register_info(&patent_details, &constituents);
    let filename = format!("{}/{}.register.biblio.json", cache_folder, patent_details);
    println!("Got second patent");
    fs::write(&filename, info)
        .unwrap_or_else(|_| panic!("Should be able to write to {}", filename));

    let constituents = vec![
        RegisterConstituents::Upp,
        RegisterConstituents::Biblio,
        RegisterConstituents::ProceduralSteps,
        RegisterConstituents::Events,
    ];

    let patent_details = PatentDetails {
        country_code: "EP".to_string(),
        number: "21176905".to_string(),
        kind_code: None,
        date: None,
        reference_type: PatentReferenceType::Application,
    };
    let info = get_register_info(&patent_details, &constituents);
    let filename = format!("{}/{}.register.upp.json", cache_folder, patent_details);
    println!("Got third patent");
    fs::write(&filename, info)
        .unwrap_or_else(|_| panic!("Should be able to write to {}", filename));
    /*
    let info = get_usage_data(
        NaiveDate::from_ymd_opt(2025, 10, 10).unwrap(),
        NaiveDate::from_ymd_opt(2025, 10, 20).unwrap(),
    );
    println!("{:#?}", info);
     */
    let query_string = "pa=nchain";
    let infos = search_register(query_string);
    println!("Searched register for nchain");
    for (i, info) in infos.iter().enumerate() {
        let filename = format!("{}/pa.nchain.{}.register.search.json", cache_folder, i);
        fs::write(&filename, info)
            .unwrap_or_else(|_| panic!("Should be able to write to {}", filename));
    }
}
