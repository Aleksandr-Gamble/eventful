
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use tokio_nsq;
use hyperactive;
use crate::err::GenericError;


pub struct EventMessage<T> {
    pub message: tokio_nsq::NSQMessage,
    pub event: T
}
pub struct DaemonNSQ {
    pub host: String,
}


impl DaemonNSQ {
    /// Instantiate a new Daemon client from a url, typically something like http://127.0.0.1:4151
    pub fn new(host: &str) -> Self {
        DaemonNSQ{host: host.to_string()}
    }

}


/// This is such an elegant trait.  
/// If serialize + deserialize are already implemented on a struct,
/// All you have to do is define a topic to publish the message to NSQ.
/// Then you can call .publish(hsqd_host) asynchronously to publish the event.
#[async_trait]
pub trait EventNSQ: Serialize + DeserializeOwned {
    fn topic() -> &'static str;
    async fn publish_to(&self, host: &str) -> Result<(), GenericError>  {
        let topic =  <Self as EventNSQ>::topic();
        let _x = post_json(host, &topic, &self).await?;
        Ok(())
    }
}


pub trait GenConsumer<T: EventNSQ> {
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
            // the consumer only lets you get one message at a time
            // why then have the max_in_flight parameter? It just sends everything to in-flight, only to be re-queued
            // just leaving it at 1 is slow but works 
            .set_max_in_flight(1)
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

