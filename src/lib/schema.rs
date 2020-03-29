table! {
    trigger (id) {
        id -> Integer,
        triggered_by -> Text,
        emoji -> Nullable<Text>,
        text_ -> Nullable<Text>,
        team_id -> Text,
    }
}
