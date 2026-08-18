# Sentiment prediction implementation notes

The Mermaid diagram source is in [`architecture-sequence.mmd`](architecture-sequence.mmd).

## Implemented persistence

- Successful predictions are stored in PostgreSQL with their source text, sentiment, probability, and creation time.
- `POST /predict` returns the persisted prediction ID.
- `GET /sentiments`, `GET /sentiments/{id}`, and `DELETE /sentiments/{id}` provide prediction history management.
- Pending migrations run during startup when `RUN_MIGRATIONS=true`.

## Remaining TODOs

- Return structured inference and configuration errors instead of only HTTP 500 responses.
- Uncomment and maintain the integration tests.
