use std::vec::Vec;
pub use aws_config;
pub use aws_sdk_sqs::{model::Message, Client, Region};
use serde::{Serialize, de::DeserializeOwned};
use serde_json;
use crate::err::EventfulError;


pub trait Event: Serialize + DeserializeOwned {
    fn queue_url() -> &'static str;
    /// Messages that belong to the same message group are always processed one by one.  
    /// [Read more](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/SQSDeveloperGuide/using-messagegroupid-property.html) on docs.aws.amazon.com 
    fn group_id(&self) -> Option<String> {
        None
    }
}



pub struct ClientSQS {
    client: Client,
}

impl ClientSQS {

    /// Instantiate a new messenger
    pub async fn new(region: &'static str) -> Self {
        let config = aws_config::from_env().region(Region::new(region)).load().await;
        let client = Client::new(&config);
        ClientSQS{client}
    }

    pub async fn poll_messages(&self, queue_url: &str, delete_on_receipt: bool) -> Result<Vec<Message>, EventfulError> {
        let message_batch = self.client
            .receive_message()
            .queue_url(queue_url)
            .send().await?;

        let messages = message_batch.messages.unwrap_or_default();
        
        if delete_on_receipt {
            for message in &messages {
                let receipt_handle = match &message.receipt_handle {
                    Some(val) => val,
                    None => continue,
                };
                let _ = &self.client.delete_message()
                    .queue_url(queue_url)
                    .receipt_handle(receipt_handle)
                    .send().await?;

            }
        }
        Ok(messages)
        
    }

    
    /// Return the body of messages as strings
    pub async fn poll_strings(&self, queue_url: &str, delete_on_receipt: bool) -> Result<Vec<String>, EventfulError> {
        let messages = self.poll_messages(queue_url, delete_on_receipt).await?;
        let mut resp = Vec::new();
        for message in messages {
            let body = &message.body.unwrap_or_default();
            resp.push(body.clone());
        }
        Ok(resp)
    }


    /// Return the body of messages as deserializable structs
    pub async fn poll<T: Event>(&self, delete_on_receipt: bool) -> Result<Vec<T>, EventfulError> {
        let messages = self.poll_messages(T::queue_url(), delete_on_receipt).await?;
        let mut resp = Vec::new();
        for message in messages {
            let body = &message.body.unwrap_or_default();
            let jz: T = serde_json::from_str(body)?; /* {
                Ok(val) => val,
                Err(_) => {
                    return Err(EventfulError{msg:"JSON dserialization error".to_string()}.into())
                }
            };*/
            resp.push(jz)
        }
        Ok(resp)
    }



    /// publish a message (could be a string or serializable struct) to the queue with a given group_id
    pub async fn publish<T: Event>(&self, event: &T) -> Result<String, EventfulError> {
        let body = serde_json::to_string(event)?;
        let send_msg = match event.group_id() {
            Some(_group_id) => { self.client
                .send_message()
                .queue_url(<T as Event>::queue_url())
                .message_body(body)
                .message_group_id("abc".to_string())},
            None => {self.client
                .send_message()
                .queue_url(<T as Event>::queue_url())
                .message_body(body)
            },
        };
        let output = send_msg.send().await?;
        let message_id = output
            .message_id.unwrap();
            //.ok_or(EventfulError{msg: "push request did not return a message_id!".to_string()})?;  
        Ok(message_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::runtime::Runtime;


}
