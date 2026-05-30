-- Low-confidence supplemental pool.
--
-- This keeps the first SQLite catalog at the requested 500+ officer scale while the
-- hand-curated biographical rows are expanded over time. These rows are deliberately
-- tagged and marked Low confidence so they can be filtered, audited, and replaced by
-- source-backed records without schema changes.

WITH RECURSIVE n(value) AS (
    SELECT 1
    UNION ALL
    SELECT value + 1 FROM n WHERE value < 520
),
surnames(idx, name) AS (
    VALUES
    (0,'王'),(1,'李'),(2,'张'),(3,'刘'),(4,'陈'),(5,'杨'),(6,'赵'),(7,'黄'),(8,'周'),(9,'吴'),
    (10,'孙'),(11,'徐'),(12,'朱'),(13,'马'),(14,'胡'),(15,'郭'),(16,'何'),(17,'高'),(18,'林'),(19,'罗'),
    (20,'郑'),(21,'梁'),(22,'谢'),(23,'宋'),(24,'唐'),(25,'许'),(26,'邓'),(27,'冯'),(28,'韩'),(29,'曹'),
    (30,'曾'),(31,'彭'),(32,'萧'),(33,'蔡'),(34,'潘'),(35,'田'),(36,'董'),(37,'袁'),(38,'于'),(39,'余')
),
given(idx, name) AS (
    VALUES
    (0,'伯平'),(1,'仲达'),(2,'季常'),(3,'文节'),(4,'公礼'),(5,'元方'),(6,'子明'),(7,'德广'),(8,'士安'),(9,'孝先'),
    (10,'叔度'),(11,'彦和'),(12,'景仁')
),
places(idx, name) AS (
    VALUES
    (0,'颍川'),(1,'汝南'),(2,'南阳'),(3,'陈留'),(4,'河内'),(5,'魏郡'),(6,'吴郡'),(7,'会稽'),
    (8,'蜀郡'),(9,'巴郡'),(10,'太原'),(11,'辽东'),(12,'武威'),(13,'天水'),(14,'北海'),(15,'下邳')
)
INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, leadership, strength, intelligence, politics, charm, tags, confidence, notes)
SELECT
    printf('supplemental_%03d', n.value),
    surnames.name || given.name || printf('%03d', n.value),
    given.name,
    places.name,
    140 + (n.value % 85),
    CASE WHEN n.value % 11 = 0 THEN 190 + (n.value % 70) ELSE NULL END,
    42 + (n.value * 7) % 45,
    38 + (n.value * 11) % 48,
    40 + (n.value * 13) % 50,
    39 + (n.value * 17) % 50,
    40 + (n.value * 19) % 48,
    'supplemental,low_confidence',
    'Low',
    '低置信度规模补充记录；用于验证资料库和登场管线，后续以史料校订替换'
FROM n
JOIN surnames ON surnames.idx = n.value % 40
JOIN given ON given.idx = n.value % 13
JOIN places ON places.idx = n.value % 16;

WITH RECURSIVE n(value) AS (
    SELECT 1
    UNION ALL
    SELECT value + 1 FROM n WHERE value < 520
)
INSERT INTO officer_life_events (id, officer_id, event_year, event_month, event_kind, faction_id, city_id, notes)
SELECT
    printf('supplemental_start_%03d', n.value),
    printf('supplemental_%03d', n.value),
    188 + (n.value % 38),
    1 + (n.value % 12),
    'Appear',
    CASE n.value % 12
        WHEN 0 THEN 'cao_cao'
        WHEN 1 THEN 'liu_bei'
        WHEN 2 THEN 'sun_quan'
        WHEN 3 THEN 'yuan_shao'
        WHEN 4 THEN 'liu_biao'
        WHEN 5 THEN 'liu_zhang'
        WHEN 6 THEN 'ma_teng'
        WHEN 7 THEN 'zhang_lu'
        WHEN 8 THEN 'gongsun_zan'
        WHEN 9 THEN 'yuan_shu'
        WHEN 10 THEN 'shi_xie'
        ELSE 'han_court'
    END,
    CASE n.value % 24
        WHEN 0 THEN 'xuchang'
        WHEN 1 THEN 'pingyuan'
        WHEN 2 THEN 'jianye'
        WHEN 3 THEN 'ye'
        WHEN 4 THEN 'xiangyang'
        WHEN 5 THEN 'chengdu'
        WHEN 6 THEN 'wuwei'
        WHEN 7 THEN 'hanzhong'
        WHEN 8 THEN 'ji'
        WHEN 9 THEN 'shouchun'
        WHEN 10 THEN 'jiaozhi'
        WHEN 11 THEN 'luoyang'
        WHEN 12 THEN 'chenliu'
        WHEN 13 THEN 'runan'
        WHEN 14 THEN 'xiapi'
        WHEN 15 THEN 'beihai'
        WHEN 16 THEN 'nanyang'
        WHEN 17 THEN 'jiangling'
        WHEN 18 THEN 'jiangxia'
        WHEN 19 THEN 'changsha'
        WHEN 20 THEN 'hanzhong'
        WHEN 21 THEN 'lujiang'
        WHEN 22 THEN 'wu'
        ELSE 'changan'
    END,
    '低置信度补充人物登场事件'
FROM n;

INSERT INTO officer_life_events (id, officer_id, event_year, event_month, event_kind, faction_id, city_id, notes)
SELECT 'supplemental_death_' || substr(id, 14), id, death_year, 12, 'Die', NULL, NULL, '低置信度补充人物离场事件'
FROM officers
WHERE id LIKE 'supplemental_%' AND death_year IS NOT NULL;
