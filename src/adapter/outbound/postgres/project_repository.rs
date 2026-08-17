use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    application::port::project_repository::ProjectRepository,
    domain::sentiment::{Sentiment, SentimentType},
    schema::sentiment_results,
};

use super::postgres_store::PostgresStore;

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = sentiment_results)]
struct SentimentRow {
    id: Uuid,
    input_text: String,
    sentiment: SentimentType,
    probability: f64,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(diesel::Insertable)]
#[diesel(table_name = sentiment_results)]
struct NewSentimentRow<'a> {
    input_text: &'a str,
    sentiment: SentimentType,
    probability: f64,
}

impl From<SentimentRow> for Sentiment {
    fn from(row: SentimentRow) -> Self {
        let _ = (row.input_text, row.created_at);
        Self {
            prompt_id: row.id,
            sentiment: row.sentiment,
            probability: row.probability,
        }
    }
}

#[async_trait::async_trait]
impl ProjectRepository for PostgresStore {
    async fn insert(
        &self,
        input_text: &str,
        sentiment: SentimentType,
        probability: f64,
    ) -> Result<Sentiment, anyhow::Error> {
        let mut connection = self.conn().await?;
        let row = NewSentimentRow {
            input_text,
            sentiment,
            probability,
        };
        let inserted = diesel::insert_into(sentiment_results::table)
            .values(&row)
            .returning(SentimentRow::as_returning())
            .get_result(&mut connection)
            .await?;
        Ok(inserted.into())
    }

    async fn get_sentiment(&self, prompt_id: Uuid) -> Result<Option<Sentiment>, anyhow::Error> {
        let mut connection = self.conn().await?;
        let row = sentiment_results::table
            .find(prompt_id)
            .select(SentimentRow::as_select())
            .first(&mut connection)
            .await
            .optional()?;
        Ok(row.map(Into::into))
    }

    async fn delete(&self, prompt_id: Uuid) -> Result<bool, anyhow::Error> {
        let mut connection = self.conn().await?;
        let affected = diesel::delete(sentiment_results::table.find(prompt_id))
            .execute(&mut connection)
            .await?;
        Ok(affected > 0)
    }

    async fn list(&self) -> Result<Vec<Sentiment>, anyhow::Error> {
        let mut connection = self.conn().await?;
        let rows = sentiment_results::table
            .order(sentiment_results::created_at.desc())
            .select(SentimentRow::as_select())
            .load(&mut connection)
            .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}
