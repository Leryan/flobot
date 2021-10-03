use std::env::var;

#[derive(Debug, Clone)]
pub struct Conf {
    /// a channel id/name/whatever is suitable for a given backend in order to publish
    /// debugging messages from the bot.
    /// this is used by Notifier trait implementations.
    pub debug_channel: String,
    /// url to you backend api.
    pub api_url: String,
    /// for any websocket connection to handle.
    pub ws_url: String,
    /// token can contain a serialized data for a backend, such as mattermost, to authenticate the bot.
    pub token: String,
    /// should you want to use a database to maintain states for the bot, use this variable.
    pub db_url: String,
}

impl Conf {
    pub fn new() -> Result<Self, std::env::VarError> {
        Ok(Self {
            debug_channel: var("BOT_DEBUG_CHAN").expect("BOT_DEBUG_CHAN"),
            api_url: var("BOT_API_URL").expect("BOT_API_URL"),
            ws_url: var("BOT_WS_URL").expect("BOT_WS_URL"),
            token: var("BOT_TOKEN").expect("BOT_TOKEN"),
            db_url: var("BOT_DB_URL").expect("BOT_DB_URL"),
        })
    }
}
