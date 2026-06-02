PRAGMA foreign_keys = ON;

INSERT OR IGNORE INTO factions
(id, name, default_ruler_id, color_r, color_g, color_b)
VALUES
('liu_yan', '刘焉军', 'ctk_5218_7109', 0.22, 0.55, 0.35);

UPDATE scenario_faction_states
SET exists_in_scenario = CASE
        WHEN faction_id IN (
            'han_court',
            'yellow_turban',
            'dong_zhuo',
            'sun_quan',
            'gongsun_zan',
            'liu_yan',
            'liu_biao',
            'ma_teng',
            'shi_xie',
            'tao_qian'
        ) THEN 1
        ELSE 0
    END,
    selectable = CASE
        WHEN faction_id IN (
            'han_court',
            'yellow_turban',
            'dong_zhuo',
            'sun_quan',
            'gongsun_zan',
            'liu_yan',
            'liu_biao',
            'ma_teng',
            'shi_xie',
            'tao_qian'
        ) THEN 1
        ELSE 0
    END,
    ruler_id = CASE
        WHEN faction_id = 'han_court' THEN 'ctk_5218_5b8f'
        WHEN faction_id = 'yellow_turban' THEN 'ctk_5f20_89d2'
        WHEN faction_id = 'sun_quan' THEN 'sun_jian'
        WHEN faction_id = 'liu_yan' THEN 'ctk_5218_7109'
        ELSE ruler_id
    END
WHERE scenario_id = 'ad180';

INSERT OR IGNORE INTO scenario_faction_states
(scenario_id, faction_id, exists_in_scenario, selectable, ruler_id)
VALUES
('ad180', 'liu_yan', 1, 1, 'ctk_5218_7109');

UPDATE scenario_city_states
SET faction_id = CASE
        WHEN city_id IN ('ganling', 'zhongshan', 'runan') THEN 'yellow_turban'
        WHEN city_id IN ('anding', 'tianshui', 'longxi') THEN 'dong_zhuo'
        WHEN city_id IN ('wuwei', 'jiuquan', 'dunhuang') THEN 'ma_teng'
        WHEN city_id IN ('youbeiping', 'liaoxi', 'xiangping') THEN 'gongsun_zan'
        WHEN city_id IN (SELECT id FROM cities WHERE province = '徐州') THEN 'tao_qian'
        WHEN city_id IN (SELECT id FROM cities WHERE province = '荆州') THEN 'liu_biao'
        WHEN city_id IN (SELECT id FROM cities WHERE province = '益州') THEN 'liu_yan'
        WHEN city_id IN (SELECT id FROM cities WHERE province = '扬州') THEN 'sun_quan'
        WHEN city_id IN (SELECT id FROM cities WHERE province = '交州') THEN 'shi_xie'
        ELSE 'han_court'
    END,
    governor_id = CASE city_id
        WHEN 'ganling' THEN 'ctk_5f20_89d2'
        WHEN 'zhongshan' THEN 'ctk_5f20_5b9d'
        WHEN 'runan' THEN 'bo_cai'
        WHEN 'luoyang' THEN 'he_jin'
        WHEN 'anding' THEN 'dong_zhuo'
        WHEN 'wuwei' THEN 'ma_teng'
        WHEN 'youbeiping' THEN 'gongsun_zan'
        WHEN 'xiapi' THEN 'tao_qian'
        WHEN 'xiangyang' THEN 'liu_biao'
        WHEN 'chengdu' THEN 'ctk_5218_7109'
        WHEN 'wu' THEN 'sun_jian'
        WHEN 'jiaozhi' THEN 'shi_xie'
        ELSE governor_id
    END
WHERE scenario_id = 'ad180';

INSERT OR IGNORE INTO scenario_diplomacy
(scenario_id, faction_a, faction_b, score, truce_until_turn)
VALUES
('ad180', 'yellow_turban', 'liu_biao', -45, NULL),
('ad180', 'yellow_turban', 'tao_qian', -35, NULL),
('ad180', 'yellow_turban', 'gongsun_zan', -35, NULL),
('ad180', 'han_court', 'dong_zhuo', 20, NULL),
('ad180', 'han_court', 'liu_yan', 25, NULL),
('ad180', 'han_court', 'liu_biao', 25, NULL),
('ad180', 'han_court', 'ma_teng', 10, NULL),
('ad180', 'han_court', 'sun_quan', 15, NULL),
('ad180', 'han_court', 'shi_xie', 20, NULL);

INSERT OR REPLACE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES
('ad180_liu_yan_yizhou', 'ctk_5218_7109', 180, 1, 'ServeFaction', 'liu_yan', 'chengdu', 86, '太平道将兴剧本益州初始归属');
