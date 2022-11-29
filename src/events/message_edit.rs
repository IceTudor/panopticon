use serenity::{prelude::Context, model::prelude::{ChannelId, MessageUpdateEvent}};
use tracing::{error, warn};

use crate::Handler;

impl Handler {
    pub async fn on_message_edit(&self, ctx: &Context, event: MessageUpdateEvent) {
        if let None = event.guild_id {
            return;
        }

        match self.redis.get_message(
            event.guild_id.unwrap().0 as i64,
            event.channel_id.0 as i64,
            event.id.0 as i64
        ).await {
            Ok(message) => {
                match message {
                    Some(message) => {
                        let (user_id, message) = message.split_once(":").unwrap();
                        
                        match self.redis.set_message(
                            event.guild_id.unwrap().0 as i64,
                            event.channel_id.0 as i64,
                            event.id.0 as i64,
                            event.author.as_ref().unwrap().id.0 as i64,
                            event.content.as_ref().unwrap().clone()
                        ).await {
                            Ok(_) => {},
                            Err(err) => {
                                error!("Failed to set message in Redis. Failed with error: {}", err);
                            }
                        }

                        let guild = match self.mongo.get_guild(event.guild_id.unwrap().0 as i64).await {
                            Ok(guild) => guild,
                            Err(err) => {
                                error!("Failed to get guild {}. Failed with error: {}", event.guild_id.unwrap().0 as i64, err);
                                return;
                            }
                        };

                        if let Some(logging_config) = guild.config.logging {
                            match ChannelId(logging_config.logging_channel as u64)
                            .send_message(ctx.http.as_ref(), |msg| {
                                msg
                                    .content(format!("Message edited in <#{}> by <@{}>:\n\n**Old:**\n`{}`\n\n**New:**\n`{}`", event.channel_id.0 as i64, user_id, message.replace("`", r"\`"), event.content.as_ref().unwrap().replace("`", r"\`")))
                            }).await {
                                Ok(_) => {},
                                Err(err) => {
                                    error!("Failed to send message to logging channel. Failed with error: {}", err);
                                    return;
                                }
                            };
                        }
                    },
                    None => {
                        warn!("Message not found in Redis. This should not happen.");
                    }
                }
            },
            Err(err) => {
                error!("Failed to get message. Failed with error: {}", err);
                return;
            }
        }
    }
}