use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Posted {
    channel_display_name: String,
    channel_name: String,
    channel_type: String,
    post: String,
    sender_name: String,
    team_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum EventData {
    Posted(Posted),
}

#[derive(Serialize, Deserialize)]
struct Event {
    #[serde(rename(serialize = "event", deserialize = "event"))]
    type_: String,
    data: EventData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_valid() {
        let data = r#"{"event": "posted", "data": {"channel_display_name":"Town Square","channel_name":"town-square","channel_type":"O","post":"{\"id\":\"ghkm74cqzbnjxr5dx638k73xqa\",\"create_at\":1576937676623,\"update_at\":1576937676623,\"edit_at\":0,\"delete_at\":0,\"is_pinned\":false,\"user_id\":\"kh9859j8kir15dmxonsm8sxq1w\",\"channel_id\":\"amtak96j3br5iyokgunmf188jc\",\"root_id\":\"\",\"parent_id\":\"\",\"original_id\":\"\",\"message\":\"test\",\"type\":\"\",\"props\":{},\"hashtags\":\"\",\"pending_post_id\":\"kh9859j8kir15dmxonsm8sxq1w:1576937676569\",\"metadata\":{}}","sender_name":"@admin","team_id":"49ck75z1figmpjy6eknrohsjnw"}, "broadcast": {"omit_users":null,"user_id":"","channel_id":"amtak96j3br5iyokgunmf188jc","team_id":""}, "seq": 7}"#;
        let valid: Event = serde_json::from_str(data).unwrap();

        assert_eq!(valid.type_, "posted");

        match valid.data {
            EventData::Posted(event) => {
                assert_eq!(event.channel_display_name, "Town Square");
                assert_eq!(event.channel_name, "town-square");
                assert_eq!(event.channel_type, "O");
                assert_ne!(event.post, "");
            }
        }
    }

    #[test]
    #[should_panic]
    fn post_invalid() {
        let data = r#"{"event": "posted", "data": {"invalid":"invalid"}}"#;
        let _invalid: Event = serde_json::from_str(data).unwrap();
    }
}
