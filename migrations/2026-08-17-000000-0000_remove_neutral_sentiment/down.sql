CREATE TYPE sentiment_type_with_neutral AS ENUM ('Positive', 'Negative', 'Neutral');

ALTER TABLE sentiment_results
    ALTER COLUMN sentiment TYPE sentiment_type_with_neutral
    USING sentiment::text::sentiment_type_with_neutral;

DROP TYPE sentiment_type;
ALTER TYPE sentiment_type_with_neutral RENAME TO sentiment_type;
