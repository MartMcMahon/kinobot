use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// https://stackoverflow.com/questions/53866508/how-to-make-a-public-struct-where-all-fields-are-public-without-repeating-pub
// macro_rules! pub_struct {
//     ($name:ident {$($field:ident: $t:ty,)*}) => {
//         #[derive(Debug, Clone, PartialEq)] // ewww
//         pub struct $name {
//             $(pub $field: $t),*
//         }
//     }
// }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TitleEntry {
    pub tconst: String,
    pub title_type: String,
    pub primary_title: String,
    pub original_title: String,
    pub is_adult: bool,
    pub start_year: u32,
    pub end_year: String,
    pub runtime_minutes: String,
    pub genres: String,
}
impl TitleEntry {
    pub fn from_items(items: Vec<&str>) -> Option<TitleEntry> {
        if items.len() == 9 {
            let year = items[5].parse::<u32>();
            Some(TitleEntry {
                tconst: items[0].to_string(),
                title_type: items[1].to_string(),
                primary_title: items[2].to_string(),
                original_title: items[3].to_string(),
                is_adult: items[4] == "1",
                start_year: match year {
                    Ok(y) => y,
                    _ => 0,
                },
                end_year: items[6].to_string(),
                runtime_minutes: items[7].to_string(),
                genres: items[8].to_string(),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NameEntry {
    pub nconst: String,
    pub primaryName: String,
    pub birthYear: u32,
    pub deathyear: String,
    pub primaryProfession: Vec<String>,
    pub knownForTitles: Vec<String>,
}
// impl NameEntry {
//     pub fn from_items(items: Vec<&str>) -> Option<NameEntry> {
//         if items.len() == 6 {
//             Some(NameEntry {
//                 nconst:
//             })

// } else {
//     None
// }
// }
// }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CrewEntry {
    pub tconst: String,
    pub directors: Vec<String>,
    pub writers: Vec<String>,
}
impl CrewEntry {
    pub fn expand(self) -> Vec<NameEntry> {
        self.directors
            .iter()
            .map(|nconst| {
                // lookup nconst for NameEntry
                NameEntry {
                    nconst: nconst.to_string(),
                    primaryName: "".to_string(),
                    birthYear: 0,
                    deathyear: "".to_string(),
                    primaryProfession: vec!["".to_string()],
                    knownForTitles: vec!["".to_string()],
                }
            })
            .collect()
    }
}

pub struct BotEntry {
    pub tconst: String,
    pub title: String,
    pub year: u32,
    pub runtime: String,
    pub director: String,
}
