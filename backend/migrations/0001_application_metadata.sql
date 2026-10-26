CREATE TABLE application_metadata (singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton), schema_generation INTEGER NOT NULL);
INSERT INTO application_metadata (singleton, schema_generation) VALUES (TRUE, 1);
