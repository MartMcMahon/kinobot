use serde::{Deserialize, Serialize};
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        Args, CommandError, StandardFramework,
    },
    model::{channel::Message, gateway::Ready},
    prelude::TypeMapKey,
    utils::MessageBuilder,
};
use std::{collections::HashMap, env, sync::Arc};
use tokio::fs;

const MAIN_DATA_PATH: &str = "./list.json";

mod types;
use types::*;
mod tests;

struct Db;
impl TypeMapKey for Db {
    type Value = Vec<TitleEntry>;
}

#[derive(Default)]
struct Handler;

#[derive(Debug, Default, Deserialize, Serialize)]
struct JsonData {
    watchlist: Vec<WatchlistMovie>,
}
impl TypeMapKey for JsonData {
    type Value = Arc<JsonData>;
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct WatchlistMovie {
    title: String,
    year: String,
    director: String,
    added_by: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        eprintln!("{} is connected", ready.user.name);
    }

    async fn message(&self, context: Context, msg: Message) {
        let message = msg.content.to_lowercase();
        if message.starts_with("hello") {
            let res = MessageBuilder::new().push("もしもし").build();
            if let Err(why) = msg.channel_id.say(&context.http, &res).await {
                eprintln!("Error sending message: {:?}", why);
            }
        }
    }
}

#[command]
#[description("says pong")]
async fn ping(ctx: &Context, msg: &Message) -> Result<(), CommandError> {
    msg.channel_id.say(&ctx.http, "pong!").await?;
    Ok(())
}

#[command]
#[min_args(1)]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> Result<(), CommandError> {
    let title = title_from_args(args);

    let read_lock = ctx.data.read().await;
    let parsed_json = Arc::clone(read_lock.get::<JsonData>().unwrap());
    drop(read_lock);
    let mut write_lock = ctx.data.write().await;

    println!("adding {}", &title);
    let mut new_list = parsed_json.watchlist.clone();
    new_list.push(WatchlistMovie {
        title,
        year: "".to_string(),
        director: "".to_string(),
        added_by: "".to_string(),
    });

    write_lock.insert::<JsonData>(Arc::new(JsonData {
        watchlist: new_list.clone(),
    }));
    drop(write_lock);

    msg.reply(&ctx.http, "added".to_owned()).await.unwrap();

    write_list_to_disk(JsonData {
        watchlist: new_list,
    })
    .await;

    Ok(())
}

#[command]
async fn list(ctx: &Context, msg: &Message) -> Result<(), CommandError> {
    //     let lock = ctx.data.read().await;
    //     let parsed_json = lock.get::<JsonData>().unwrap();
    //     println!("list: {}", parsed_json.watchlist.join(", "));
    //     msg.channel_id
    //         .say(
    //             &ctx.http,
    //             MessageBuilder::new().push_codeblock(parsed_json.watchlist.join("\n"), None),
    //         )
    //         .await
    //         .expect("error saying");

    Ok(())
}

// async fn crew_from_tconst(ctx: &Context, tconst: String) -> Option<CrewEntry> {
//     let crew_data_str = fs::read_to_string("./crews.json")
//         .await
//         .expect("error reading crews.json");
//     let data: CrewMap = serde_json::from_str(&crew_data_str).expect("error parsing crew json");
//     data.get(&tconst)
// }

#[command]
#[min_args(1)]
async fn lookup(ctx: &Context, msg: &Message, args: Args) -> Result<(), CommandError> {
    let title = title_from_args(args);

    let typing = msg.channel_id.start_typing(&ctx.http).unwrap();

    let imdb_entry = imdb_find_title(ctx, title.clone()).await;
    // let crew_entry = imdb_find_crew(ctx, )

    // match imdb_entry {
    //     Some(entry) => {
    //         let directors = entry.directors.join(", and ");
    //         msg.reply(
    //             &ctx.http,
    //             format!(
    //                 "{} ({}) -- directed by {}",
    //                 entry.original_title, entry.start_year, director
    //             ),
    //         )
    //         .await
    //         .unwrap();
    //     }
    //     None => {
    //         msg.reply(&ctx.http, format!("couldn't find {}", title))
    //             .await
    //             .unwrap();
    //     }
    // }
    // typing.stop();

    Ok(())
}

#[group("commands")]
#[commands(add, list, lookup, ping)]
struct CommandGroup;

#[tokio::main]
async fn main() {
    // set up framework
    let framework = StandardFramework::new()
        .group(&COMMANDGROUP_GROUP)
        .configure(|c| c.prefix("/"));
    let token = env::var("KINOBOT_TOKEN").expect("token");

    // create serenity client
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    let films_json = fs::read_to_string("./movies.json").await.unwrap();
    let imdb_movies: Vec<TitleEntry> =
        serde_json::from_str(films_json.as_str()).expect("error parsing movies.json");
    {
        let mut data = client.data.write().await;
        // load initial state from file
        data.insert::<JsonData>(Arc::new(read_data().await));
        data.insert::<Db>(imdb_movies);
    }

    // start bot
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

async fn read_data() -> JsonData {
    let x = fs::read_to_string(MAIN_DATA_PATH)
        .await
        .expect("error reading file");

    let data: JsonData = serde_json::from_str(x.as_str()).expect("error parsing json");

    let x = data
        .watchlist
        .iter()
        .map(|movie| movie.title.as_str())
        .collect::<String>();

    println!("{}", x);
    data
}

async fn write_list_to_disk(data: JsonData) {
    fs::write(
        MAIN_DATA_PATH,
        serde_json::to_string(&data).expect("error stringifying json"),
    )
    .await
    .unwrap();
}

fn title_from_args(mut args: Args) -> String {
    println!("{}", args.remaining());
    let len = args.len();
    let mut title_words: Vec<String> = Vec::new();
    for arg in 0..len {
        println!("arg {:#?}", arg);
        title_words.push(args.single().unwrap());
    }
    let title = title_words.join(" ").to_lowercase();
    title
}

async fn imdb_find_title(ctx: &Context, title: String) -> Option<TitleEntry> {
    let lock = ctx.data.read().await;
    let imdb_movies = lock.get::<Db>().unwrap();
    let mut res = None;
    for entry in imdb_movies {
        if entry.primary_title.to_lowercase() == title
            || entry.original_title.to_lowercase() == title
        {
            res = Some(entry.clone());
            break;
        }
    }
    drop(lock);
    res
}
