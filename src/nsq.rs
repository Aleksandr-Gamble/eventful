//! The NSQ module make it easy to produce and consume events using the [NSQ messaging platform](https://nsq.io/)
 
use std::env;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use tokio_nsq;
use hyperactive;
use crate::err::GenericError;


/// The DaemonNSQ struct stores the host information for one [nsqd daemon](https://nsq.io/components/nsqd.html) 
pub struct DaemonNSQ {
    pub host: String,
}


impl DaemonNSQ {
    /// Instantiate a new Daemon client from a url, typically something like http://127.0.0.1:4151
    pub fn new(host: &str) -> Self {
        DaemonNSQ{host: host.to_string()}
    }

}


/// This elegant trait makes it super simple to send a struct as an event.  
/// If a struct implements Serialize + DeserializeOwned, 
/// all you have to do is define a topic to publish the message to NSQ.  
/// Then you can call .publish_to(host) asynchronously to publish the event.
/// # Examples:
/// ```
/// use serde::{Serialize, Deserialize};
/// 
/// #[derive(Serialize, Deserialize)]
/// struct UserClickedSomething {
///     user_id: i32, 
///     clicked_on: String,
/// }
/// 
/// impl EventNSQ for UserClickedSomething {
///     fn topic() -> &'static str {
///         "website_clicks"
///     }
/// }
/// 
/// let click = UserClickedSomething{user_id: 5, clicked_on: "some_button".to_string()};
/// click.publish_to("http://127.0.0.1:4151").await.unwrwap();
/// ```
#[async_trait]
pub trait EventNSQ: Serialize + DeserializeOwned {
    fn topic() -> &'static str;
    async fn publish_to(&self, host: &str) -> Result<(), GenericError>  {
        let topic =  <Self as EventNSQ>::topic();
        let _x = post_json(host, &topic, &self).await?;
        Ok(())
    }
    /// Publish, using an environment variable for the NSQ host
    async fn publish_env(&self, var: &str) -> Result<(), GenericError> {
        let host = match env::var(var) {
            Ok(val) => val,
            Err(_) => "http://127.0.0.1:4151".to_string(),
        };
        self.publish_to(&host).await
    }
}



/// The ChannelConsumer trait can be instantiated on a struct to make it easy to
/// (1) Generate a tokio_nsq::NSQConsumer, via the .consumer() method
/// (2) Deserialize events from the body of a tokio_nsq::NSQMessage, via the .deserialize_event(&message) method.  
/// NOTE: This trait does not manage the async processing of messages:
/// The function signature required to do so would be (1) cumbersome to implement, and
/// (2) might not be ideal for all use cases.  
/// A common use case might be to implement ChannelConsumer<T: EventNSQ>
/// Then implement a custom async fn run(&self) -> Result<(), GenericError> or similar.
/// # Examples:
/// ```
/// use serde::{Serialize, Deserialize};
/// 
/// #[derive(Serialize, Deserialize)]
/// struct UserClickedSomething {
///     user_id: i32, 
///     clicked_on: String,
/// }
/// 
/// impl EventNSQ for UserClickedSomething {
///     fn topic() -> &'static str {
///         "website_clicks"
///     }
/// }
/// 
/// struct ClickConsumer{}
/// 
/// impl ChannelConsumer<UserClickedSomething> for ClickConsumer P
///     fn channel(&sefl) -> String {
///         "first_chanel".to_string()
///     }
/// }
/// 
/// impl ClickProcessor {
///     async fn run(&self) -> Result<(), GenericError> {
///         let mut consumer = self.consumer();
///         loop {
///             let message = consumer.consume_filtered().await.unwrap();
///             let event = self.deserialize_event(&message)?;
///             println!("    CONSUME:  user_id={} clicked_on='{}'", &event.user_id, &event.clicked_on);
///             message.finish().await;
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait ChannelConsumer<T: EventNSQ> {
    /// This method must be implemented to set the channel 
    /// It is the only one that does not have a default implementation
    fn channel(&self) -> String;
    /// This method will often be implemented to set the configuration, but should work 'out of the box'
    /// if your NSQ setup has the conventional hosts/ports
    fn config_source(&self) -> tokio_nsq::NSQConsumerConfigSources {
        tokio_nsq::NSQConsumerConfigSources::Daemons(vec!["127.0.0.1:4150".to_string()])
    }
    /// For most use cases, this defaul implementation would likely not be overwritten 
    fn consumer(&self) -> tokio_nsq::NSQConsumer {
        let topic = tokio_nsq::NSQTopic::new(<T as EventNSQ>::topic()).unwrap();
        let channel = tokio_nsq::NSQChannel::new(&self.channel()).unwrap();
        let config_source = self.config_source();
        let config = tokio_nsq::NSQConsumerConfig::new(topic, channel)
            .set_max_in_flight(15)
            .set_sources(config_source);
        config.build()
    }
    fn deserialize_event(&self, message: &tokio_nsq::NSQMessage) -> Result<T, GenericError> {
        let event: T = serde_json::from_slice(&message.body)?;
        Ok(event)
    }
}

pub async fn post_json<T: Serialize>(host: &str, topic: &str, body: &T) -> Result<(), GenericError> {
    let url = format!("{}/pub?topic={}", &host, topic);
    let _x: () = hyperactive::client::post_noback(&url, &body, None).await?;
    Ok(())
}


pub async fn post_event<T: EventNSQ>(host: &str, event: &T) -> Result<(), GenericError> {
    let topic = <T as EventNSQ>::topic();
    post_json(host, topic, event).await
}

pub async fn post_to<T: EventNSQ>(event: &T, daemon: &DaemonNSQ) -> Result<(), GenericError> {
    post_event(&daemon.host, event).await
}

