//! The NSQ module make it easy to produce and consume events using the [NSQ messaging platform](https://nsq.io/)
 
use std::env;
use rand::Rng;
use rand::seq::SliceRandom; // 0.7.2
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use tokio_nsq;
use hyperactive;
use crate::err::GenericError;


/// let urls be a list of NSQD instances, separated by commas (,)
/// pick one at random to post an event to 
pub fn rand_nsqd_url(urls: &str) -> String {
	let sp = urls.split(",").collect::<Vec<&str>>();
    sp.choose(&mut rand::thread_rng()).unwrap().to_string()
}


pub fn rand_nsq_url() -> String {
    let i = rand::thread_rng().gen_range(1..4);
    match i {
        1 => format!("http://{}:{}", env::var("NSQ1_HOST").unwrap(), env::var("NSQ1_HTTP_PORT").unwrap() ),
        2 => format!("http://{}:{}", env::var("NSQ2_HOST").unwrap(), env::var("NSQ2_HTTP_PORT").unwrap() ),
        _ => format!("http://{}:{}", env::var("NSQ3_HOST").unwrap(), env::var("NSQ3_HTTP_PORT").unwrap() ),
    }
}

/// The Daemon struct stores the host information for one [nsqd daemon](https://nsq.io/components/nsqd.html) 
/// This representation of the Daemon struct tries to simply one mistake which can make configuration confusing for beginners 
/// when providing address data:
/// 1) The ports for production and consumption may not be the same
/// 2) NSQConsumerConfigSources need to be prefixed with http:// whereas NSQProducerConfig::new() does not 
pub struct Daemon {
    /// The host where the Daemon worker runs: typically 127.0.0.1 for localhost or nsq-nsqd1,2,3 etc. for docker deployments
    pub host: String,
    /// The http port to which events will be published, typically 4151
    pub http_port: u16,
    /// The tcp port from which events will be consumed, typically 4150
    pub tcp_port: u16,
    /// The URL to which events should be published 
    pub pub_url: String,
    /// The address from which events can be consumed 
    pub cons_address: String,
}


impl Daemon {
    pub fn new(host: &str, http_port: u16, tcp_port: u16) -> Self {
        let pub_url = format!("http://{}:{}", host, http_port);
        let cons_address = format!("{}:{}", host, tcp_port);
        Daemon{host: host.to_string(), http_port, tcp_port, pub_url, cons_address}
    }


    /// create a new Daemon from environment variables 
    pub fn new_from_env(var_host: &str, var_http_port: &str, var_tcp_port: &str) -> Self {
        let host = env::var(var_host).unwrap();
        let http_port = env::var(var_http_port).unwrap().parse::<u16>().unwrap();
        let tcp_port = env::var(var_tcp_port).unwrap().parse::<u16>().unwrap();
        Daemon::new(&host, http_port, tcp_port)
    }
}

pub struct FleetNSQ {
    pub d1: Daemon,
    pub d2: Daemon,
    pub d3: Daemon,
}

impl FleetNSQ {
    pub fn new_from_env() -> Self {
        let d1 = Daemon::new_from_env("NSQ1_HOST", "NSQ1_HTTP_PORT", "NSQ1_TCP_PORT");
        let d2 = Daemon::new_from_env("NSQ2_HOST", "NSQ2_HTTP_PORT", "NSQ2_TCP_PORT");
        let d3 = Daemon::new_from_env("NSQ3_HOST", "NSQ3_HTTP_PORT", "NSQ3_TCP_PORT");
        FleetNSQ{d1, d2, d3}
    }

    pub fn rand(&self) -> &Daemon {
        let i = rand::thread_rng().gen_range(1..4);
        match i {
            1 => &self.d1,
            2 => &self.d2,
            _ => &self.d3,
        }
    }

    pub fn as_refs<'a>(&'a self) -> [&'a Daemon; 3] {
        [&self.d1, &self.d2, &self.d3]
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
/// click.publish_to_url("http://127.0.0.1:4151").await.unwrwap();
/// ```
#[async_trait]
pub trait EventNSQ: Serialize + DeserializeOwned {
    fn topic() -> &'static str;
    async fn publish_to_url(&self, host: &str) -> Result<(), GenericError>  {
        let topic =  <Self as EventNSQ>::topic();
        let _x = post_json(host, &topic, &self).await?;
        Ok(())
    }
    async fn publish_to(&self, daemon: &Daemon) -> Result<(), GenericError> {
        self.publish_to_url(&daemon.pub_url).await
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
    fn channel(&self) -> String;

    /// This method will often be implemented to set the configuration, but should work 'out of the box'
    /// if your NSQ setup has the conventional hosts/ports
    fn config_source(&self, daemons: &[&Daemon]) -> tokio_nsq::NSQConsumerConfigSources {
        let mut addresses = Vec::new();
        for daemon in daemons {
            let address = daemon.cons_address.to_string();
            addresses.push(address);
        }
        tokio_nsq::NSQConsumerConfigSources::Daemons(addresses)
    }

    /// For most use cases, this defaul implementation would likely not be overwritten 
    fn consumer(&self, daemons: &[&Daemon]) -> tokio_nsq::NSQConsumer {
        let topic = tokio_nsq::NSQTopic::new(<T as EventNSQ>::topic()).unwrap();
        let channel = tokio_nsq::NSQChannel::new(&self.channel()).unwrap();
        let config_source = self.config_source(daemons);
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


pub async fn post_event<T: EventNSQ>(url: &str, event: &T) -> Result<(), GenericError> {
    let topic = <T as EventNSQ>::topic();
    post_json(url, topic, event).await
}

pub async fn post_to<T: EventNSQ>(event: &T, daemon: &Daemon) -> Result<(), GenericError> {
    post_event(&daemon.pub_url, event).await
}

