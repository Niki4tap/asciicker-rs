use asciicker_rs::callback;
use asciicker_rs::macro_rules_attribute::apply;
use asciicker_rs::y6::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let mut bot = Bot::new("player", "ws://asciicker.com/ws/y6/", true);
    bot.on_talk(talk_callback);
    let (threads, _data) = match bot.run().await {
        Err(e) => panic!("Failed to run the bot: {:?}", e),
        Ok(stuff) => stuff,
    };
    println!("{:?}", threads.0.thread.await);
}

#[apply(callback!)]
pub async fn talk_callback(
    talk_brc: TalkBroadcast,
    _: Arc<Mutex<Player>>,
    _: Arc<Mutex<World>>,
    _: MessageSender,
) -> BotResult {
    println!("{:?}", talk_brc.str);
    Ok(())
}
