use asciicker_rs::y6::prelude::*;

const RADIUS: f32 = 4f32;

#[tokio::main]
async fn main() {
    let bot = Bot::new("player", "ws://asciicker.com/ws/y6/", true);
    let (threads, data) = match bot.run().await {
        Err(e) => panic!("Failed to run the bot: {:?}", e),
        Ok(stuff) => stuff,
    };
    let mut i = 0f32;
    loop {
        match *threads.0.is_finished.lock().await {
            true => {
                println!("{:?}", threads.0.thread.await);
                return;
            }
            _ => {}
        };
        let mut bot = data.0.lock().await;
        let x = i.cos() * RADIUS;
        let y = i.sin() * RADIUS;
        bot.pose.position = [x, y, 300f32];
        i += 0.00001;
        if i >= std::f32::consts::PI * 2f32 {
            i = 0f32;
        }
    }
}
