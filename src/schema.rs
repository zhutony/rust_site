table! {
    posts (id) {
        id -> Integer,
        content -> Text,
        parent_id -> Integer,
    }
}
// joinable!(posts -> posts(parent_id));
