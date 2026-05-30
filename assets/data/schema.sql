PRAGMA foreign_keys = ON;

CREATE TABLE cities (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    province TEXT NOT NULL,
    commandery TEXT NOT NULL,
    x REAL NOT NULL,
    y REAL NOT NULL,
    scale TEXT NOT NULL CHECK (scale IN ('County', 'Commandery', 'RegionalCapital', 'ImperialCapital')),
    strategic_rank INTEGER NOT NULL CHECK (strategic_rank BETWEEN 1 AND 10),
    agriculture_base INTEGER NOT NULL,
    commerce_base INTEGER NOT NULL,
    defense_base INTEGER NOT NULL,
    population_min INTEGER NOT NULL,
    population_max INTEGER NOT NULL,
    confidence TEXT NOT NULL CHECK (confidence IN ('High', 'Medium', 'Low')),
    notes TEXT NOT NULL DEFAULT ''
);

CREATE TABLE factions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    default_ruler_id TEXT NOT NULL,
    color_r REAL NOT NULL,
    color_g REAL NOT NULL,
    color_b REAL NOT NULL
);

CREATE TABLE officers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    courtesy_name TEXT,
    native_place TEXT,
    birth_year INTEGER,
    death_year INTEGER,
    leadership INTEGER NOT NULL CHECK (leadership BETWEEN 1 AND 100),
    strength INTEGER NOT NULL CHECK (strength BETWEEN 1 AND 100),
    intelligence INTEGER NOT NULL CHECK (intelligence BETWEEN 1 AND 100),
    politics INTEGER NOT NULL CHECK (politics BETWEEN 1 AND 100),
    charm INTEGER NOT NULL CHECK (charm BETWEEN 1 AND 100),
    tags TEXT NOT NULL DEFAULT '',
    confidence TEXT NOT NULL CHECK (confidence IN ('High', 'Medium', 'Low')),
    notes TEXT NOT NULL DEFAULT ''
);

CREATE TABLE roads (
    from_city_id TEXT NOT NULL REFERENCES cities(id),
    to_city_id TEXT NOT NULL REFERENCES cities(id),
    PRIMARY KEY (from_city_id, to_city_id),
    CHECK (from_city_id <> to_city_id)
);

CREATE TABLE scenarios (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    year INTEGER NOT NULL,
    month INTEGER NOT NULL CHECK (month BETWEEN 1 AND 12)
);

CREATE TABLE scenario_faction_states (
    scenario_id TEXT NOT NULL REFERENCES scenarios(id),
    faction_id TEXT NOT NULL REFERENCES factions(id),
    exists_in_scenario INTEGER NOT NULL CHECK (exists_in_scenario IN (0, 1)),
    selectable INTEGER NOT NULL CHECK (selectable IN (0, 1)),
    ruler_id TEXT NOT NULL REFERENCES officers(id),
    PRIMARY KEY (scenario_id, faction_id)
);

CREATE TABLE scenario_city_states (
    scenario_id TEXT NOT NULL REFERENCES scenarios(id),
    city_id TEXT NOT NULL REFERENCES cities(id),
    faction_id TEXT NOT NULL REFERENCES factions(id),
    population INTEGER NOT NULL,
    gold INTEGER NOT NULL,
    food INTEGER NOT NULL,
    troops INTEGER NOT NULL,
    training INTEGER NOT NULL CHECK (training BETWEEN 0 AND 100),
    agriculture INTEGER NOT NULL,
    commerce INTEGER NOT NULL,
    defense INTEGER NOT NULL,
    city_order INTEGER NOT NULL CHECK (city_order BETWEEN 0 AND 100),
    governor_id TEXT REFERENCES officers(id),
    PRIMARY KEY (scenario_id, city_id)
);

CREATE TABLE officer_life_events (
    id TEXT PRIMARY KEY,
    officer_id TEXT NOT NULL REFERENCES officers(id),
    event_year INTEGER NOT NULL,
    event_month INTEGER NOT NULL CHECK (event_month BETWEEN 1 AND 12),
    event_kind TEXT NOT NULL CHECK (event_kind IN ('Appear', 'ServeFaction', 'MoveToCity', 'BecomeUnavailable', 'Die')),
    faction_id TEXT REFERENCES factions(id),
    city_id TEXT REFERENCES cities(id),
    notes TEXT NOT NULL DEFAULT ''
);

CREATE TABLE scenario_diplomacy (
    scenario_id TEXT NOT NULL REFERENCES scenarios(id),
    faction_a TEXT NOT NULL REFERENCES factions(id),
    faction_b TEXT NOT NULL REFERENCES factions(id),
    score INTEGER NOT NULL CHECK (score BETWEEN -100 AND 100),
    truce_until_turn INTEGER,
    PRIMARY KEY (scenario_id, faction_a, faction_b),
    CHECK (faction_a <> faction_b)
);

CREATE INDEX idx_roads_from ON roads(from_city_id);
CREATE INDEX idx_roads_to ON roads(to_city_id);
CREATE INDEX idx_scenario_city_states_scenario ON scenario_city_states(scenario_id);
CREATE INDEX idx_scenario_city_states_faction ON scenario_city_states(faction_id);
CREATE INDEX idx_scenario_faction_states_scenario ON scenario_faction_states(scenario_id);
CREATE INDEX idx_officer_life_events_date ON officer_life_events(event_year, event_month);
CREATE INDEX idx_officer_life_events_officer ON officer_life_events(officer_id);
CREATE INDEX idx_officers_name ON officers(name);
CREATE INDEX idx_cities_province ON cities(province, commandery);
