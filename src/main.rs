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
use std::{env, num::ParseIntError, sync::Arc};
use tokio::fs;

const MAIN_DATA_PATH: &str = "./list.json";

#[derive(Debug, Deserialize)]
struct ImdbEntry {
    tconst: String,
    title_type: String,
    primary_title: String,
    original_title: String,
    is_adult: bool,
    start_year: u32,
    end_year: String,
    runtime_minutes: String,
    genres: String,
}
impl ImdbEntry {
    fn from_items(items: Vec<&str>) -> Option<ImdbEntry> {
        if items.len() == 9 {
            let year = items[5].parse::<u32>();
            Some(ImdbEntry {
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

struct Db;
impl TypeMapKey for Db {
    type Value = Vec<ImdbEntry>;
}

#[derive(Default)]
struct Handler;

#[derive(Debug, Default, Deserialize, Serialize)]
struct JsonData {
    watchlist: Vec<String>,
}
impl TypeMapKey for JsonData {
    type Value = Arc<JsonData>;
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
    let first = args.single::<String>().unwrap();
    msg.reply(&ctx.http, "added".to_owned()).await.unwrap();
    println!("added {}", first);
    let read_lock = ctx.data.read().await;
    let parsed_json = Arc::clone(read_lock.get::<JsonData>().unwrap());
    drop(read_lock);
    let mut write_lock = ctx.data.write().await;

    let mut new_list = parsed_json.watchlist.clone();
    new_list.push(first);

    write_lock.insert::<JsonData>(Arc::new(JsonData {
        watchlist: new_list,
    }));
    drop(write_lock);
    Ok(())
}

#[command]
async fn list(ctx: &Context, msg: &Message) -> Result<(), CommandError> {
    let lock = ctx.data.read().await;
    let parsed_json = lock.get::<JsonData>().unwrap();
    println!("list: {}", parsed_json.watchlist.join(", "));
    msg.channel_id
        .say(
            &ctx.http,
            MessageBuilder::new().push_codeblock(parsed_json.watchlist.join("\n"), None),
        )
        .await
        .expect("error saying");

    Ok(())
}

#[command]
#[min_args(1)]
async fn lookup(ctx: &Context, msg: &Message, mut args: Args) -> Result<(), CommandError> {
    let title = args.single::<String>().unwrap();
    let typing = msg.channel_id.start_typing(&ctx.http).unwrap();

    let lock = ctx.data.read().await;
    let imdb_movies = lock.get::<Db>().unwrap();
    let mut res = None;
    for entry in imdb_movies {
        if entry.primary_title == title || entry.original_title == title {
            res = Some(entry.clone());
            break;
        }
    }
    match res {
        Some(entry) => {
            let director = "some person";
            msg.reply(
                &ctx.http,
                format!(
                    "{} ({}) -- directed by {}",
                    entry.original_title, entry.start_year, director
                ),
            )
            .await
            .unwrap();
        }
        None => {
            msg.reply(&ctx.http, format!("couldn't find {}", title))
                .await
                .unwrap();
        }
    }
    drop(lock);
    typing.stop();

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

    // load movie database
    let mut imdb_movies: Vec<ImdbEntry> = Vec::new();
    let f_lines = fs::read_to_string("./title.basics.tsv").await.unwrap();
    let lines: Vec<&str> = f_lines.split('\n').collect();
    let l = lines.len() as f32;
    let end = (l / 8.).floor() as usize;
    for (i, line) in lines[1..].iter().enumerate() {
        println!("{}/{}", i, l);
        let cells: Vec<&str> = line.split('\t').collect();
        if let Some(i) = ImdbEntry::from_items(cells) {
            imdb_movies.push(i);
        }
    }
    println!("done parsing all {:?} movies", imdb_movies.len());
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

    println!("{}", data.watchlist.join(", "));
    data
}

async fn write_to_file(data: JsonData) {
    fs::write(
        MAIN_DATA_PATH,
        serde_json::to_string(&data).expect("error stringifying json"),
    )
    .await
    .unwrap();
}
