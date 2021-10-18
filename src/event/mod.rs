mod sub_event_handler;
mod bot_start;

use sub_event_handler::SubEventHandler;


use futures::lock::Mutex;
use serenity::{async_trait, client::{Context, EventHandler}, model::prelude::Ready};


#[derive(PartialEq, Eq, Hash, Clone)]
enum EventType {
    OnCacheReady,
    OnChannelCreate,
    OnCategoryCreate,
    OnCategoryDelete,
    OnChannelDelete,
    OnChannelPinsUpdate,
    OnChannelUpdate,
    OnGuildBanAddition,
    OnGuildBanRemoval,
    OnGuildCreate,
    OnGuildDelete,
    OnGuildEmojisUpdate,
    OnGuildIntegrationsUpdate,
    OnGuildMemberAddition,
    OnGuildMemberRemoval,
    OnGuildMemberUpdate,
    OnGuildMembersChunk,
    OnGuildRoleCreate,
    OnGuildRoleDelete,
    OnGuildRoleUpdate,
    OnGuildUnavailable,
    OnGuildUpdate,
    OnInviteCreate,
    OnInviteDelete,
    OnMessage,
    OnMessageDelete,
    OnMessageDeleteBulk,
    OnMessageUpdate,
    OnReactionAdd,
    OnReactionRemove,
    OnReactionRemoveAll,
    OnPresenceReplace,
    OnPresenceUpdate,
    OnReady,
    OnResume,
    OnShardStageUpdate,
    OnTypingStart,
    OnUnknown,
    OnUserUpdate,
    OnVoiceServerUpdate,
    OnVoiceStateUpdate,
    OnWebhookUpdate,
    OnInteractionCreate,
    OnIntegrationCreate,
    OnIntegrationUpdate,
    OnIntegrationDelete,
    OnApplicationCommandCreate,
    OnApplicationCommandUpdate,
    OnApplicationCommandDelete,
    OnStageInstanceCreate,
    OnStageInstanceUpdate,
    OnStageInstanceDelete,
    OnThreadCreate,
    OnThreadUpdate,
    OnThreadDelete,
    OnThreadListSync,
    OnThreadMemberUpdate,
    OnThreadMembersUpdate,
}

struct EventListener {
    name: String,
    listener: Mutex<Box<dyn SubEventHandler>>,
}

#[derive(Default)]
pub struct EventListenerContainer {
    event_listeners: Vec<EventListener>,
    // event_helper: HashMap<EventType, Vec<Arc<dyn EventHandler>>>
}

impl EventListenerContainer {
    pub fn init() -> EventListenerContainer {
        use EventType::*;
        let mut evts = EventListenerContainer::default();
        evts.register_event_listener("bot_start", Box::new(bot_start::BotStart), vec![OnReady]);
        evts
    }
    fn register_event_listener(&mut self, name: &str, event_listener: Box<dyn SubEventHandler>, _:Vec<EventType>) {
        self.event_listeners.push(EventListener {
            name: name.to_string(),
            listener: Mutex::new(event_listener),
        });
    }
}

#[async_trait]
impl EventHandler for EventListenerContainer {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let ctx = Mutex::new(ctx);
        let ready = Mutex::new(ready);
        for evt in &self.event_listeners {
            let mut evt = evt.listener.lock().await;
            evt.as_mut().ready(&ctx, &ready).await
        }
    }
}
