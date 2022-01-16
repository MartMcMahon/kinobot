use flate2::read::GzDecoder;
use std::io::prelude::*;
use tokio::fs;

mod types;

#[tokio::main]
async fn main() {
    // let f_lines = fs::read_to_string("./title.basics.tsv").await.unwrap();
    // let mut d = GzDecoder::new("...".as_bytes());
    // let mut s = String::new();
    // d.read_to_string(&mut s).unwrap();
    // println!("{}", s);

    // load movie database
    // let mut imdb_movies: Vec<types::TitleEntry> = Vec::new();
    // let f_lines = fs::read_to_string("./title.basics.tsv").await.unwrap();
    // let lines: Vec<&str> = f_lines.split('\n').collect();
    // let l = lines.len() as f32;
    // for (i, line) in lines[1..].iter().enumerate() {
    //     println!("{}/{}", i, l);
    //     let cells: Vec<&str> = line.split('\t').collect();
    //     if let Some(i) = types::TitleEntry::from_items(cells) {
    //         imdb_movies.push(i);
    //     }
    // }
    // write_movies_to_json(imdb_movies).await;

    // load names
    let mut imdb_names: Vec<types::NameEntry> = Vec::new();
    let n_lines = fs::read_to_string("./name.basics.tsv").await.unwrap();
    let lines: Vec<&str> = n_lines.split('\n').collect();
    let l = lines.len() as f32;
    for (i, line) in lines[1..].iter().enumerate() {
        println!("{}/{}", i, l);
        let cells: Vec<&str> = line.split('\t').collect();
        for cell in cells {
            println!("{}", cell)
        }
        // if let Some(i) = types::TitleEntry::from_items(cells) {
        //     imdb_movies.push(i);
        // }
    }
}

async fn write_movies_to_json(list: Vec<types::TitleEntry>) {
    fs::write(
        "movies.json",
        serde_json::to_string(&list).expect("error stringifying json"),
    )
    .await
    .unwrap()
}
