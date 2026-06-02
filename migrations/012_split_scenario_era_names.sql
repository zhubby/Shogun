ALTER TABLE scenarios ADD COLUMN era_name TEXT NOT NULL DEFAULT '';

UPDATE scenarios
SET era_name = CASE id
        WHEN 'ad180' THEN '光和三年'
        WHEN 'ad190' THEN '初平元年'
        WHEN 'ad200' THEN '建安五年'
        WHEN 'ad208' THEN '建安十三年'
        WHEN 'ad220' THEN '延康元年'
        ELSE era_name
    END,
    name = CASE id
        WHEN 'ad180' THEN '太平道将兴'
        WHEN 'ad190' THEN '讨董余波'
        WHEN 'ad200' THEN '官渡前夜'
        WHEN 'ad208' THEN '赤壁前夜'
        WHEN 'ad220' THEN '魏晋序幕'
        ELSE name
    END
WHERE id IN ('ad180', 'ad190', 'ad200', 'ad208', 'ad220');
