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
use std::{env, sync::Arc};
use tokio::fs;

const MAIN_DATA_PATH: &str = "./list.json";

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

#[group("commands")]
#[commands(add, list, ping)]
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
    {
        let mut data = client.data.write().await;
        // load initial state from file
        data.insert::<JsonData>(Arc::new(read_data().await));
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
