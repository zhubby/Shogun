UPDATE officers SET gender = 'Male' WHERE gender NOT IN ('Male', 'Female');

PRAGMA foreign_keys = OFF;

CREATE TABLE officers_v3 (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    courtesy_name TEXT,
    native_place TEXT,
    birth_year INTEGER,
    death_year INTEGER,
    gender TEXT NOT NULL DEFAULT 'Male' CHECK (gender IN ('Male', 'Female')),
    leadership INTEGER NOT NULL CHECK (leadership BETWEEN 1 AND 100),
    strength INTEGER NOT NULL CHECK (strength BETWEEN 1 AND 100),
    intelligence INTEGER NOT NULL CHECK (intelligence BETWEEN 1 AND 100),
    politics INTEGER NOT NULL CHECK (politics BETWEEN 1 AND 100),
    charm INTEGER NOT NULL CHECK (charm BETWEEN 1 AND 100),
    tags TEXT NOT NULL DEFAULT '',
    confidence TEXT NOT NULL CHECK (confidence IN ('High', 'Medium', 'Low')),
    biography TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT ''
);

INSERT INTO officers_v3
SELECT id, name, courtesy_name, native_place, birth_year, death_year, gender,
       leadership, strength, intelligence, politics, charm, tags, confidence,
       biography, notes
FROM officers;

DROP TABLE officers;
ALTER TABLE officers_v3 RENAME TO officers;
CREATE INDEX idx_officers_name ON officers(name);

PRAGMA foreign_keys = ON;
