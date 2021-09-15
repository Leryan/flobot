table! {
    blague (id) {
        id -> Integer,
        team_id -> Text,
        text -> Text,
    }
}

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
    sms_contact (id) {
        id -> Integer,
        team_id -> Text,
        name -> Text,
        number -> Text,
        last_sending_unixts -> BigInt,
    }
}

table! {
    sms_prepare (id) {
        id -> Integer,
        team_id -> Text,
        sms_contact_id -> Integer,
        trigname -> Text,
        name -> Text,
        text -> Text,
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

joinable!(sms_prepare -> sms_contact (sms_contact_id));

allow_tables_to_appear_in_same_query!(blague, edits, sms_contact, sms_prepare, trigger,);
