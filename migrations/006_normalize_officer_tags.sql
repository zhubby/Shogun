CREATE TABLE officer_tag_definitions (
    id TEXT PRIMARY KEY,
    category TEXT NOT NULL CHECK (category IN ('Role', 'Affiliation', 'Source', 'Batch', 'Basis', 'Region', 'Context')),
    label_zh TEXT NOT NULL,
    label_en TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE officer_tag_aliases (
    alias TEXT PRIMARY KEY,
    tag_id TEXT NOT NULL REFERENCES officer_tag_definitions(id) ON DELETE CASCADE
);

CREATE TABLE officer_tags (
    officer_id TEXT NOT NULL REFERENCES officers(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES officer_tag_definitions(id) ON DELETE CASCADE,
    PRIMARY KEY (officer_id, tag_id)
);

INSERT INTO officer_tag_definitions
(id, category, label_zh, label_en, description, sort_order)
VALUES
('role:ruler', 'Role', '君主', 'Ruler', 'Ruler or sovereign figure', 10),
('role:emperor', 'Role', '皇帝', 'Emperor', 'Imperial sovereign', 11),
('role:warlord', 'Role', '诸侯', 'Warlord', 'Independent regional warlord', 12),
('role:general', 'Role', '武将', 'General', 'Military commander or field officer', 20),
('role:warrior', 'Role', '猛将', 'Warrior', 'Warrior-focused military figure', 21),
('role:administrator', 'Role', '文官', 'Administrator', 'Civil administrator or office holder', 30),
('role:official', 'Role', '官员', 'Official', 'Court or local official', 31),
('role:adviser', 'Role', '谋士', 'Adviser', 'Strategist, adviser, or counselor', 40),
('role:diplomat', 'Role', '外交官', 'Diplomat', 'Diplomatic envoy or alliance broker', 41),
('role:scholar', 'Role', '学者', 'Scholar', 'Scholar, critic, or learned figure', 42),
('role:poet', 'Role', '诗人', 'Poet', 'Literary or poetic figure', 43),
('role:spouse', 'Role', '配偶', 'Spouse', 'Dynastic spouse or marriage-link figure', 50),
('role:empress', 'Role', '后妃', 'Empress', 'Empress, queen, or imperial consort', 51),
('affiliation:han_court', 'Affiliation', '汉室', 'Han court', 'Eastern Han court and imperial household', 100),
('affiliation:dong_zhuo', 'Affiliation', '董卓军', 'Dong Zhuo', 'Dong Zhuo faction and retainers', 101),
('affiliation:cao_wei', 'Affiliation', '曹魏', 'Cao Wei', 'Cao Cao, Wei, and Cao-Wei-aligned figures', 102),
('affiliation:shu_han', 'Affiliation', '蜀汉', 'Shu Han', 'Liu Bei, Shu, and Shu-Han-aligned figures', 103),
('affiliation:eastern_wu', 'Affiliation', '东吴', 'Eastern Wu', 'Sun clan, Wu, and Eastern-Wu-aligned figures', 104),
('affiliation:yuan_shao', 'Affiliation', '袁绍军', 'Yuan Shao', 'Yuan Shao faction and Hebei network', 105),
('affiliation:yuan_shu', 'Affiliation', '袁术军', 'Yuan Shu', 'Yuan Shu faction', 106),
('affiliation:liu_biao', 'Affiliation', '刘表军', 'Liu Biao', 'Liu Biao and Jingzhou network', 107),
('affiliation:liu_zhang', 'Affiliation', '刘璋军', 'Liu Zhang', 'Liu Zhang and Yi Province network', 108),
('affiliation:lu_bu', 'Affiliation', '吕布军', 'Lu Bu', 'Lu Bu faction', 109),
('affiliation:zhang_lu', 'Affiliation', '张鲁军', 'Zhang Lu', 'Zhang Lu and Hanzhong network', 110),
('affiliation:western_jin', 'Affiliation', '西晋', 'Western Jin', 'Jin transition and Western Jin figures', 111),
('affiliation:rebel', 'Affiliation', '起义军', 'Rebel forces', 'Rebel or uprising faction', 112),
('source:ctk_import', 'Source', 'CTK 导入', 'CTK import', 'Imported from Characters of the Three Kingdoms corpus', 200),
('source:source_backed', 'Source', '有来源', 'Source backed', 'Profile has an explicit source or curated backing', 201),
('source:manual_curated', 'Source', '人工校订', 'Manual curated', 'Manually curated historical profile', 202),
('batch:expansion_003', 'Batch', '扩充 003', 'Expansion 003', 'Officer added or revised in expansion migration 003', 300),
('basis:history', 'Basis', '正史', 'History', 'Historically grounded profile', 400),
('basis:tradition', 'Basis', '传说', 'Tradition', 'Traditional or folklore-derived profile', 401),
('basis:romance', 'Basis', '演义', 'Romance', 'Romance-derived profile', 402),
('region:northern', 'Region', '北方', 'Northern', 'Northern frontier or Hebei-Liaodong context', 500),
('region:liangzhou', 'Region', '凉州', 'Liangzhou', 'Liangzhou and northwestern context', 501),
('region:jingzhou', 'Region', '荆州', 'Jingzhou', 'Jingzhou context', 502),
('region:xuzhou', 'Region', '徐州', 'Xuzhou', 'Xuzhou context', 503),
('region:nanzhong', 'Region', '南中', 'Nanzhong', 'Nanzhong and southwestern frontier context', 504),
('context:coalition', 'Context', '讨董联盟', 'Anti-Dong coalition', 'Anti-Dong Zhuo coalition context', 600),
('context:jin_transition', 'Context', '魏晋过渡', 'Jin transition', 'Late Wei and Jin transition context', 601);

INSERT INTO officer_tag_aliases (alias, tag_id) VALUES
('ruler', 'role:ruler'),
('emperor', 'role:emperor'),
('warlord', 'role:warlord'),
('general', 'role:general'),
('warrior', 'role:warrior'),
('administrator', 'role:administrator'),
('official', 'role:official'),
('adviser', 'role:adviser'),
('diplomat', 'role:diplomat'),
('scholar', 'role:scholar'),
('poet', 'role:poet'),
('spouse', 'role:spouse'),
('empress', 'role:empress'),
('han_court', 'affiliation:han_court'),
('faction:东汉', 'affiliation:han_court'),
('dong_zhuo', 'affiliation:dong_zhuo'),
('cao_wei', 'affiliation:cao_wei'),
('魏', 'affiliation:cao_wei'),
('faction:魏', 'affiliation:cao_wei'),
('faction:魏国', 'affiliation:cao_wei'),
('shu_han', 'affiliation:shu_han'),
('蜀', 'affiliation:shu_han'),
('faction:蜀', 'affiliation:shu_han'),
('faction:蜀汉', 'affiliation:shu_han'),
('eastern_wu', 'affiliation:eastern_wu'),
('东吴', 'affiliation:eastern_wu'),
('faction:东吴', 'affiliation:eastern_wu'),
('faction:吴', 'affiliation:eastern_wu'),
('yuan_shaos', 'affiliation:yuan_shao'),
('yuan_shao', 'affiliation:yuan_shao'),
('yuan_shu', 'affiliation:yuan_shu'),
('faction:袁术', 'affiliation:yuan_shu'),
('jingzhou', 'region:jingzhou'),
('faction:刘璋割据军阀', 'affiliation:liu_zhang'),
('lu_bu', 'affiliation:lu_bu'),
('zhang_lu', 'affiliation:zhang_lu'),
('西晋', 'affiliation:western_jin'),
('faction:西晋', 'affiliation:western_jin'),
('faction:起义军', 'affiliation:rebel'),
('ctk_import', 'source:ctk_import'),
('source_backed', 'source:source_backed'),
('manual_curated', 'source:manual_curated'),
('expansion_003', 'batch:expansion_003'),
('history', 'basis:history'),
('tradition', 'basis:tradition'),
('romance', 'basis:romance'),
('northern', 'region:northern'),
('liangzhou', 'region:liangzhou'),
('xuzhou', 'region:xuzhou'),
('nanzhong', 'region:nanzhong'),
('coalition', 'context:coalition'),
('jin_transition', 'context:jin_transition');

WITH RECURSIVE split(officer_id, tag, rest) AS (
    SELECT id, '', tags || ','
    FROM officers
    UNION ALL
    SELECT
        officer_id,
        trim(substr(rest, 1, instr(rest, ',') - 1)),
        substr(rest, instr(rest, ',') + 1)
    FROM split
    WHERE rest <> ''
)
INSERT OR IGNORE INTO officer_tags (officer_id, tag_id)
SELECT split.officer_id, aliases.tag_id
FROM split
JOIN officer_tag_aliases aliases ON aliases.alias = split.tag
WHERE split.tag <> ''
  AND split.tag <> 'female';

CREATE INDEX idx_officer_tags_tag ON officer_tags(tag_id, officer_id);
CREATE INDEX idx_officer_tags_officer ON officer_tags(officer_id);
CREATE INDEX idx_officer_tag_definitions_category ON officer_tag_definitions(category, sort_order, id);

ALTER TABLE officers DROP COLUMN tags;
