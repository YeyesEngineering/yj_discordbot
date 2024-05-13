use poise::{serenity_prelude as serenity, PrefixFrameworkOptions};
use reqwest::Client as HttpClient;
use serenity::{
    // async_trait,
    client::{Client, Context as SerenityContext, EventHandler},
    model::channel::Message,
    prelude::{GatewayIntents, TypeMapKey},
};
use songbird::input::AuxMetadata;
use songbird::SerenityInit;
use std::collections::VecDeque;
use std::env;
use std::sync::{Arc, Mutex};

struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

struct SongQueue;

impl TypeMapKey for SongQueue {
    type Value = Arc<Mutex<VecDeque<AuxMetadata>>>;
}

// struct Handler;

mod discord;

// #[async_trait]
// impl EventHandler for Handler {
//     async fn message(&self, ctx: SerenityContext, msg: Message) {
//     }

// }

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILDS;

    // TEST INTENTS/////////////////////////////////////////////////
    // let intents = GatewayIntents::privileged()
    // | GatewayIntents::all();
    // println!("intents check = {}", intents.is_all());
    // println!("intents check = {}", intents.is_privileged());
    ////////////////////////////////////////////////////////////////

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                discord::age(),
                discord::lucky_time(),
                discord::voice_join(),
                discord::voice_deaf(),
                discord::voice_undeaf(),
                discord::voice_out(),
                discord::voice_mute(),
                discord::voice_unmute(),
                discord::queue(),
                discord::pause(),
                discord::resume(),
                discord::skip(),
                discord::sudo(),
                discord::ping_pong(),
                discord::hello(),
                discord::silence_user(),
                discord::unsilence_user(),
                discord::spamming(),
                discord::play_list(),
                discord::next_playing(),
                discord::repeat_song(),
                discord::repeat_song_disable(),
                // discord::hidemember_find(),
                // discord::hidemember_find2(),
            ],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("!".to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        // .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<SongQueue>(Arc::new(Mutex::new(VecDeque::new())));
    }
    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
