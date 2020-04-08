use crate::models::GenericPost;
use crate::models::GenericPostEdited;
use crate::models::GenericStatus;
use crate::models::StatusCode;
use crate::models::StatusError as GenericStatusError;
use crate::models::{GenericEvent, GenericHello};
use serde::{Deserialize, Serialize};
use std::convert::Into;

#[derive(Serialize, Deserialize)]
pub struct Auth {
    token: String,
}

#[derive(Serialize)]
pub struct PostEdit<'a> {
    pub message: Option<&'a str>,
    pub file_ids: Option<Vec<&'a str>>,
}

#[derive(Deserialize, Serialize)]
pub struct Hello {
    server_version: String,
}

#[derive(Deserialize, Serialize)]
struct Post {
    id: String,
    message: String,
    create_at: u64,
    update_at: u64,
    edit_at: u64,
    delete_at: u64,
    is_pinned: bool,
    user_id: String,
    channel_id: String,
    root_id: String,
    parent_id: String,
    original_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct Posted {
    channel_display_name: String,
    channel_name: String,
    channel_type: String,
    post: String,
    sender_name: String,
    team_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct PostEdited {
    post: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Status {
    pub status: String,
    pub error: Option<StatusDetails>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StatusDetails {
    id: String,
    message: String,
    detailed_error: String,
    request_id: Option<String>,
    status_code: f64,
    is_oauth: Option<bool>,
}

impl Into<GenericPostEdited> for PostEdited {
    fn into(self) -> GenericPostEdited {
        // FIXME: must still decode self.post
        let post: Post = serde_json::from_str(&self.post).unwrap();
        GenericPostEdited {
            user_id: post.user_id.clone(),
            message: post.message.clone(),
            id: post.id.clone(),
            channel_id: post.channel_id.clone(),
            parent_id: post.parent_id.clone(),
            root_id: post.root_id.clone(),
        }
    }
}

impl Into<GenericPost> for Posted {
    fn into(self) -> GenericPost {
        // FIXME: must still decode self.post
        let post: Post = serde_json::from_str(&self.post).unwrap();
        GenericPost {
            user_id: post.user_id.clone(),
            root_id: post.root_id.clone(),
            parent_id: post.parent_id.clone(),
            message: post.message.clone(),
            channel_id: post.channel_id.clone(),
            id: post.id.clone(),
            team_id: self.team_id.clone(),
        }
    }
}

impl Into<GenericStatusError> for StatusDetails {
    fn into(self) -> GenericStatusError {
        GenericStatusError {
            message: self.message,
            detailed_error: self.detailed_error,
            request_id: self.request_id,
            status_code: self.status_code as i32,
        }
    }
}

impl Into<GenericStatus> for Status {
    fn into(self) -> GenericStatus {
        if self.status.contains("OK") {
            return GenericStatus {
                code: StatusCode::OK,
                error: None,
            };
        }

        if self.status.contains("FAIL") {
            return GenericStatus {
                code: StatusCode::Error,
                error: Some(self.error.unwrap().into()),
            };
        }

        GenericStatus {
            code: StatusCode::Unsupported,
            error: None,
        }
    }
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
    channel_id: String,
    omit_users: Option<String>,
    team_id: String,
    user_id: String,
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

impl Into<GenericEvent> for Event {
    fn into(self) -> GenericEvent {
        match self.data {
            EventData::Posted(posted) => GenericEvent::Post(posted.into()),
            EventData::Hello(hello) => GenericEvent::Hello(GenericHello {
                my_user_id: self.broadcast.user_id.clone(),
                server_string: hello.server_version.clone(),
            }),
            EventData::PostEdited(edited) => GenericEvent::PostEdited(edited.into()),
        }
    }
}

impl Into<GenericEvent> for Status {
    fn into(self) -> GenericEvent {
        GenericEvent::Status(self.into())
    }
}

impl Into<GenericEvent> for MetaEvent {
    fn into(self) -> GenericEvent {
        match self {
            MetaEvent::Event(event) => event.into(),
            MetaEvent::Status(status) => status.into(),
            MetaEvent::Unsupported(unsupported) => GenericEvent::Unsupported(unsupported),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mattermost::*;

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
