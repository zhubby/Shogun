-- Keep the officer name column to the canonical display name.
-- Alternate names belong in profile text, not the table name used by UI lists.

UPDATE officers
SET name = '张道陵',
    biography = '五斗米道的创始人。初名张陵。张鲁祖父。'
WHERE id = 'ctk_5f20_9053_9675'
  AND name = '张道陵，初名张陵';
