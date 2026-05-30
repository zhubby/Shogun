ALTER TABLE officers ADD COLUMN gender TEXT NOT NULL DEFAULT 'Male' CHECK (gender IN ('Male', 'Female'));
ALTER TABLE officers ADD COLUMN biography TEXT NOT NULL DEFAULT '';
ALTER TABLE officer_life_events ADD COLUMN loyalty INTEGER CHECK (loyalty BETWEEN 1 AND 100);

CREATE TABLE officer_external_ids (
    officer_id TEXT NOT NULL REFERENCES officers(id) ON DELETE CASCADE,
    source TEXT NOT NULL,
    external_id TEXT NOT NULL,
    source_url TEXT NOT NULL DEFAULT '',
    confidence TEXT NOT NULL CHECK (confidence IN ('High', 'Medium', 'Low')),
    notes TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (officer_id, source, external_id)
);

CREATE TABLE officer_relationships (
    source_officer_id TEXT NOT NULL REFERENCES officers(id) ON DELETE CASCADE,
    target_officer_id TEXT NOT NULL REFERENCES officers(id) ON DELETE CASCADE,
    relationship_kind TEXT NOT NULL CHECK (relationship_kind IN (
        'RulerSubject',
        'ParentChild',
        'AdoptiveParentChild',
        'Spouse',
        'Sibling',
        'SwornSibling',
        'Enemy'
    )),
    confidence TEXT NOT NULL CHECK (confidence IN ('High', 'Medium', 'Low')),
    notes TEXT NOT NULL DEFAULT '',
    source TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (source_officer_id, target_officer_id, relationship_kind),
    CHECK (source_officer_id <> target_officer_id)
);

CREATE INDEX idx_officer_relationships_target ON officer_relationships(target_officer_id);
CREATE INDEX idx_officer_relationships_kind ON officer_relationships(relationship_kind);
CREATE INDEX idx_officer_external_ids_source ON officer_external_ids(source, external_id);

DELETE FROM officer_life_events WHERE officer_id LIKE 'supplemental_%';
DELETE FROM officers WHERE id LIKE 'supplemental_%';
