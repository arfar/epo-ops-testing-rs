use chrono::NaiveDate;
use epo_ops_testing::{
    PatentDetails, PatentReferenceType, PublicationConstituents, RegApplicationReferenceOneOrMany,
    RegOpsRegisterResult, RegSearchOpsSearchResults, RegisterConstituents, get_cache_folder,
    get_publication, get_publication_bulk, get_register_info, get_usage_data, load_config,
    search_register,
};
use glob::glob;
use std::fs;
use std::io::BufReader;
use std::path::Path;

fn main() {
    env_logger::init();
    load_config("conf.ini");
    /*
    let info = get_usage_data(
        NaiveDate::from_ymd_opt(2025, 10, 20).unwrap(),
        NaiveDate::from_ymd_opt(2025, 11, 4).unwrap(),
    );
    println!("{:#?}", info);
     */
    //search_ibm();
    //get_all_ibm_register_info();
    get_all_ibm_ep_filing_dates();
}

#[allow(dead_code)]
fn search_ibm() {
    let cache_folder = get_cache_folder();
    let query_string = "pa=ibm";
    let infos = search_register(query_string);
    println!("Searched register for ibm");
    for (i, info) in infos.iter().enumerate() {
        let filename = format!(
            "{}/ibm_space/pa.ibm.{}.register.search.json",
            cache_folder, i
        );
        fs::write(&filename, info)
            .unwrap_or_else(|_| panic!("Should be able to write to {}", filename));
    }
}

#[allow(dead_code)]
fn get_all_ibm_ep_filing_dates() {
    let cache_folder = get_cache_folder();
    let ibm_reg_search_result_files = glob(&format!("{}/ibm_space/*.register.json", cache_folder))
        .expect("Failed to read glob pattern");
    let mut i = 0;
    for ibm_reg_search_result_file in ibm_reg_search_result_files {
        i += 1;
        match ibm_reg_search_result_file {
            Err(e) => println!("{:?}", e),
            Ok(path) => {
                let file = std::fs::File::open(&path).expect("Couldn't open file");
                let reader = BufReader::new(file);
                let json: RegOpsRegisterResult =
                    serde_json::from_reader(reader).expect("JSON was not well formatted");
                let reg_application_references = json
                    .ops_world_patent_data
                    .ops_register_search
                    .reg_register_documents
                    .reg_register_document
                    .reg_bibliographic_data
                    .reg_application_reference;
                let reg_application_reference = match &reg_application_references {
                    RegApplicationReferenceOneOrMany::One(v) => v,
                    RegApplicationReferenceOneOrMany::Many(v) => v.first().unwrap(),
                };
                let doc_id = &reg_application_reference.reg_document_id;

                println!("{} - {:?}", i, doc_id);
            }
        }
    }
}

#[allow(dead_code)]
fn get_all_ibm_register_info() {
    println!("Hello");
    let cache_folder = get_cache_folder();
    let ibm_reg_search_result_files = glob(&format!(
        "{}/ibm_space/pa.ibm.*.register.search.json",
        cache_folder
    ))
    .expect("Failed to read glob pattern");
    for ibm_reg_search_result_file in ibm_reg_search_result_files {
        println!("{:?}", &ibm_reg_search_result_file);
        match ibm_reg_search_result_file {
            Err(e) => println!("{:?}", e),
            Ok(path) => {
                let file = std::fs::File::open(path).expect("Couldn't open file");
                let reader = BufReader::new(file);
                let json: RegSearchOpsSearchResults =
                    serde_json::from_reader(reader).expect("JSON was not well formatted");
                for register_document in json
                    .ops_world_patent_data
                    .ops_register_search
                    .reg_register_documents
                    .reg_register_document
                {
                    let doc_id = register_document
                        .reg_bibliographic_data
                        .reg_application_reference
                        .reg_document_id;
                    let pat_details = PatentDetails {
                        reference_type: PatentReferenceType::Application,
                        ..doc_id
                    };
                    let filename =
                        format!("{}/ibm_space/{}.register.json", cache_folder, pat_details);
                    if Path::new(&filename).exists() {
                        continue;
                    }
                    let bulk_info =
                        get_register_info(&pat_details, &vec![RegisterConstituents::Biblio]);
                    fs::write(&filename, bulk_info)
                        .unwrap_or_else(|_| panic!("Should be able to write to {}", filename));
                }
            }
        }
    }
}

#[allow(dead_code)]
fn example_getting_patent_details() {
    let cache_folder = get_cache_folder();

    let constituents = vec![
        RegisterConstituents::Biblio,
        RegisterConstituents::ProceduralSteps,
        RegisterConstituents::Events,
    ];

    let first_patent_details = PatentDetails {
        country_code: "EP".to_string(),
        number: "4451611".to_string(),
        kind_code: None,
        date: None,
        reference_type: PatentReferenceType::Publication,
    };
    let info = get_register_info(&first_patent_details, &constituents);
    let filename = format!(
        "{}/{}.register.biblio.json",
        cache_folder, first_patent_details
    );
    fs::write(&filename, info)
        .unwrap_or_else(|_| panic!("Should be able to write to {}", filename));

    let second_patent_details = PatentDetails {
        country_code: "EP".to_string(),
        number: "24198419.1".to_string(),
        kind_code: None,
        date: None,
        reference_type: PatentReferenceType::Application,
    };
    let info = get_register_info(&second_patent_details, &constituents);
    let filename = format!(
        "{}/{}.register.biblio.json",
        cache_folder, second_patent_details
    );
    println!("Got second patent");
    fs::write(&filename, info)
        .unwrap_or_else(|_| panic!("Should be able to write to {}", filename));

    let constituents = vec![
        RegisterConstituents::Upp,
        RegisterConstituents::Biblio,
        RegisterConstituents::ProceduralSteps,
        RegisterConstituents::Events,
    ];

    let third_patent_details = PatentDetails {
        country_code: "EP".to_string(),
        number: "21176905".to_string(),
        kind_code: None,
        date: None,
        reference_type: PatentReferenceType::Application,
    };
    let info = get_register_info(&third_patent_details, &constituents);
    let filename = format!(
        "{}/{}.register.upp.json",
        cache_folder, third_patent_details
    );
    println!("Got third patent");
    fs::write(&filename, info)
        .unwrap_or_else(|_| panic!("Should be able to write to {}", filename));

    let info = get_usage_data(
        NaiveDate::from_ymd_opt(2025, 10, 10).unwrap(),
        NaiveDate::from_ymd_opt(2025, 10, 20).unwrap(),
    );
    println!("{:#?}", info);

    let query_string = "pa=nchain";
    let infos = search_register(query_string);
    println!("Searched register for nchain");
    for (i, info) in infos.iter().enumerate() {
        let filename = format!("{}/pa.nchain.{}.register.search.json", cache_folder, i);
        fs::write(&filename, info)
            .unwrap_or_else(|_| panic!("Should be able to write to {}", filename));
    }

    let constituents = vec![
        PublicationConstituents::Biblio,
        PublicationConstituents::Abstract,
        PublicationConstituents::FullCycle,
    ];
    let info = get_publication(&third_patent_details, &constituents);
    let filename = format!(
        "{}/{}.publication.all.json",
        cache_folder, third_patent_details
    );
    println!("Got third patent again");
    fs::write(&filename, info)
        .unwrap_or_else(|_| panic!("Should be able to write to {}", filename));

    let all_patent_details = vec![
        //first_patent_details,
        second_patent_details,
        third_patent_details,
    ];
    let info = get_publication_bulk(&all_patent_details, &constituents);
    println!("Got two patents");
    let filename = format!("{}/multiple.publication.all.json", cache_folder);
    fs::write(&filename, info)
        .unwrap_or_else(|_| panic!("Should be able to write to {}", filename));
}
