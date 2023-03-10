use std::env;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use hyperactive;
use crate::err::GenericError;



pub struct Daemon {
    host: String,
}


impl Daemon {
    /// Instantiate a new Daemon client from a url, typically something like http://127.0.0.1:4151
    pub fn new(host: &str) -> Self {
        Daemon{host: host.to_string()}
    }
}


/// This is such an elegant trait.  
/// If serialize + deserialize are already implemented on a struct,
/// All you have to do is define a topic to publish the message to NSQ.
/// Then you can call .publish(hsqd_host) asynchronously to publish the event.
#[async_trait]
pub trait Event: Serialize + DeserializeOwned {
    fn topic(&self) -> &'static str;
    async fn publish_to(&self, daemon: &Daemon) -> Result<(), GenericError>  {
        let topic = self.topic();
        let _x = post_msg(daemon, &topic, &self).await?;
        Ok(())
    }
}



async fn post_msg<T: Serialize>(daemon: &Daemon, topic: &str, body: &T) -> Result<(), GenericError> {
    let url = format!("{}/pub?topic={}", &daemon.host, topic);
    let _x: () = hyperactive::client::post_noback(&url, &body, None).await?;
    Ok(())
}