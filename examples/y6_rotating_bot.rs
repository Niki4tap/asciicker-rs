use asciicker_rs::y6::prelude::*;

#[tokio::main]
async fn main() {
    let bot = Bot::new("player", "ws://asciicker.com/ws/y6/", true);
    let (threads, data) = match bot.run().await {
        Err(e) => panic!("Failed to run the bot: {:?}", e),
        Ok(stuff) => stuff,
    };
    let mut i: f32 = 0f32;
    loop {
        match *threads.0.is_finished.lock().await {
            true => {
                println!("{:?}", threads.0.thread.await);
                return;
            }
            _ => {}
        };
        let mut bot = data.0.lock().await;
        bot.pose.direction = i;
        i += 0.01f32;
        if i >= 360f32 {
            i = 0f32;
        }
    }
}
