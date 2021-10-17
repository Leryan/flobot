use flobot_lib::models as gm;
use serde::{Deserialize, Serialize};
use std::convert::Into;

#[derive(Serialize)]
pub struct Metadata {}

#[derive(Serialize)]
pub struct Props {}

#[derive(Serialize)]
pub struct NewPost<'a> {
    pub channel_id: String,
    pub create_at: u64,
    pub file_ids: Vec<String>,
    pub message: &'a str,
    pub metadata: Metadata,
    pub props: Props,
    pub update_at: u64,
    pub user_id: String,
    pub root_id: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Post {
    pub id: String,
    pub message: String,
    pub create_at: u64,
    pub update_at: u64,
    pub edit_at: u64,
    pub delete_at: u64,
    pub is_pinned: bool,
    pub user_id: String,
    pub channel_id: String,
    pub root_id: String,
    pub original_id: String,
}

#[derive(Debug, Serialize)]
pub struct CreateChannel<'a> {
    pub team_id: &'a str,
    pub name: &'a str,
    pub display_name: &'a str,
    #[serde(rename = "type")]
    pub type_: &'a str,
}

#[derive(Serialize)]
pub struct Reaction {
    pub user_id: String,
    pub post_id: String,
    pub emoji_name: String,
}

#[derive(Deserialize, Debug)]
pub struct GenericID {
    pub id: String,
    pub status_code: Option<usize>,
    pub message: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Serialize)]
pub struct UserID {
    pub user_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Auth {
    pub token: String,
}

#[derive(Serialize)]
pub struct PostEdit<'a> {
    pub message: Option<&'a str>,
    pub file_ids: Option<Vec<&'a str>>,
}

#[derive(Deserialize, Serialize)]
pub struct Hello {
    pub server_version: String,
}

#[derive(Clone, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
}

#[derive(Deserialize, Serialize)]
pub struct Posted {
    pub channel_display_name: String,
    pub channel_name: String,
    pub channel_type: String,
    pub post: String,
    pub sender_name: String,
    pub team_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct PostEdited {
    pub post: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Status {
    pub status: String,
    pub error: Option<StatusDetails>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StatusDetails {
    pub id: String,
    pub message: String,
    pub detailed_error: String,
    pub request_id: Option<String>,
    pub status_code: f64,
    pub is_oauth: Option<bool>,
}

impl Into<gm::PostEdited> for PostEdited {
    fn into(self) -> gm::PostEdited {
        // FIXME: must still decode self.post
        let post: Post = serde_json::from_str(&self.post).unwrap();
        gm::PostEdited {
            user_id: post.user_id.clone(),
            message: post.message.clone(),
            id: post.id.clone(),
            channel_id: post.channel_id.clone(),
            parent_id: post.root_id.clone(),
            root_id: post.root_id.clone(),
        }
    }
}

impl Into<gm::User> for User {
    fn into(self) -> gm::User {
        gm::User {
            id: self.id,
            display_name: self.username.clone(),
            username: self.username.clone(),
        }
    }
}

impl Into<gm::Post> for Posted {
    fn into(self) -> gm::Post {
        // FIXME: must still decode self.post
        let post: Post = serde_json::from_str(&self.post).unwrap();
        gm::Post {
            user_id: post.user_id.clone(),
            root_id: post.root_id.clone(),
            parent_id: post.root_id.clone(),
            message: post.message.clone(),
            channel_id: post.channel_id.clone(),
            id: post.id.clone(),
            team_id: self.team_id.clone(),
        }
    }
}

impl Into<gm::StatusError> for StatusDetails {
    fn into(self) -> gm::StatusError {
        gm::StatusError {
            message: self.message,
            detailed_error: self.detailed_error,
            request_id: self.request_id,
            status_code: self.status_code as i32,
        }
    }
}

impl Into<gm::Status> for Status {
    fn into(self) -> gm::Status {
        if self.status.contains("OK") {
            return gm::Status {
                code: gm::StatusCode::OK,
                error: None,
            };
        }

        if self.status.contains("FAIL") {
            return gm::Status {
                code: gm::StatusCode::Error,
                error: Some(self.error.unwrap().into()),
            };
        }

        gm::Status {
            code: gm::StatusCode::Unsupported,
            error: None,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Me {
    pub id: String,
    pub username: String,
    pub email: String,
    pub nickname: String,
    pub first_name: String,
    pub last_name: String,
    pub is_bot: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventData {
    Posted(Posted),
    PostEdited(PostEdited),
    Hello(Hello),
}

#[derive(Serialize, Deserialize)]
pub struct Broadcast {
    pub channel_id: String,
    pub omit_users: Option<String>,
    pub team_id: String,
    pub user_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Event {
    #[serde(rename(serialize = "event", deserialize = "event"))]
    type_: String,
    data: EventData,
    broadcast: Broadcast,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetaEvent {
    Status(Status),
    Event(Event),
    Unsupported(String),
}

impl Into<gm::Event> for Event {
    fn into(self) -> gm::Event {
        match self.data {
            EventData::Posted(posted) => gm::Event::Post(posted.into()),
            EventData::Hello(hello) => gm::Event::Hello(gm::Hello {
                server_string: hello.server_version.clone(),
            }),
            EventData::PostEdited(edited) => gm::Event::PostEdited(edited.into()),
        }
    }
}

impl Into<gm::Event> for Status {
    fn into(self) -> gm::Event {
        gm::Event::Status(self.into())
    }
}

impl Into<gm::Event> for MetaEvent {
    fn into(self) -> gm::Event {
        match self {
            MetaEvent::Event(event) => event.into(),
            MetaEvent::Status(status) => status.into(),
            MetaEvent::Unsupported(unsupported) => gm::Event::Unsupported(unsupported),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_valid() {
        let data = r#"{"event": "posted", "data": {"channel_display_name":"Town Square","channel_name":"town-square","channel_type":"O","post":"{\"id\":\"ghkm74cqzbnjxr5dx638k73xqa\",\"create_at\":1576937676623,\"update_at\":1576937676623,\"edit_at\":0,\"delete_at\":0,\"is_pinned\":false,\"user_id\":\"kh9859j8kir15dmxonsm8sxq1w\",\"channel_id\":\"amtak96j3br5iyokgunmf188jc\",\"root_id\":\"\",\"parent_id\":\"\",\"original_id\":\"\",\"message\":\"test\",\"type\":\"\",\"props\":{},\"hashtags\":\"\",\"pending_post_id\":\"kh9859j8kir15dmxonsm8sxq1w:1576937676569\",\"metadata\":{}}","sender_name":"@admin","team_id":"49ck75z1figmpjy6eknrohsjnw"}, "broadcast": {"omit_users":null,"user_id":"","channel_id":"amtak96j3br5iyokgunmf188jc","team_id":""}, "seq": 7}"#;
        let valid: MetaEvent = serde_json::from_str(data).unwrap();
        let event = match valid {
            MetaEvent::Event(event) => event,
            _ => panic!("wrong type"),
        };

        assert_eq!(event.type_, "posted");

        match event.data {
            EventData::Posted(event) => {
                assert_eq!(event.channel_display_name, "Town Square");
                assert_eq!(event.channel_name, "town-square");
                assert_eq!(event.channel_type, "O");
                assert_ne!(event.post, "");
            }
            _ => panic!("event type not tested"),
        }
    }

    #[test]
    fn post_edited() {
        let data = r#"{"event": "post_edited", "data": {"post": "{\"id\":\"f4nj6eim7ir8fm6w9a1r75zwmy\",\"create_at\":1586031101535,\"update_at\":1586031103044,\"edit_at\":1586031103044,\"delete_at\":0,\"is_pinned\":false,\"user_id\":\"nn751zdmhfgq9k8orsiyreonbc\",\"channel_id\":\"sxoe6m6y8fr13jcajmaqbqawfh\",\"root_id\":\"\",\"parent_id\":\"\",\"original_id\":\"\",\"message\":\"!e test_team\",\"type\":\"\",\"props\":{},\"hashtags\":\"\",\"pending_post_id\":\"\",\"metadata\":{}}"}, "broadcast": {"omit_users": null, "user_id": "", "channel_id": "sxoe6m6y8fr13jcajmaqbqawfh", "team_id": ""}, "seq": 6}"#;
        let valid: MetaEvent = serde_json::from_str(data).unwrap();
        let event = match valid {
            MetaEvent::Event(event) => event,
            _ => panic!("wrong type"),
        };

        assert_eq!(event.type_, "post_edited");

        match event.data {
            EventData::PostEdited(_event) => {}
            _ => panic!("event type not tested"),
        }
    }

    #[test]
    #[should_panic]
    fn post_invalid() {
        let data = r#"{"event": "posted", "data": {"invalid":"invalid"}}"#;
        let _invalid: MetaEvent = serde_json::from_str(data).unwrap();
    }

    #[test]
    fn app_error() {
        let data = r#"{"status": "FAIL", "error": {"id": "api.web_socket_router.bad_seq.app_error", "message": "Invalid sequence for WebSocket message.", "detailed_error": "", "status_code": 400}}"#;
        let valid: MetaEvent = serde_json::from_str(data).unwrap();
        let status = match valid {
            MetaEvent::Status(status) => status,
            _ => panic!("wrong type"),
        };

        assert_eq!("FAIL", status.status);
    }
}
