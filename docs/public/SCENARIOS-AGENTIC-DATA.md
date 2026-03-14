# AgenticData Scenarios

## Scenario 1: Enterprise Data Archaeology

**Situation:** A company has 200+ database tables. Original developers left. Documentation is wrong. Column "flag7" could mean anything.

**AgenticData solution:**
1. `data_soul_extract` → analyzes all tables, infers what each field represents
2. `data_soul_dictionary` → generates a complete data dictionary
3. `data_soul_relationships` → discovers foreign keys and cross-table relationships
4. `data_soul_rules` → extracts business rules from data patterns

**Result:** Complete understanding of legacy data in hours, not months.

## Scenario 2: GDPR Compliance Audit

**Situation:** Regulations require identifying and redacting all PII across 50 data sources.

**AgenticData solution:**
1. `data_federate_register` → register all 50 sources
2. `data_redact_detect` → scan every source for PII (email, phone, SSN, IP, names)
3. `data_redact_apply` → apply redaction policy (mask, hash, or synthetic replacement)
4. `data_redact_audit` → generate audit trail proving compliance

**Result:** Full PII inventory and automated redaction with audit proof.

## Scenario 3: Multi-Source Analytics

**Situation:** Marketing uses PostgreSQL, Sales uses Excel, Support uses JSON API. CEO wants a unified report.

**AgenticData solution:**
1. `data_federate_register` → register all three sources
2. `data_cross_discover` → auto-detect how datasets relate (customer IDs)
3. `data_cross_join` → federated join across all sources
4. `data_bridge_convert` → export to PDF report with charts

**Result:** Cross-source analytics without building a data warehouse.

## Scenario 4: Data Quality Monitoring

**Situation:** An ML pipeline silently ingests bad data. Model accuracy drops. Nobody notices for 3 weeks.

**AgenticData solution:**
1. `data_quality_score` → continuous quality scoring per dataset
2. `data_quality_anomaly` → automatic anomaly detection (null spikes, outliers)
3. `data_anomaly_subscribe` → alerts when quality drops below threshold
4. `data_predict_quality` → predict quality degradation before it happens

**Result:** Bad data caught in minutes, not weeks.

## Scenario 5: Format Migration

**Situation:** Legacy system exports data as XML. New system needs JSON. 50 entity types.

**AgenticData solution:**
1. `data_schema_infer` → infer schema from XML
2. `data_bridge_map` → define semantic mapping XML→JSON
3. `data_bridge_preview` → preview conversion on sample data
4. `data_bridge_loss` → report what information would be lost
5. `data_bridge_convert` → execute migration with full audit trail

**Result:** Lossless, auditable format migration.
