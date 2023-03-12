use std::{time::Duration, collections::HashSet};
use tokio::{time::sleep};
use tokio_nsq::{NSQTopic, NSQChannel, NSQConsumerConfig, NSQConsumerConfigSources, NSQConsumerLookupConfig};
use rand::{Rng, distributions::{Alphanumeric, DistString}};
use serde::{Serialize, Deserialize};
use eventful::{err::GenericError, nsq::{DaemonNSQ, EventNSQ, ChannelConsumer}};


#[derive(Serialize, Deserialize)]
struct UserClickedSomething {
    pub user_id: i32,
    pub clicked_on: String,
}

impl EventNSQ for UserClickedSomething {
    fn topic() -> &'static str {
        "click"
    }
}

pub struct ClickProcessor{}


impl eventful::nsq::ChannelConsumer<UserClickedSomething> for ClickProcessor {
    fn channel(&self) -> String {
        format!("some_channel")
    }
}

impl ClickProcessor {
    async fn run(&self) -> Result<(), GenericError> {
        let mut consumer = self.consumer();
        loop {
            let message = consumer.consume_filtered().await.unwrap();
            let event = self.deserialize_event(&message)?;
            println!("    CONSUME:  user_id={} clicked_on='{}'", &event.user_id, &event.clicked_on);
            message.finish().await;
        }
        Ok(())
    }
}


async fn simulate_clicks() -> Result<(), GenericError> {
    let nsqd = DaemonNSQ::new("http://127.0.0.1:4151");
    loop {
        let millis: u64 = rand::thread_rng().gen_range(300..1200);
        let count: u64 = rand::thread_rng().gen_range(1..4);
        sleep(Duration::from_millis(millis)).await;
        for _ in 0..count {
            let user_id = rand::thread_rng().gen_range(0..1000);
            let clicked_on: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
            let event = UserClickedSomething{user_id, clicked_on};
            println!("PRODUCE: user_id={} clicked_on='{}'", &event.user_id, &event.clicked_on);
            let _x = event.publish_to(&nsqd.host).await?;
        }
    }
    Ok(())
}

async fn consume_events() -> Result<(), GenericError> {
    let cp = ClickProcessor{};
    cp.run().await
}


#[tokio::main]
async fn main() -> Result<(), GenericError> {
    
    tokio::spawn(async move {
        let _ = simulate_clicks().await;
    });

    // let events accumulate in NSQ for a few seconds to illustrate the decoupled nature of the producer and the consumer
    sleep(Duration::from_millis(2000u64)).await;
    let _ = consume_events().await?;

    Ok(())
}
