table! {
    edits (id) {
        id -> Integer,
        edit -> Text,
        team_id -> Nullable<Text>,
        user_id -> Nullable<Text>,
        replace_with_text -> Nullable<Text>,
        replace_with_file -> Nullable<Text>,
    }
}

table! {
    trigger (id) {
        id -> Integer,
        triggered_by -> Text,
        emoji -> Nullable<Text>,
        text_ -> Nullable<Text>,
        team_id -> Text,
    }
}

table! {
    blague (id) {
        id -> Integer,
        team_id -> Text,
        text -> Text,
    }
}

allow_tables_to_appear_in_same_query!(edits, trigger,);
