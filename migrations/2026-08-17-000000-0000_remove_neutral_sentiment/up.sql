CREATE TYPE sentiment_type_binary AS ENUM ('Positive', 'Negative');

ALTER TABLE sentiment_results
    ALTER COLUMN sentiment TYPE sentiment_type_binary
    USING sentiment::text::sentiment_type_binary;

DROP TYPE sentiment_type;
ALTER TYPE sentiment_type_binary RENAME TO sentiment_type;
