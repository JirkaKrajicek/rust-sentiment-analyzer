#[async_trait::async_trait]
impl ProjectRepository for PostgresStore {
    async fn insert(&self, prompt: String) -> Result<(), anyhow::Error> {
        // let mut conn = self.conn().await?;
        // Implement the logic to insert a new prompt into the database
        Ok(())
    }

    async fn get_sentiment(&self, prompt_id: Uuid) -> Result<Sentiment, anyhow::Error> {
        // Implement the logic to retrieve sentiment analysis result for a given prompt ID
        Ok(Sentiment::Neutral) // Placeholder return value
    }

    async fn delete(&self, prompt_id: Uuid) -> Result<(), anyhow::Error> {
        // Implement the logic to delete a prompt from the database
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Sentiment>, anyhow::Error> {
        // Implement the logic to list all prompts and their sentiment analysis results
        Ok(vec![]) // Placeholder return value
    }
}