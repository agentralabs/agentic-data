# AgenticData — With vs Without

## Without AgenticData

| Task | What happens |
|------|-------------|
| "Parse this CSV" | pandas reads it. You get columns. No type inference. No quality score. |
| "What format is this?" | You guess from the extension. Wrong extension = wrong parser. |
| "Where did this data come from?" | Nobody knows. The ETL pipeline is a black box. |
| "Is this data clean?" | You check manually. Or you don't. Bad data flows downstream silently. |
| "Convert this to JSON" | You write a script. It loses types, nulls become empty strings. |
| "Find PII in this dataset" | Regex for emails and SSNs. Misses "Dr. Smith's patient on Oak St." |
| "What changed since Tuesday?" | You diff two exports manually. Fields that were added/removed are invisible. |
| "Query across 3 databases" | You build a data warehouse. It takes 6 months and $200K. |

## With AgenticData

| Task | What happens |
|------|-------------|
| "Parse this CSV" | Schema inferred with type detection (email, date, currency). Quality scored. |
| "What format is this?" | Auto-detected from content with 95%+ confidence across 16 formats. |
| "Where did this data come from?" | Full lineage chain: source → transforms → current state. Trust scored. |
| "Is this data clean?" | Quality score 0-100. Anomalies detected. Null spikes flagged. |
| "Convert this to JSON" | Semantic bridge preserves types, reports information loss. |
| "Find PII in this dataset" | Context-aware PII detection: email, phone, SSN, IP, credit card. |
| "What changed since Tuesday?" | Temporal query: snapshot any point in time, diff any two versions. |
| "Query across 3 databases" | Federated query. Data stays in place. Results arrive in milliseconds. |
