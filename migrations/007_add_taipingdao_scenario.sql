PRAGMA foreign_keys = ON;

INSERT INTO officer_tag_definitions
(id, category, label_zh, label_en, description, sort_order)
VALUES
('affiliation:yellow_turban', 'Affiliation', '黄巾军', 'Yellow Turban', 'Yellow Turban and Taiping Dao network', 113),
('context:taipingdao', 'Context', '太平道', 'Taiping Dao', 'Taiping Dao and Yellow Turban uprising context', 602);

INSERT INTO officer_tag_aliases (alias, tag_id) VALUES
('yellow_turban', 'affiliation:yellow_turban'),
('太平道', 'context:taipingdao');

INSERT INTO factions
(id, name, default_ruler_id, color_r, color_g, color_b)
VALUES
('yellow_turban', '黄巾军', 'ctk_5f20_89d2', 0.92, 0.72, 0.18);

INSERT INTO scenarios (id, name, year, month)
VALUES ('ad180', '光和三年 太平道将兴', 180, 1);

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, confidence, biography, notes)
VALUES
('bo_cai', '波才', NULL, '豫州颍川郡', 152, 184, 'Male', 72, 70, 58, 44, 61, 'Medium', '黄巾起义颍川方面渠帅，曾在长社一带与皇甫嵩、朱儁交战。', '黄巾补充人物；事迹见后汉书黄巾相关记载，生年为游戏平衡估算。'),
('ma_yuanyi', '马元义', NULL, '荆州南阳郡', 150, 184, 'Male', 55, 48, 72, 68, 64, 'Medium', '太平道骨干，往来京师联络内应，事泄后被捕处死，黄巾起事因此提前。', '黄巾补充人物；生年为游戏平衡估算。'),
('bu_ji', '卜己', NULL, '兖州东郡', 153, 184, 'Male', 66, 64, 52, 46, 55, 'Medium', '黄巾起义东郡方面渠帅，后为汉军讨平。', '黄巾补充人物；生年为游戏平衡估算。'),
('peng_tuo', '彭脱', NULL, '豫州汝南郡', 154, 184, 'Male', 68, 66, 50, 43, 57, 'Medium', '黄巾起义汝南方面渠帅，活动于豫州一带。', '黄巾补充人物；生年为游戏平衡估算。'),
('guan_hai', '管亥', NULL, '青州北海郡', 156, NULL, 'Male', 69, 75, 42, 34, 48, 'Low', '青州黄巾人物，演义中围攻北海，与孔融、关羽等故事相关。', '黄巾补充人物；演义和民间叙事色彩较强，按低置信度处理。'),
('zhang_mancheng', '张曼成', NULL, '荆州南阳郡', 151, 184, 'Male', 74, 68, 61, 57, 63, 'Medium', '黄巾起义南阳方面渠帅，号称神上使，曾攻杀南阳太守褚贡。', '黄巾补充人物；生年为游戏平衡估算。'),
('zhao_hong_yellow', '赵弘', NULL, '荆州南阳郡', 154, 184, 'Male', 67, 65, 49, 44, 52, 'Medium', '南阳黄巾将领，张曼成死后继续据宛抵抗汉军。', '黄巾补充人物；使用 zhao_hong_yellow 避免常见姓名冲突。'),
('han_zhong_yellow', '韩忠', NULL, '荆州南阳郡', 154, 184, 'Male', 66, 63, 50, 45, 54, 'Medium', '南阳黄巾将领，赵弘死后继续守宛。', '黄巾补充人物；使用 han_zhong_yellow 避免常见姓名冲突。'),
('sun_xia_yellow', '孙夏', NULL, '荆州南阳郡', 156, 184, 'Male', 61, 60, 46, 42, 50, 'Medium', '南阳黄巾将领，韩忠死后接续统众。', '黄巾补充人物；使用 sun_xia_yellow 避免常见姓名冲突。'),
('huang_shao', '黄邵', NULL, '豫州汝南郡', 156, 196, 'Male', 64, 62, 47, 40, 51, 'Low', '汝南黄巾余部人物，后在曹操平定汝南、颍川黄巾时败亡。', '黄巾补充人物；史料较简略，生年为游戏平衡估算。'),
('he_man', '何曼', NULL, '豫州汝南郡', 157, 196, 'Male', 58, 78, 35, 28, 45, 'Low', '黄巾余部武勇人物，演义中以力战形象出现。', '黄巾补充人物；演义色彩较强，按低置信度处理。'),
('pei_yuanshao', '裴元绍', NULL, '豫州汝南郡', 158, NULL, 'Male', 52, 66, 38, 31, 47, 'Low', '黄巾余部人物，演义中与周仓、赵云相关故事相连。', '黄巾补充人物；演义色彩较强，按低置信度处理。');

WITH yellow_turban_tags(officer_id, tag_id) AS (
    VALUES
    ('bo_cai', 'role:general'), ('bo_cai', 'affiliation:yellow_turban'), ('bo_cai', 'basis:history'), ('bo_cai', 'context:taipingdao'), ('bo_cai', 'source:source_backed'),
    ('ma_yuanyi', 'role:adviser'), ('ma_yuanyi', 'affiliation:yellow_turban'), ('ma_yuanyi', 'basis:history'), ('ma_yuanyi', 'context:taipingdao'), ('ma_yuanyi', 'source:source_backed'),
    ('bu_ji', 'role:general'), ('bu_ji', 'affiliation:yellow_turban'), ('bu_ji', 'basis:history'), ('bu_ji', 'context:taipingdao'), ('bu_ji', 'source:source_backed'),
    ('peng_tuo', 'role:general'), ('peng_tuo', 'affiliation:yellow_turban'), ('peng_tuo', 'basis:history'), ('peng_tuo', 'context:taipingdao'), ('peng_tuo', 'source:source_backed'),
    ('guan_hai', 'role:warrior'), ('guan_hai', 'affiliation:yellow_turban'), ('guan_hai', 'basis:romance'), ('guan_hai', 'context:taipingdao'), ('guan_hai', 'source:manual_curated'),
    ('zhang_mancheng', 'role:general'), ('zhang_mancheng', 'affiliation:yellow_turban'), ('zhang_mancheng', 'basis:history'), ('zhang_mancheng', 'context:taipingdao'), ('zhang_mancheng', 'source:source_backed'),
    ('zhao_hong_yellow', 'role:general'), ('zhao_hong_yellow', 'affiliation:yellow_turban'), ('zhao_hong_yellow', 'basis:history'), ('zhao_hong_yellow', 'context:taipingdao'), ('zhao_hong_yellow', 'source:source_backed'),
    ('han_zhong_yellow', 'role:general'), ('han_zhong_yellow', 'affiliation:yellow_turban'), ('han_zhong_yellow', 'basis:history'), ('han_zhong_yellow', 'context:taipingdao'), ('han_zhong_yellow', 'source:source_backed'),
    ('sun_xia_yellow', 'role:general'), ('sun_xia_yellow', 'affiliation:yellow_turban'), ('sun_xia_yellow', 'basis:history'), ('sun_xia_yellow', 'context:taipingdao'), ('sun_xia_yellow', 'source:source_backed'),
    ('huang_shao', 'role:general'), ('huang_shao', 'affiliation:yellow_turban'), ('huang_shao', 'basis:history'), ('huang_shao', 'context:taipingdao'), ('huang_shao', 'source:manual_curated'),
    ('he_man', 'role:warrior'), ('he_man', 'affiliation:yellow_turban'), ('he_man', 'basis:romance'), ('he_man', 'context:taipingdao'), ('he_man', 'source:manual_curated'),
    ('pei_yuanshao', 'role:general'), ('pei_yuanshao', 'affiliation:yellow_turban'), ('pei_yuanshao', 'basis:romance'), ('pei_yuanshao', 'context:taipingdao'), ('pei_yuanshao', 'source:manual_curated'),
    ('ctk_5f20_89d2', 'affiliation:yellow_turban'), ('ctk_5f20_89d2', 'context:taipingdao'),
    ('ctk_5f20_5b9d', 'affiliation:yellow_turban'), ('ctk_5f20_5b9d', 'context:taipingdao'),
    ('ctk_5f20_6881', 'affiliation:yellow_turban'), ('ctk_5f20_6881', 'context:taipingdao'),
    ('ctk_5f20_71d5', 'affiliation:yellow_turban'), ('ctk_5f20_71d5', 'context:taipingdao')
)
INSERT INTO officer_tags (officer_id, tag_id)
SELECT officer_id, tag_id
FROM yellow_turban_tags;

INSERT INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES
('bo_cai', 'manual_curated', '波才', '', 'Medium', '黄巾剧本补充'),
('ma_yuanyi', 'manual_curated', '马元义', '', 'Medium', '黄巾剧本补充'),
('bu_ji', 'manual_curated', '卜己', '', 'Medium', '黄巾剧本补充'),
('peng_tuo', 'manual_curated', '彭脱', '', 'Medium', '黄巾剧本补充'),
('guan_hai', 'manual_curated', '管亥', '', 'Low', '黄巾剧本补充'),
('zhang_mancheng', 'manual_curated', '张曼成', '', 'Medium', '黄巾剧本补充'),
('zhao_hong_yellow', 'manual_curated', '赵弘', '', 'Medium', '黄巾剧本补充'),
('han_zhong_yellow', 'manual_curated', '韩忠', '', 'Medium', '黄巾剧本补充'),
('sun_xia_yellow', 'manual_curated', '孙夏', '', 'Medium', '黄巾剧本补充'),
('huang_shao', 'manual_curated', '黄邵', '', 'Low', '黄巾剧本补充'),
('he_man', 'manual_curated', '何曼', '', 'Low', '黄巾剧本补充'),
('pei_yuanshao', 'manual_curated', '裴元绍', '', 'Low', '黄巾剧本补充');

INSERT INTO scenario_faction_states
(scenario_id, faction_id, exists_in_scenario, selectable, ruler_id)
SELECT
    'ad180',
    f.id,
    CASE WHEN f.id IN ('han_court', 'yellow_turban') THEN 1 ELSE 0 END,
    CASE WHEN f.id IN ('han_court', 'yellow_turban') THEN 1 ELSE 0 END,
    CASE
        WHEN f.id = 'han_court' THEN 'ctk_5218_5b8f'
        WHEN f.id = 'yellow_turban' THEN 'ctk_5f20_89d2'
        ELSE f.default_ruler_id
    END
FROM factions f;

INSERT INTO scenario_city_states
(scenario_id, city_id, faction_id, population, gold, food, troops, training, agriculture, commerce, defense, city_order, governor_id)
SELECT
    'ad180',
    c.id,
    CASE
        WHEN c.id IN ('ganling', 'zhongshan', 'runan') THEN 'yellow_turban'
        ELSE 'han_court'
    END,
    CAST((c.population_min + c.population_max) / 2 AS INTEGER),
    CASE WHEN c.id IN ('ganling', 'zhongshan', 'runan') THEN 420 + c.commerce_base ELSE 260 + c.commerce_base * 2 END,
    CASE WHEN c.id IN ('ganling', 'zhongshan', 'runan') THEN 900 + c.agriculture_base * 3 ELSE 700 + c.agriculture_base * 4 END,
    CASE WHEN c.id IN ('ganling', 'zhongshan', 'runan') THEN 3200 + c.strategic_rank * 450 ELSE 1200 + c.strategic_rank * 650 END,
    CASE WHEN c.id IN ('ganling', 'zhongshan', 'runan') THEN 48 ELSE 35 + c.strategic_rank * 4 END,
    c.agriculture_base,
    c.commerce_base,
    c.defense_base,
    CASE WHEN c.id IN ('ganling', 'zhongshan', 'runan') THEN 62 ELSE 70 END,
    CASE c.id
        WHEN 'ganling' THEN 'ctk_5f20_89d2'
        WHEN 'zhongshan' THEN 'ctk_5f20_5b9d'
        WHEN 'runan' THEN 'bo_cai'
        WHEN 'luoyang' THEN 'he_jin'
        ELSE NULL
    END
FROM cities c;

INSERT INTO scenario_diplomacy
(scenario_id, faction_a, faction_b, score, truce_until_turn)
VALUES
('ad180', 'han_court', 'yellow_turban', -60, NULL);

INSERT OR REPLACE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES
('yellow_180_zhang_jiao', 'ctk_5f20_89d2', 180, 1, 'ServeFaction', 'yellow_turban', 'ganling', 95, '太平道将兴剧本初始归属'),
('yellow_180_zhang_bao', 'ctk_5f20_5b9d', 180, 1, 'ServeFaction', 'yellow_turban', 'zhongshan', 88, '太平道将兴剧本初始归属'),
('yellow_180_zhang_liang', 'ctk_5f20_6881', 180, 1, 'ServeFaction', 'yellow_turban', 'ganling', 86, '太平道将兴剧本初始归属'),
('yellow_180_zhang_yan', 'ctk_5f20_71d5', 180, 1, 'ServeFaction', 'yellow_turban', 'zhongshan', 74, '太平道将兴剧本初始归属'),
('yellow_180_bo_cai', 'bo_cai', 180, 1, 'ServeFaction', 'yellow_turban', 'runan', 82, '太平道将兴剧本初始归属'),
('yellow_180_ma_yuanyi', 'ma_yuanyi', 180, 1, 'ServeFaction', 'yellow_turban', 'ganling', 80, '太平道将兴剧本初始归属'),
('yellow_180_bu_ji', 'bu_ji', 180, 1, 'ServeFaction', 'yellow_turban', 'ganling', 76, '太平道将兴剧本初始归属'),
('yellow_180_peng_tuo', 'peng_tuo', 180, 1, 'ServeFaction', 'yellow_turban', 'runan', 76, '太平道将兴剧本初始归属'),
('yellow_180_guan_hai', 'guan_hai', 180, 1, 'ServeFaction', 'yellow_turban', 'zhongshan', 72, '太平道将兴剧本初始归属'),
('yellow_180_zhang_mancheng', 'zhang_mancheng', 180, 1, 'ServeFaction', 'yellow_turban', 'runan', 80, '太平道将兴剧本初始归属'),
('yellow_180_zhao_hong', 'zhao_hong_yellow', 180, 1, 'ServeFaction', 'yellow_turban', 'runan', 75, '太平道将兴剧本初始归属'),
('yellow_180_han_zhong', 'han_zhong_yellow', 180, 1, 'ServeFaction', 'yellow_turban', 'runan', 75, '太平道将兴剧本初始归属'),
('yellow_180_sun_xia', 'sun_xia_yellow', 180, 1, 'ServeFaction', 'yellow_turban', 'runan', 73, '太平道将兴剧本初始归属'),
('yellow_180_huang_shao', 'huang_shao', 180, 1, 'ServeFaction', 'yellow_turban', 'runan', 72, '太平道将兴剧本初始归属'),
('yellow_180_he_man', 'he_man', 180, 1, 'ServeFaction', 'yellow_turban', 'runan', 70, '太平道将兴剧本初始归属'),
('yellow_180_pei_yuanshao', 'pei_yuanshao', 180, 1, 'ServeFaction', 'yellow_turban', 'zhongshan', 70, '太平道将兴剧本初始归属');
