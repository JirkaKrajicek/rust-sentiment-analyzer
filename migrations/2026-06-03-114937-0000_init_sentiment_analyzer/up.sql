CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE sentiment_type AS ENUM ('Positive', 'Negative', 'Neutral');

CREATE TABLE sentiment_results (
    id          UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    input_text  TEXT        NOT NULL,
    sentiment   sentiment_type NOT NULL,
    probability DOUBLE PRECISION NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
