// Diesel schema mapping for the sentiment_results migration.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "sentiment_type"))]
    pub struct SentimentType;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SentimentType;

    sentiment_results (id) {
        id -> Uuid,
        input_text -> Text,
        sentiment -> SentimentType,
        probability -> Float8,
        created_at -> Timestamptz,
    }
}
