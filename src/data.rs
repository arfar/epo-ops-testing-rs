use chrono::naive::NaiveDate;
use std::fmt;

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

pub enum PatentReferenceType {
    Publication,
    Application,
    Priority,
}

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
 * it. I'm not going to implement it right now.
 * There's also something about escaping ... I haven't looked into that
 * either. Might come up when doing US application numbers.
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
