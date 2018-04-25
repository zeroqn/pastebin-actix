table! {
    pastes (id) {
        id -> Int8,
        title -> Varchar,
        body -> Text,
        created_at -> Timestamp,
        modified_at -> Timestamp,
    }
}
