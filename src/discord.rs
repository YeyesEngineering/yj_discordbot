use rand::Rng;

use ::serenity::all::{ChannelId, PermissionOverwrite};

use crate::{Context, SongQueue};
use poise::serenity_prelude as serenity;
use serenity::async_trait;
use serenity::builder::{CreateEmbed, CreateEmbedFooter, CreateMessage};
use serenity::client::Context as SerenityContext;
use serenity::model::channel::PermissionOverwriteType;
use serenity::model::Permissions;
use serenity::model::Timestamp;
use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use songbird::input::{AuxMetadata, Compose, YoutubeDl};
use songbird::tracks::LoopState;

use crate::HttpKey;

// pub const RANDOM: i32 = 1;
type Error = Box<dyn std::error::Error + Send + Sync>;
struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }
        None
    }
}

struct TrackEndNotifier {
    ctx: SerenityContext,
    channel_id: ChannelId,
    // http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_) = ctx {
            let song_data = {
                let queue_list = {
                    let data_read = self.ctx.data.read().await;
                    data_read
                        .get::<SongQueue>()
                        .expect("Expected StrictMod in TypeMap.")
                        .clone()
                };
                let mut queue_list_set = queue_list.lock().unwrap();
                let song_data = queue_list_set.pop_front().unwrap();
                song_data
            };
            let builder = song_information(song_data).await;
            let _ = self.channel_id.send_message(&self.ctx.http, builder).await;
        }
        None
    }
}

// struct TrackStartNotifier;

// #[async_trait]
// impl VoiceEventHandler for TrackStartNotifier {
//     async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
//         if let EventContext::Track(track_list) = ctx {
//         }
//         None
//     }
// }

//Command

#[poise::command(slash_command, prefix_command)]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn ping_pong(ctx: Context<'_>) -> Result<(), Error> {
    let channel = ctx.guild_channel().await.unwrap();
    let response = serenity::utils::MessageBuilder::new()
        .push("User ")
        .push_bold_safe(&ctx.author().name)
        .push(" used the 'ping' command in the ")
        .mention(&channel)
        .push(" channel")
        .build();

    ctx.say(&response).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "지옥버튼")]
pub async fn lucky_time(ctx: Context<'_>) -> Result<(), Error> {
    let time = Timestamp::now();
    ctx.say(lucky(time)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "인사")]
pub async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    let user = ctx.author();

    let user_nickname = user_nickname(&user.name);

    let footer = CreateEmbedFooter::new("나를 상징하는 이미지");
    let embed = CreateEmbed::new()
        .title((user_nickname).to_string() + " 등장!")
        .description("반갑게 맞이해주세요")
        .image(&user.avatar_url().unwrap_or(String::new()))
        .footer(footer)
        .timestamp(Timestamp::now());

    let builder = CreateMessage::new().embed(embed);
    let _ = ctx.say("HELLO").await?;
    let msg = ctx
        .channel_id()
        .send_message(ctx.serenity_context(), builder)
        .await;
    if let Err(why) = msg {
        println!("Error sending message: {why:?}");
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "devtest")]
pub async fn sudo(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user_id = user.unwrap().id;

    let allow = Permissions::ADMINISTRATOR;
    let deny = Permissions::empty();
    let per_over = PermissionOverwrite {
        allow,
        deny,
        kind: PermissionOverwriteType::Member(user_id),
        // kind: PermissionOverwriteType::Role(role_id),
    };

    // println!("{:?}", per_over);

    let channel = ctx.guild_channel().await.unwrap();
    let permission_change = channel
        .create_permission(ctx.serenity_context(), per_over)
        .await?;
    println!("permission_change = {:?}", permission_change);

    //Permission Check
    // let guild_id = ctx.guild_id().unwrap();
    // let channel_id = ctx.channel_id();
    let member = ctx.guild_channel().await.unwrap();
    // let member = ctx.cache().member(guild_id, user_id).unwrap();
    // let member = ctx.guild_channel().await.unwrap().member;
    // let permission_check = member.permissions;
    let permission_check = member.permissions;

    println!("permission_check = {:?}", permission_check);

    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "감옥행버튼")]
pub async fn silence_user(
    ctx: Context<'_>,
    #[description = "유저 선택"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let author = &ctx.author().name;
    if author != "eorb" {
        return Ok(());
    }

    let user_id = user.as_ref().unwrap().id;

    let allow = Permissions::VIEW_CHANNEL;
    let deny = Permissions::SEND_MESSAGES
        | Permissions::SEND_VOICE_MESSAGES
        | Permissions::CONNECT
        | Permissions::SPEAK;

    let per_over = PermissionOverwrite {
        allow,
        deny,
        kind: PermissionOverwriteType::Member(user_id),
        // kind: PermissionOverwriteType::Role(role_id),
    };

    let user_name = user.unwrap();

    let channel = ctx.guild_channel().await.unwrap();
    let permission_change = channel
        .create_permission(ctx.serenity_context(), per_over)
        .await?;
    println!("Permission change = {:?}", permission_change);

    let _ = ctx.say(user_name.name.to_string() + " 입막음").await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "출소버튼")]
pub async fn unsilence_user(
    ctx: Context<'_>,
    #[description = "유저 선택"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let author = &ctx.author().name;
    if author != "eorb" {
        return Ok(());
    }

    let user_id = user.as_ref().unwrap().id;

    let allow =
        Permissions::SEND_MESSAGES | Permissions::SEND_VOICE_MESSAGES | Permissions::CONNECT;
    let deny = Permissions::ADMINISTRATOR;

    let per_over = PermissionOverwrite {
        allow,
        deny,
        kind: PermissionOverwriteType::Member(user_id),
        // kind: PermissionOverwriteType::Role(role_id),
    };

    let user_name = user.unwrap();

    let channel = ctx.guild_channel().await.unwrap();
    let permission_change = channel
        .create_permission(ctx.serenity_context(), per_over)
        .await?;

    println!("Permission change = {:?}", permission_change);

    let _ = ctx.say(user_name.name.to_string() + " 입막음 해제").await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "테러")]
pub async fn spamming(
    ctx: Context<'_>,
    #[description = "유저 선택"] user: Option<serenity::User>,
    #[description = "반복 횟수"] number: i16,
) -> Result<(), Error> {
    let user_name = &user.as_ref().unwrap().name;
    let user_name = &user_nickname(user_name);
    let _ = ctx.say("Spamming").await?;
    for _ in 1..=number {
        // let user_name = &user.as_ref().unwrap().name;
        let embed = CreateEmbed::new()
            .title("응답하라 ".to_string() + &user_name)
            .timestamp(Timestamp::now());
        let builder = CreateMessage::new().embed(embed);
        let user = user.as_ref().unwrap();
        let _ = user.direct_message(ctx.serenity_context(), builder).await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "하이딩금지")]
pub async fn hidemember_find(
    ctx: Context<'_>,
    #[description = "유저 선택"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user_id = user.unwrap().id;
    let channel_id = ctx.channel_id();
    let member = ctx
        .guild_id()
        .unwrap()
        .member(ctx.serenity_context(), user_id)
        .await
        .unwrap();
    let _ = member.move_to_voice_channel(ctx.http(), channel_id).await?;
    let _ = ctx.say("m").await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "하이딩금지2")]
pub async fn hidemember_find2(
    ctx: Context<'_>,
    #[description = "유저 선택"] user: Option<serenity::User>,
) -> Result<(), Error> {
    // let members = ctx.guild_channel().await.unwrap().members(ctx.cache()).unwrap();
    let user_id = user.unwrap().id;
    // let guild_id = ctx.guild_id();
    let guild = ctx.guild().unwrap();
    let temp = guild.presences.get(&user_id);
    println!("temp = {:?}", temp);

    // let mut offline_members  = Vec::new();
    // for member in members {
    //     member.status
    // }
    // println!("guild member = {:?}", guild);
    // let _ = ctx.say("").await;
    // let user = user.unwrap();
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "조인")]
pub async fn voice_join(ctx: Context<'_>) -> Result<(), Error> {
    let _ = voice_join_noncommand(ctx).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "아웃")]
pub async fn voice_out(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            if let Err(why) = ctx.say(format!("Failed: {:?}", e)).await {
                println!("Error sending message: {why:?}");
            }
        }
        let _ = ctx.say("잘있어 얘들아..").await?;
    } else {
        let _ = ctx.say("보이스채널에 없습니다.").await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "귀막아")]
pub async fn voice_deaf(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let voice_ctx = ctx.serenity_context();

    let manager = songbird::get(voice_ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            let _ = ctx.say("보이스 채널에 없습니다.").await;
            return Err("보이스 채널에 없습니다.".into());
        }
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_deaf() {
        let _ = ctx.say("이미 귀를막은 상태입니다.").await;
    } else {
        if let Err(e) = handler.deafen(true).await {
            let _ = ctx.say(format!("Failed: {:?}", e)).await;
        }
        if let Err(why) = ctx.say("귀막기").await {
            println!("Error sending message: {why:?}");
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "뮤트")]
pub async fn voice_mute(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            // ctx.say("보이스 채널에 없습니다.").await;
            return Err("err".into());
        }
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        ctx.say("이미 뮤트 상태입니다.").await?;
    } else {
        if let Err(e) = handler.mute(true).await {
            ctx.say("보이스채널에 없습니다.").await?;
        } else {
            ctx.say("뮤트 완료").await?;
        }
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "귀열어")]
pub async fn voice_undeaf(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            let _ = ctx.say("보이스 채널에 없습니다.").await;
            return Err("보이스 채널에 없습니다.".into());
        }
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_deaf() {
        if let Err(e) = handler.deafen(false).await {
            let _ = ctx
                .channel_id()
                .say(ctx.http(), format!("Failed: {:?}", e))
                .await;
        }
        let _ = ctx.say("귀막기 성공!").await?;
    } else {
        let _ = ctx.say("이미 귀를 막은 상태입니다.").await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "재생")]
pub async fn queue(
    ctx: Context<'_>,
    #[description = "검색 및 URL"] url: Option<String>,
) -> Result<(), Error> {
    let _ = queue_noncommand(ctx, url).await;
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "정지")]
pub async fn pause(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.pause();
        let _ = ctx.say("음악 정지").await?;
    } else {
        let _ = ctx.say("보이스 채널에 없습니다.").await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "언뮤트")]
pub async fn voice_unmute(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        if let Err(e) = handler.mute(false).await {
            let _ = ctx.say(format!("Error = {}", e)).await?;
        }
        let _ = ctx.say("언뮤트").await?;
        return Ok(());
    } else {
        if let Err(why) = ctx
            .channel_id()
            .say(ctx.http(), "보이스 채널에 없습니다.")
            .await
        {
            println!("Error sending message: {why:?}");
        };
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "재개")]
pub async fn resume(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        println!("QUEUE = {queue:?}");

        let _ = queue.resume();
        let _ = ctx.say("음악 재개").await?;

        return Ok(());
    } else {
        let _ = ctx.say("보이스 채널에 없습니다.").await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "넘기기")]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        //이부분에 큐의 개수를 확인하고 실행할지 말지 결정하는 코드 작성 예정
        if queue.len() == 0 {
            let _ = ctx.say("남아있는 곡이 없습니다.").await?;
            return Ok(());
        }
        let _ = queue.skip();
        let _ = ctx
            .say(format!("곡 넘기기: {}곡 남았습니다.", queue.len() - 1))
            .await;
    } else {
        let _ = ctx.say("보이스 채널에 없습니다.").await;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "플레이리스트")]
pub async fn play_list(ctx: Context<'_>) -> Result<(), Error> {
    // let _ = ctx.say("현재 재생중인 PLAY LIST").await;
    let builder = {
        let queue = {
            let data_read = ctx.serenity_context().data.read().await;
            data_read
                .get::<SongQueue>()
                .expect("Expected StrictMod in TypeMap.")
                .clone()
        };
        let queue = queue.lock().unwrap();
        let builder = CreateMessage::new();
        let mut embed_vec: Vec<CreateEmbed> = Vec::new();
        for val in queue.iter() {
            let embed_in = CreateEmbed::new().field(
                val.artist.clone().unwrap_or("None".to_string()),
                val.title.clone().unwrap_or("None".to_string()),
                false,
            );
            embed_vec.push(embed_in);
        }
        builder.add_embeds(embed_vec)
    };
    let _ = ctx.channel_id().send_message(ctx.http(), builder).await;

    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "다음곡")]
pub async fn next_playing(ctx: Context<'_>) -> Result<(), Error> {
    let _ = ctx.say("다음에 재생될 곡").await;
    let aux_data = {
        let queue = {
            let data_read = ctx.serenity_context().data.read().await;
            data_read
                .get::<SongQueue>()
                .expect("Expected StrictMod in TypeMap.")
                .clone()
        };
        let queue = queue.lock().unwrap();

        // let first_ele = queue[0].to_owned();
        // println!("first_ele")
        // let embed = song_information(first_ele).await;
        queue[0].to_owned()
    };
    let builder = song_information(aux_data).await;

    let _ = ctx.channel_id().send_message(ctx.http(), builder).await;
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "반복")]
pub async fn repeat_song(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let now = queue.current().unwrap();
        let now_info = now.get_info().await.unwrap();
        if now_info.loops == LoopState::Infinite {
            let _ = now.disable_loop();
            let _ = ctx.say("반복 해제 완료").await?;
        } else {
            let _ = now.enable_loop();
            let _ = ctx.say("반복 설정 완료").await?;
        }
    } else {
        let _ = ctx.say("보이스 채널에 없습니다.").await?;
    }
    Ok(())
}

#[poise::command(slash_command, prefix_command, rename = "반복해제")]
pub async fn repeat_song_disable(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let now = queue.current().unwrap();

        let now_info = now.get_info().await.unwrap();
        if now_info.loops == LoopState::Infinite {
            let _ = now.disable_loop();
            let _ = ctx.say("반복 해제 완료").await?;
        } else {
            let _ = ctx.say("이미 해제된 상태입니다.").await?;
        }
    } else {
        let _ = ctx.say("보이스 채널에 없습니다.").await;
    }
    Ok(())
}

//Function

// pub fn random_exe() -> Option<bool> {
//     let secret_number = rand::thread_rng().gen_range(1..=RANDOM);
//     if secret_number == 1 {
//         Some(true)
//     } else {
//         None
//     }
// }

pub fn lucky(now: Timestamp) -> String {
    let time = now.to_string();
    let minute: Result<i32, _> = time[14..16].parse();
    let minute = minute.unwrap();
    let minute_ran = rand::thread_rng().gen_range(1..=2);
    let second = rand::thread_rng().gen_range(0..=59);
    let minute = if minute_ran + minute < 10 {
        format!("0{}", minute_ran + minute)
    } else if minute_ran + minute >= 60 {
        (minute_ran + minute - 60).to_string()
    } else {
        (minute_ran + minute).to_string()
    };

    let second = if second < 10 {
        format!("0{}", second)
    } else {
        second.to_string()
    };
    let min_sec = minute + "분" + &second.to_string() + "초";
    return min_sec;
}

pub fn user_nickname(user_name: &String) -> String {
    let user_nickname = if user_name == "eorb" {
        String::from("킹갓엠퍼러지니어스연준")
    } else if user_name == "psh0478" {
        String::from("에펙불나방성호")
    } else if user_name == "gkfrpdjqtsp" {
        String::from("나이트워커에정신과에너지를갈아넣은미련한인중")
    } else if user_name == "eh0410" {
        String::from("퇴사한다고말만번복하는고민이많은으니")
    } else {
        String::from("이방인")
    };
    user_nickname
}

pub async fn voice_join_noncommand(ctx: Context<'_>) -> Result<(), Error> {
    let (guild_id, channel_id) = {
        let user_id = ctx.author().id;
        let guild = ctx.guild().unwrap();
        let channel_id = guild
            .voice_states
            .get(&user_id)
            .and_then(|voice_state| voice_state.channel_id);

        (guild.id, channel_id)
    };
    println!("GID = {guild_id}");
    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            let _ = ctx.say("보이스 채널에 없습니다.").await;
            return Err("보이스 채널에 없습니다.".into());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        if let Err(why) = handler.deafen(true).await {
            println!("Error sending message: {why:?}");
        }
        let channelid = ctx.channel_id();
        // let http_send = ctx.serenity_context().http.clone();
        // let http_send = ctx.serenity_context().http.clone();
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
        handler.add_global_event(
            TrackEvent::End.into(),
            TrackEndNotifier {
                ctx: ctx.serenity_context().clone(),
                channel_id: channelid,
                // http: http_send,
            },
        );
        // handler.add_global_event(TrackEvent::Play.into(), TrackStartNotifier);
    }
    // voice_deaf();
    if let Err(why) = ctx.say("칭구들 곁으로 이동 완료~").await {
        println!("Error sending message: {why:?}");
    }
    Ok(())
}

pub async fn song_information(metadata: AuxMetadata) -> CreateMessage {
    let embed = CreateEmbed::new()
        .title("연준봇의 외침")
        .image(metadata.thumbnail.unwrap_or("none".to_string()))
        .field(
            "아티스트",
            metadata.artist.unwrap_or("none".to_string()),
            true,
        )
        .field("제목", metadata.title.unwrap_or("none".to_string()), true)
        .field(
            "링크",
            metadata.source_url.unwrap_or("none".to_string()),
            false,
        )
        // .field("",  (match metadata.duration {
        //     Some(duration) => duration.as_secs(),
        //     None => 0,
        // }).to_string(),false)
        .timestamp(Timestamp::now());
    let builder = CreateMessage::new().embed(embed);
    // .add_file(CreateAttachment::path("./ferris_eyes.png").await.unwrap());
    builder
}

pub async fn queue_noncommand(ctx: Context<'_>, url: Option<String>) -> Result<(), Error> {
    let cp_url = url.clone();
    let url = url.unwrap_or("나얼".to_string());
    let url = url.trim().to_string();
    let do_search = !url.starts_with("http");
    let do_list = url.contains("&list");
    let guild_id = ctx.guild_id().unwrap();

    let http_client = {
        let data = ctx.serenity_context().data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        // Here, we use lazy restartable sources to make sure that we don't pay
        // for decoding, playback on tracks which aren't actually live yet.

        //초반에는 빠르게 시작하는 코드 작성

        //큐에 음악이 없으면 빠르게 재생?

        //곡을 추가할때는 state 를 확인

        //재생중인지 확인

        let mut src = if do_search {
            YoutubeDl::new_search(http_client, url)
        } else if do_list {
            // YoutubeDl::new_list(http_client, url)
            YoutubeDl::new(http_client, url)
        } else {
            YoutubeDl::new(http_client, url)
        };
        let mut src_copy = src.clone();
        let metadata = src.aux_metadata().await;

        if handler.queue().len() == 0 {
            //처음 시작 모드
            match metadata {
                Ok(metadata) => {
                    let builder = song_information(metadata).await;
                    let msg = ctx.channel_id().send_message(ctx, builder).await;
                    if let Err(why) = msg {
                        println!("Error sending message: {why:?}");
                    }
                }
                Err(error) => {
                    println!("{:?}", error);
                }
            };
        } else {
            let _ = ctx.say("추가 모드").await;
            match metadata {
                //추가 모드
                Ok(metadata) => {
                    let artist = metadata.clone().artist.unwrap_or("None".to_string());
                    let title = metadata.clone().title.unwrap_or("None".to_string());
                    let _ = ctx.say(format!("{} - {}", artist, title)).await;
                    // 데이터 저장
                    {
                        let queue = {
                            let data_read = ctx.serenity_context().data.read().await;
                            data_read
                                .get::<SongQueue>()
                                .expect("Expected StrictMod in TypeMap.")
                                .clone()
                        };
                        let mut queue = queue.lock().unwrap();
                        queue.push_back(metadata);
                        // queue.push(metadata);
                        println!("queue = {:?}", queue);
                    }
                }
                Err(error) => {
                    println!("{:?}", error);
                }
            };
            //이러면 추가 모드
        }

        // handler.enqueue_input(src.into()).await;
        // handler.enqueue(src.into()).await;
        handler.enqueue_with_preload(src.into(), None);
        handler.set_bitrate(songbird::driver::Bitrate::Max);
        println!("handler in enqueue = {:?}", handler);
        //추가 모드일떄는 추가하는 메시지 뜨지 않도록 변경

        let add_message = ctx.say("곡을 추가하는 중입니다...").await;
        let src_list = src_copy.search(Some(1)).await;
        let src_list = match src_list {
            Ok(v) => v,
            Err(error) => panic!("{error}"),
        };
        drop(handler);
        let _ = queue_add(ctx, src_list).await;
        //add_message Delete
        let _ = ctx
            .channel_id()
            .delete_message(ctx.http(), add_message.unwrap().message().await.unwrap().id)
            .await;

        if let Err(why) = ctx.channel_id().say(ctx.http(), "추가 완료!").await {
            println!("Error sending message: {why:?}");
        };
    } else {
        let voice_join = voice_join_noncommand(ctx).await;
        match voice_join {
            Ok(()) => {
                let _ = Box::pin(queue_noncommand(ctx, cp_url.clone())).await;
            }
            Err(err) => return Err(err),
        }
    };
    Ok(())
}

async fn queue_add(ctx: Context<'_>, src_list: Vec<AuxMetadata>) -> Result<(), Error> {
    for (index, ve) in src_list.iter().enumerate() {
        if index > 1 {
            let http_client = {
                let data = ctx.serenity_context().data.read().await;
                data.get::<HttpKey>()
                    .cloned()
                    .expect("Guaranteed to exist in the typemap.")
            };

            let guild_id = ctx.guild_id().unwrap();
            let manager = songbird::get(ctx.serenity_context())
                .await
                .expect("Songbird Voice client placed in at initialisation.")
                .clone();

            if let Some(handler_lock) = manager.get(guild_id) {
                let mut handler = handler_lock.lock().await;
                let src_url = ve.source_url.as_ref().unwrap();
                let mut new_queue = YoutubeDl::new(http_client, src_url.to_string());
                //
                let metadata = new_queue.aux_metadata().await;

                match metadata {
                    Ok(metadata) => {
                        //큐에 저장하는 부분
                        {
                            let queue = {
                                let data_read = ctx.serenity_context().data.read().await;
                                data_read
                                    .get::<SongQueue>()
                                    .expect("Expected StrictMod in TypeMap.")
                                    .clone()
                            };
                            let mut queue = queue.lock().unwrap();
                            queue.push_back(metadata);
                            // queue.push(metadata);
                            println!("queue = {:?}", queue);
                        }

                        // println!("temp data = {:?}",tempdata.get(index));
                        // let mut queue = ctx.data().song_queue.clone();
                        // queue.push(metadata);

                        // let queue = ctx.invocation_data().await.as_deref_mut();
                        // let queue = Vec::new().push(metadata);
                        // ctx.set_invocation_data(queue).await;
                        // println!("queue = {:?}", ctx.invocation_data::<&Vec<AuxMetadata>>().await.as_deref());
                        // ctx.data().song_queue.;

                        // let queue:&mut Vec<AuxMetadata> = ctx.data().song_queue.borrow_mut();
                        // queue.push(metadata);
                        // let builder = song_information(metadata).await;
                        // let msg = ctx.channel_id().send_message(ctx.http(), builder).await;
                        // if let Err(why) = msg {
                        //     println!("Error sending message: {why:?}");
                        // }
                    }
                    Err(error) => return Err(error.into()),
                };
                //
                handler.enqueue_input(new_queue.into()).await;
                if index >= 10 {
                    break;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]

mod tests {
    // use super::random_exe;

    #[test]
    fn add() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
    // #[test]
    // fn random() {
    //     let ra = random_exe();
    //     assert_eq!(ra.unwrap(), true);
    // }
}
