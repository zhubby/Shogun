-- Ensure runtime officer age can be derived for every historical profile.
-- Historically unknown birth years use a conservative adult fallback before the earliest scenario.

UPDATE officers
SET birth_year = 160
WHERE birth_year IS NULL;
