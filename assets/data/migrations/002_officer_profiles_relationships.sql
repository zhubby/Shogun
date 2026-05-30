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

-- Source-backed import seed generated from the MIT-licensed
-- fthux/Characters_of_the_Three_Kingdoms character JSON corpus.
--
-- Rebuild with:
--   rtk cargo run --bin import_three_kingdoms -- <source-characters-dir> assets/data/seeds/002_three_kingdoms_import.sql

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('liu_bei', '刘备', '玄德', '幽州涿郡涿县', 161, 223, 'Male', 61, 45, 63, 70, 74, 'ctk_import,faction:蜀汉,ruler,source_backed', 'High', '蜀汉的开国皇帝，相传是汉景帝之子中山靖王刘胜的后代。刘备少年丧父，与母亲贩鞋织草席为生。黄巾起义时，刘备组织义兵，随政府军剿除黄巾，有功，任安喜县尉，不久因鞭打督邮弃官。后诸侯割据，刘备势力弱小，经常寄人篱下，先后投靠过公孙瓒、曹操、袁绍、刘表等人，几经波折，却仍无自己的地盘。赤壁之战之际，刘备联吴抗曹，取得胜利，从东吴处“借”到荆州，迅速发展起来，吞并益州，占领汉中，建立蜀汉政权。后关羽战死，荆州被孙权夺取，刘备于称帝后伐吴，在夷陵之战中被陆逊击败，病逝于白帝城，临终托孤于诸葛亮。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 刘备.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('liu_bei', 'characters_of_the_three_kingdoms', '刘备', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/刘备.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5218_5b8f', '刘宏', NULL, '冀州河间国', 156, 189, 'Male', 58, 51, 79, 82, 61, 'ctk_import,faction:东汉,ruler,source_backed', 'High', '汉灵帝。听信宦官之言，将国家大权都交于宦官之手，使汉末百姓苦不堪言。中年爆发黄巾起义，国家一度陷入灭亡危机，后来黄巾起义被镇压。听信宦官，忠臣之言全然不听，致令不少忠心之士含冤而死或归隐山林。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 刘宏.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5218_5b8f', 'characters_of_the_three_kingdoms', '刘宏', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/刘宏.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5218_5b8f', 'ctk_5218_5b8f', 174, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5218_5ba0', '刘宠', '祖荣', '东莱郡牟平县', NULL, NULL, 'Male', 85, 67, 50, 90, 62, 'administrator,ctk_import,faction:东汉,general,source_backed', 'High', '以明经举孝廉，除东平陵令。母疾，弃官奉养。累迁至豫章太守、会稽太守。桓帝、灵帝时，四列九卿，两为司空，再迁司徒、太尉。任内，薄衣服，弊车马，无货积。往来京师，下道而过。为政简，除烦苛，禁察非法。以老病卒。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 刘宠.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5218_5ba0', 'characters_of_the_three_kingdoms', '刘宠', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/刘宠.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5218_5ba0', 'ctk_5218_5ba0', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5218_5cb1', '刘岱', '公山', '东莱郡牟平县', NULL, 192, 'Male', 69, 43, 67, 90, 87, 'administrator,ctk_import,faction:东汉,source_backed', 'High', '汉兖州刺史。初平元年，从袁绍起兵伐董卓。后与桥瑁借粮，桥瑁不借，刘岱怒而杀之，尽分其兵。后同王忠引兵五万，虚打曹公旗号，攻徐州。为张飞生擒，后为刘备释放。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 刘岱.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5218_5cb1', 'characters_of_the_three_kingdoms', '刘岱', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/刘岱.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5218_5cb1', 'ctk_5218_5cb1', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5218_7109', '刘焉', '君郎', '江夏郡竟陵县', NULL, 194, 'Male', 69, 64, 82, 78, 57, 'administrator,ctk_import,faction:东汉,source_backed', 'High', '少以宗室拜中郎，未几因事去官。后举贤良方正，辟司徒府，历洛阳令、冀州刺史、南阳太守、宗正、太常。灵帝末年，目睹政治腐败，天下将乱，求为外任，因之出为监军使者，领益州牧，封阳城侯。到任后，一方面镇压、招抚益州黄巾军；另一方面抑制地方豪强，先后杀州中大姓王咸、李权、贾龙等十余人。同时，招兵买马，大肆扩军。献帝初平四年，以长子刘范、次子刘诞为内应，与征西将军马腾合谋偷袭长安，除掉董卓余党李傕。结果，计事不密，范、诞被杀，偷袭失败。益州治所亦失火被焚，蓄积荡尽。焉既伤其子，亦恐妖灾，于次年病卒。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 刘焉.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5218_7109', 'characters_of_the_three_kingdoms', '刘焉', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/刘焉.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5218_7109', 'ctk_5218_7109', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5218_7426', '刘琦', NULL, '山阳郡高平县', NULL, 209, 'Male', 60, 49, 52, 72, 57, 'administrator,ctk_import,faction:东汉,source_backed', 'High', '荆州牧刘表的长子，知道自己会被后母和蔡瑁陷害，故此特意去找诸葛亮请教救命之计诸葛亮告诉他晋文公流亡外国保住性命的事例，教他出奔。随后，江夏太守黄祖战死，刘琦立刻自告奋勇请求担任江夏太守之职，成功逃过了后母和蔡瑁的陷害，并为刘备建立避战之所。赤壁之战后，刘备向朝廷上表，保举刘琦为荆州刺史，并以他的名义收复了荆南四郡。同年，刘琦因病逝世。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 刘琦.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5218_7426', 'characters_of_the_three_kingdoms', '刘琦', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/刘琦.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5218_7426', 'ctk_5218_7426', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('liu_yao', '刘繇', '正礼', '东莱郡牟平县', 156, 197, 'Male', 90, 49, 70, 72, 77, 'ctk_import,faction:东汉,general,source_backed', 'High', '汉扬州牧、振武将军。齐孝王少子封牟平侯，子孙家焉。繇伯父宠，为汉太尉。兄岱，历位侍中，兖州刺吏。繇年十九，从父韪为贼所劫质，繇篡取以归，由是显名。举孝廉，为郎中，除下邑长。时郡守以贵三国志卷戚讬之，遂弃官去。州辟部济南，济南相中常侍子，贪秽不循，繇奏免之。平原陶丘洪荐繇，欲令举茂才。会辟司空掾，除侍御史，不就。避乱淮浦，诏书以为扬州刺史。时袁术在淮南，繇畏惮，不敢之州。欲南渡江，吴景、孙贲迎置曲阿。术图为僭逆，攻没诸郡县。繇遣将屯江边以拒之。以景、贲术所授用，乃迫逐使去。於是术乃自置扬州刺史，与景、贲并力攻繇将，岁馀不下。汉命加繇为牧，振武将军，众数万人，孙策东渡，破之。繇奔丹徒，遂溯江南保豫章，驻彭泽。繇进讨笮融，为融所破，更复招合属县，攻破融。融败走入山，为民所杀，繇寻病卒，时年四十二。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 刘繇.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('liu_yao', 'characters_of_the_three_kingdoms', '刘繇', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/刘繇.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5218_8206', '刘舆', NULL, '东莱郡牟平县', NULL, 192, 'Male', 78, 43, 78, 90, 56, 'administrator,ctk_import,faction:东汉,source_backed', 'High', '《续汉书》曰：繇父舆，一名方，山阳太守。岱、繇皆有隽才。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 刘舆.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5218_8206', 'characters_of_the_three_kingdoms', '刘舆', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/刘舆.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5218_8206', 'ctk_5218_8206', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5218_865e', '刘虞', '伯安', '徐州东海郡郯', NULL, 193, 'Male', 74, 64, 72, 90, 85, 'administrator,ctk_import,faction:东汉,source_backed', 'High', '初举孝廉，累迁至幽州（旧治在今河北涿县）刺史。黄巾起义中，率兵镇压境内起义军张纯、张举部，因功拜太尉，封容丘侯。董卓秉政，授大司马，进封襄贲侯。史称其劝课农植，开渔盐之利。献帝初平二年（191），冀州刺史韩馥、勃海太守袁绍等欲立虞为帝，以抗董卓，虞严词拒绝。刘虞追求宽政，发展经济，安抚百姓，主张以怀柔政策对待少数民族，而下属公孙瓒主张武力解决，二人出现矛盾。后矛盾激化，刘虞率兵进攻公孙瓒，失败后被擒，不久，被杀。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 刘虞.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5218_865e', 'characters_of_the_three_kingdoms', '刘虞', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/刘虞.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5218_865e', 'ctk_5218_865e', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('liu_biao', '刘表', '景升', '山阳郡高平县', 142, 208, 'Male', 79, 73, 57, 82, 61, 'ctk_import,faction:东汉,general,source_backed', 'High', '姿貌温伟，少时知名于世，与七位贤士同号为「八俊」。为大将军何进辟为掾，出任北军中候。后代王睿为荆州刺史，用蒯氏兄弟、蔡瑁等人为辅。又进为镇南将军、荆州牧，封成武侯。在荆州期间，刘表恩威并着，招诱有方，万里肃清，群民悦服。又开经立学，爱民养士，从容自保。远交袁绍，近结张绣，内纳刘备，据地数千里，带甲十余万，称雄荆江，先杀江东孙坚，后又常抗曹操，是曹操强敌之一。然而刘表为人性多疑忌，好于坐谈，立意自守，而无四方之志，后更宠溺后妻蔡氏，使妻族蔡瑁等得权。刘表死后，蔡瑁等人废长立幼，奉表次子刘琮为主；曹操南征，刘琮举州以降，荆州遂没。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 刘表.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('liu_biao', 'characters_of_the_three_kingdoms', '刘表', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/刘表.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5218_9676', '刘陶', '子奇', '豫州颍川郡颍阴县', NULL, NULL, 'Male', 71, 72, 60, 86, 64, 'administrator,ctk_import,faction:东汉,source_backed', 'High', '刘陶于灵帝时任，为朝廷中谏议大夫。一日，帝在后院与十常侍饮宴，陶直径帝前大恸。帝问其故，陶曰：“天下危在旦夕，陛下尚自与阉宦共饮耶？”帝曰：“国家承平，有何危急？”陶曰：“四方盗贼并起，侵掠州郡。其祸皆由十常侍卖官害民，欺君罔上。朝廷正人皆去，祸在目前矣！”十常侍皆免冠跪伏于帝前曰：“大臣不相容，臣等不能活矣！愿乞性命归田里，尽将家产以助军资。”言罢痛哭。帝怒谓陶曰：“汝家亦有近侍之人，何独不容朕乎？”呼武士推出斩之。刘陶大呼：“臣死不惜！可怜汉室天下，四百余年，到此一旦休矣！”武士拥陶出，方欲行刑，一大臣喝住曰：“勿得下手，待我谏去。”众视之，乃司徒陈耽，径入宫中来谏帝曰：“刘谏议得何罪而受诛？”帝曰：“毁谤近臣，冒渎朕弓。”耽曰：“天下人民，欲食十常侍之肉，陛下敬之如父母，身无寸功，皆封列侯；况封胥等结连黄巾，欲为内乱： 陛下今不自省，社稷立见崩摧矣！”帝曰：“封胥作乱，其事不明。十常侍中，岂无一二忠臣？”陈耽以头撞阶而谏。帝怒，命牵出，与刘陶皆下狱。是夜，十常侍即于狱中谋杀之。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 刘陶.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5218_9676', 'characters_of_the_three_kingdoms', '刘陶', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/刘陶.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5218_9676', 'ctk_5218_9676', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5218_9676_32', '刘陶', '季治', '淮南成德', 255, NULL, 'Male', 59, 72, 60, 74, 54, 'administrator,ctk_import,faction:魏国,source_backed', 'High', '《三国志·刘晔传》裴松之注引《傅子》：陶字季冶，善名称，有大辩。曹爽时为选部郎，邓飏之徒称之以为伊吕。当此之时，其人意陵青云，谓玄曰：“仲尼不圣。何以知其然？智者图国；天下群愚，如弄一丸于掌中，而不能得天下。”玄以其言大惑，不复详难也。谓之曰：“天下之质，变无常也。今见卿穷！”爽之败，退居里舍，乃谢其言之过。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 刘陶2.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5218_9676_32', 'characters_of_the_three_kingdoms', '刘陶2', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/刘陶2.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5218_9676_32', 'ctk_5218_9676_32', 273, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('sun_quan', '孙权', '仲谋', '扬州吴郡富春县', 182, 252, 'Male', 60, 54, 50, 89, 81, 'ctk_import,faction:东吴,ruler,source_backed', 'High', '孙坚之子，孙策之弟。东汉建安五年，兄孙策病死，孙权继位吴侯、讨逆将军，领会稽太守，开始统领江东。他击败了黄祖。后东吴联合刘备，于赤壁击溃了曹操军。东吴后来又和曹操军在合肥附近鏖战，并从刘备手中夺回荆州、杀死关羽、大破刘备的讨伐军。曹丕称帝后孙权先向北方称臣，后自己建吴称帝，迁都建业。孙权称帝后曾大规模派人航海，加强对夷州的联系。又设置农官，实行屯田；并在山越地区设立郡县，促进了江南土地的开发。晚年的孙权日益骄奢，宠信吕壹，赋役繁重、刑罚残酷。立嗣之争，孙权也犯下极大错误，多数名臣死于非命。自孙登夭折后，孙权先是废了孙和，又赐死孙霸，最后立幼子孙亮，这为日后的吴宫政变埋下了祸根。孙权病逝后谥号大皇帝，史称东吴大帝。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 孙权.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('sun_quan', 'characters_of_the_three_kingdoms', '孙权', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/孙权.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_4e16_5e73', '张世平', NULL, '冀州中山国', NULL, NULL, 'Male', 68, 58, 66, 80, 53, 'ctk_import,faction:东汉,source_backed', 'High', '（刘备）少语言，善下人，喜怒不形於色。好交结豪侠，年少争附之。中山大商张世平、苏双等赀累千金，贩马周旋於涿郡，见而异之，乃多与之金财。先主由是得用合徒众。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张世平.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_4e16_5e73', 'characters_of_the_three_kingdoms', '张世平', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张世平.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_4e16_5e73', 'ctk_5f20_4e16_5e73', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_4e3e', '张举', NULL, '幽州渔阳郡', NULL, NULL, 'Male', 66, 62, 70, 70, 44, 'administrator,ctk_import,source_backed', 'High', '土豪。与张纯联合发动叛乱，自称「天子」。有武装9000人。和幽州牧刘虞等人的官军展开激战，见败势渐成定局，于是上吊而死。（资料取自后汉书，三国志中未记载此人。）', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张举.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_4e3e', 'characters_of_the_three_kingdoms', '张举', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张举.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_4e3e', 'ctk_5f20_4e3e', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('zhang_ren', '张任', NULL, '蜀郡成都', NULL, 214, 'Male', 69, 55, 64, 47, 61, 'ctk_import,faction:刘璋割据军阀,source_backed', 'High', '根据《三国志·先主传》的记载，张任是三国时期蜀郡人（现四川省成都市），出身寒门，年轻时就非常有胆略，有气节，刘备进攻蜀地时，张任是益州从事。刘璋遣张任、刘璝率精兵拒先主于涪城，建安十九年（214年）败退到雒城，刘备进军包围雒城，雁桥之战失败被擒。刘备想招降他，但是张任说：“老臣终不复事二主矣。”于是被杀，刘备也很惋惜。后人一般感叹张任的忠贞和勇气，自西晋以来修建有张任墓，即在今天的四川省广汉市', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张任.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('zhang_ren', 'characters_of_the_three_kingdoms', '张任', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张任.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_4f11', '张休', '叔嗣', '徐州彭城国', 205, 254, 'Male', 70, 72, 68, 50, 63, 'ctk_import,faction:东吴,general,source_backed', 'High', '张休刚刚成年时与诸葛恪、顾谭和陈表同为太子孙登的僚友，并以《汉书》传授太子，从中庶子转为右弼都尉。孙权当时时常狩猎，每每到黄昏才回来，张休上疏劝谏，孙权对张休的上疏很满意，并向其父张昭展示张休的上疏。孙登死后，张休被加为侍中，后拜羽林都督，平三典军事，又迁为扬武将军。后来，张休与顾谭的弟弟顾承一同跟随全琮与魏将王淩在芍陂交战，张休等人奋力迎击牵制了魏军，全琮的子侄全绪、全端等人趁机击退了王淩。此后论功行赏时，由于张休等人起到了牵制敌军的作用而获得了比全氏兄弟更大的官职，因此招致全琮的怨恨。当时太子孙和与鲁王孙霸的夺嫡之争正激烈，顾谭一族为太子孙和一党，而全琮一族属于鲁王孙霸一党，与顾氏关系亲密的张休自然也难逃全氏的攻击。于是全琮父子借着这次“芍陂论功”事件，趁机向孙权进谗言，称张休、顾承与典军陈恂有私下的往来，因此得到了更多的奖赏。245年（赤乌八年），张休、顾承、顾谭一起也因此被发配交州。中书令孙弘为人阴险狡诈，张休向来很讨厌他，此时孙弘见张休被处罚，便趁机再进谗言，张休遂被赐死，时年41岁。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张休.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_4f11', 'characters_of_the_three_kingdoms', '张休', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张休.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_4f11', 'ctk_5f20_4f11', 223, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_4fed', '张俭', '元节', '山阳郡高平县', 115, 198, 'Male', 75, 65, 74, 53, 57, 'ctk_import,faction:东汉,source_backed', 'High', '东汉时期名士，江夏八俊之一。因党锢之祸，被迫逃亡，人甚重之，众多门阀名士因收留他而获罪被杀，包含孔融之兄长孔褒。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张俭.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_4fed', 'characters_of_the_three_kingdoms', '张俭', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张俭.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_4fed', 'ctk_5f20_4fed', 133, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_5141', '张允', NULL, NULL, NULL, NULL, 'Male', 62, 39, 66, 50, 42, 'ctk_import,source_backed', 'High', '东汉末年荆州牧刘表的外甥，先后侍奉刘表、曹操。其母刘氏，是刘表的姊妹，故允应与蔡瑁有姻亲之谊。他与蔡瑁同样得宠于刘表，亦与刘表次子刘琮相睦。刘表及蔡氏欲以刘琮为后，而蔡瑁、张允则为其党羽。及至刘表病重，刘琦归看父疾，张允等人恐刘琦入见后父子相感，竟有托后之意，乃谓刘琦道：「将军（指刘表）命你抚临江夏，其任至重。如今你竟舍众擅来，将军必然见怒。伤亲之欢，只会重增其疾，实非孝敬之道。」遂阻遏刘琦于门户之外，使不得见。刘琦无奈，只能流涕而去，众人闻而感伤。蔡瑁、张允等便以刘琮为嗣。会曹操军至新野，张允亦随州而降。之后史书没有关于张允的记载。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张允.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_5141', 'characters_of_the_three_kingdoms', '张允', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张允.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_5141', 'ctk_5f20_5141', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_52cb', '张勋', NULL, NULL, NULL, NULL, 'Male', 63, 75, 69, 66, 42, 'ctk_import,faction:袁术,general,source_backed', 'High', '术称帝，为大将军。孙策投袁术，桥蕤、勋皆倾心敬焉。建安二年，术僭号，乃遣使以窃号告吕布，并为子娉布女。布执术使送许。术大怒，遣其将勋、桥蕤攻布，大败而还。曹操征术，术闻大骇，即走度淮，留张勋、桥蕤于蕲阳，以拒操。操击破斩蕤，共击勋等于下邳，大破之，而勋退走。术死，长史杨弘、大将勋等将其众欲就策，庐江太守刘勋要击，悉虏之，收其珍宝以归。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张勋.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_52cb', 'characters_of_the_three_kingdoms', '张勋', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张勋.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_52cb', 'ctk_5f20_52cb', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_534e', '张华', '茂先', '范阳方城', 232, 300, 'Male', 60, 39, 57, 45, 54, 'ctk_import,faction:西晋,source_backed', 'High', '晋国之臣。秘书丞。与皇帝司马炎下棋时，送来了讨伐吴国的奏章。因他表示赞成刚而强烈反对贾充的消极论，自此遭到贾充怨恨。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张华.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_534e', 'characters_of_the_three_kingdoms', '张华', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张华.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_534e', 'ctk_5f20_534e', 250, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_5357_31', '张南', NULL, NULL, NULL, NULL, 'Male', 79, 52, 59, 46, 55, 'ctk_import,general,source_backed', 'High', '袁熙部将，后与同僚焦触一起降曹。在赤壁之战被周泰所杀。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张南1.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_5357_31', 'characters_of_the_three_kingdoms', '张南1', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张南1.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_5357_31', 'ctk_5f20_5357_31', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_5357_32', '张南', '文进', NULL, NULL, 222, 'Male', 91, 52, 59, 58, 65, 'ctk_import,faction:蜀汉,general,source_backed', 'High', '张南与冯习一同从荆州跟随刘备入蜀，夷陵之战时为前部督，后蜀军为陆逊击破，张南和冯习都为吴军所斩。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张南2.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_5357_32', 'characters_of_the_three_kingdoms', '张南2', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张南2.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_5357_32', 'ctk_5f20_5357_32', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_536b', '张卫', '公则', '豫州沛国丰', NULL, NULL, 'Male', 70, 48, 63, 61, 67, 'ctk_import,faction:东汉,source_backed', 'High', '建安二十年（215年），丞相曹操征张鲁，七月，到阳平关，张鲁派五官掾前去举汉中投降，张卫不肯，和部将杨昂率众数万守关，横山筑城十馀里。曹操本来听信凉州从事和武都降人的话，以为张鲁易攻，但与张卫对峙三日，却不能攻下阳平山上诸屯，士卒多伤亡，粮食也将尽，于是决意退军，派大将军夏侯惇、将军许褚召回攻山的军队。但是曹操前军在夜间迷了路，误入了张卫的大营，当夜又有数千野麋踩坏张卫大营，于是张卫军退散。侍中辛毗、刘晔等在后，将此情报知夏侯惇、许褚，二将不信。夏侯惇亲自看到后，才告诉曹操，刘晔也建议曹操回攻，于是曹操派解𢢼、高祚等回击，斩将杨任，误遇张卫军，高祚等多鸣鼓角，张卫害怕，以为曹操大军到了，于是遁走（一作投降）。张鲁奔巴中，投', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张卫.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_536b', 'characters_of_the_three_kingdoms', '张卫', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张卫.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_536b', 'ctk_5f20_536b', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_5b9d', '张宝', NULL, '冀州钜鹿郡', NULL, 184, 'Male', 48, 48, 65, 69, 67, 'ctk_import,faction:起义军,source_backed', 'High', '东汉末农民起义领袖。汉灵帝时期在各地传播“太平道”，发展部众至几十万，遍及青、徐、幽、冀、荆、扬、兗、豫八州。184年二月，张角发动起义，打出了“苍天已死，黄天当立”的口号，矛头直指东汉政府，史称“黄巾起义”，影响遍及全国，极大地动摇了东汉政府的统治。同年十月，张角病死，不久其直接领导的黄巾军被镇压，但其后各地起义不断，最终导致东汉灭亡汉末农民起义首领。张角之弟。熹平年间，张角创太平道，宝与角同在河北一带传教，秘密进行组织工作。中平元年（184），发动黄巾起义，号称“地公将军”。张梁战败牺牲，宝率军在下曲阳继续抗击皇甫嵩，英勇战死，全军十余万人亦壮烈牺牲。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张宝.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_5b9d', 'characters_of_the_three_kingdoms', '张宝', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张宝.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_5b9d', 'ctk_5f20_5b9d', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_5cfb', '张峻', NULL, NULL, NULL, NULL, 'Male', 69, 59, 59, 91, 73, 'ctk_import,faction:蜀汉,source_backed', 'High', '蜀太常。蜀国投降后，邓艾命令太常张峻招安蜀国各郡军民。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张峻.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_5cfb', 'characters_of_the_three_kingdoms', '张峻', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张峻.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_5cfb', 'ctk_5f20_5cfb', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_5db7', '张嶷', '伯岐', '益州巴郡南充国', 194, 254, 'Male', 69, 62, 58, 66, 44, 'ctk_import,faction:蜀,general,source_backed', 'High', '嶷出自孤微，而少有通壮之节。弱冠为县功曹。先主定蜀之际，山寇攻县，县长捐家逃亡，嶷冒白刃，携负夫人，夫人得免。由是显名，州召为从事。时郡内士人龚禄、姚伷，当世有声名，皆与嶷友善。建兴五年，丞相亮北住汉中，广汉、绵竹山贼张慕等钞盗军资，劫掠吏民，嶷以都尉将兵讨之。拜为牙门将，属马忠，北讨汶山叛羌，南平四郡蛮夷，辄有筹画战克之功。越巂郡自丞相亮讨高定之后，叟夷数反，杀太守龚禄、焦璜，是后太守不敢之郡，除嶷为越巂太守，嶷将所领往之郡，诱以恩信，蛮夷皆服，颇来降附。嶷以功赐爵关内侯。在官三年，徙还故郡，缮治城郭，夷种男女莫不致力。郡有旧道，绝道已百馀年。嶷开通旧道，千里肃清，复古亭驿。后主於是加嶷怃戎将军，领郡如故。在郡十五年，邦域安穆。屡乞求还，乃徵诣成都。嶷至，拜荡寇将军，慷慨壮烈，士人咸多贵之，然放荡少礼，人亦以此讥焉，是岁延熙十七年也。魏狄道长李简密书请降，卫将军姜维率嶷等因简之资以出陇西。嶷时风湿固疾，至都浸笃，扶杖然后能起。姜维之出，时论以嶷初还，股疾不能在行中，由是嶷自乞肆力中原，致身敌庭。临发，辞后主曰：“臣当值圣明，受恩过量，加以疾病在身，常恐一朝陨没，辜负荣遇。天不违原，得豫戎事。若凉州克定，臣为籓表守将；若有未捷，杀身以报。”后主慨然为之流涕。军前与魏将徐质交锋，嶷临陈陨身，然其所杀伤亦过倍。既亡，封长子瑛西乡侯，次子护雄袭爵。南土越巂民夷闻嶷死，无不悲泣，为嶷立庙，四时水旱辄祀之。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张嶷.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_5db7', 'characters_of_the_three_kingdoms', '张嶷', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张嶷.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_5db7', 'ctk_5f20_5db7', 212, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_5e03', '张布', NULL, '江苏苏州市', NULL, 264, 'Male', 83, 52, 73, 77, 51, 'ctk_import,faction:吴,general,source_backed', 'High', '早在孙休为琅琊王时，张布为会稽王手下左右督将，与孙休相识。永安元年（258年），孙休被孙𬘭迎立为帝，时任长水校尉的张布因辅政勤劳，被任命为辅义将军，封永康侯。东吴自孙峻死后，朝政长期由孙峻的堂弟孙𬘭把持。孙休害怕孙𬘭图谋叛乱，与丁奉、张布合谋，在腊月的百官宴上诛杀孙𬘭，张布因功封左将军，后加任中军督。弟弟张惇为都亭侯，张恂为校尉。永安五年（262年）十月，张布因旧功获加封，主管宫中尚书、中书、门下[来源请求]各署。孙休死后，因蜀汉灭亡以及东吴交趾郡发生叛乱，东吴国内需要一位较年长的君主。在左典军万彧的极力推荐下，丞相濮阳兴、左将军张布说服朱太后，废孙休长子孙𩅦，改立乌程侯孙皓为帝，张布因拥立之功被封为骠骑将军加侍中。孙皓即位后粗暴骄横、嗜酒好色，濮阳兴和张布都暗自后悔拥立孙皓，被万彧秘密告发。元兴元年（264年）十一月，濮阳兴和张布被流放广州，流放途中被孙皓派人杀害，三族被灭。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张布.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_5e03', 'characters_of_the_three_kingdoms', '张布', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张布.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_5e03', 'ctk_5f20_5e03', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_5f25', '张弥', NULL, NULL, NULL, 233, 'Male', 53, 69, 61, 88, 42, 'administrator,ctk_import,faction:东吴,source_backed', 'High', '孙权派太常张弥、执金吾许晏、将军贺达领兵万人，金银财宝，九锡备物，渡海封公孙渊为燕王。但公孙渊怕孙权远水救不了近火，斩杀张弥、许晏等。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张弥.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_5f25', 'characters_of_the_three_kingdoms', '张弥', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张弥.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_5f25', 'ctk_5f20_5f25', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_5f53', '张当', NULL, NULL, NULL, 249, 'Male', 68, 49, 65, 45, 46, 'ctk_import,faction:魏,source_backed', 'High', '张当与曹爽亲善，任为都监。司马懿说他伺察皇帝的情况，挑拨离间皇帝和太后二宫的关系，伤害骨肉之情，天下动荡不安，人人心怀畏惧。嘉平元年（249年）正月初十，有司奏告“黄门张当私自把才人张氏、何氏等被张当送给曹爽。曹爽亦曾私取曹睿的才人七八人为乐伎，伪作诏书，发才人五十七人送邺台，使婕妤教习为伎。怀疑他们之间隐有奸谋。”于是逮捕了张当，交廷尉讯问。张当交待：“曹爽与尚书何晏、邓飏、丁谧，司隶校尉毕轨，荆州刺史李胜等人阴谋反叛，等到三月中旬起事”。朝廷逮捕曹爽、曹羲、曹训、何晏、邓飏、丁谧、毕轨、李胜以及桓范等人入狱，与张当一起都被诛灭三族。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张当.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_5f53', 'characters_of_the_three_kingdoms', '张当', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张当.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_5f53', 'ctk_5f20_5f53', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_608c', '张悌', '巨先', '荆州襄阳郡', NULL, 280, 'Male', 71, 49, 51, 49, 74, 'ctk_import,faction:东吴,source_backed', 'High', '少有名理，孙休时为屯骑校尉。魏伐蜀，悌论以“蜀阉宦专朝，国劳不备。必为魏所灭。”果如是。后拜丞相。晋伐吴，悌督沈莹、诸葛靓等率众三万渡江逆之。所领奋勇死战，无去意。军败，为王浑军所杀。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张悌.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_608c', 'characters_of_the_three_kingdoms', '张悌', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张悌.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_608c', 'ctk_5f20_608c', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_627f_31', '张承', '公先', '河内脩武', NULL, NULL, 'Male', 74, 40, 75, 41, 48, 'ctk_import,faction:魏国,source_backed', 'High', '正史《三国志》记载：魏国初建，承以丞相参军祭酒领赵郡太守，政化大行。太祖将西征，徵承参军事，至长安，病卒。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张承1.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_627f_31', 'characters_of_the_three_kingdoms', '张承1', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张承1.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_627f_31', 'ctk_5f20_627f_31', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_627f_32', '张承', '仲嗣', '徐州彭城郡彭城县', 177, 244, 'Male', 84, 50, 75, 41, 48, 'ctk_import,faction:东吴,general,source_backed', 'High', '张承年少时已以才学而知名，与诸葛瑾、步骘、严畯交好。孙权当骠骑将军时，辟他为西曹掾，出任长沙西部都尉。曾讨平山越，获得精兵一万五千人。后来又当濡须都督，奋威将军，封都乡侯，领私兵（部曲）五千人。张承能甄识人物，曾提拔彭城人蔡款和南阳人谢景，那时他们还很年少。后来他们皆被朝廷所用。诸葛瑾之子诸葛恪年轻时，因为他的才智令众人惊叹，但张承断言诸葛恪必定令诸葛家衰败。（后来诸葛恪被孙峻于政变中杀死，诛灭三族。）吴大帝赤乌七年（公元244年），张承逝世，谥定侯，享年六十七岁。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张承2.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_627f_32', 'characters_of_the_three_kingdoms', '张承2', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张承2.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_627f_32', 'ctk_5f20_627f_32', 195, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_65e2', '张既', '德容', '司隶冯翊高陵', NULL, 223, 'Male', 73, 49, 64, 54, 54, 'administrator,ctk_import,faction:魏,source_backed', 'High', '年十六为郡小吏。举茂才，除新丰令，治为三辅第一。既授命说马腾与繇会击高干、郭援叛，大破之。又会腾等败张晟等，斩卫固、张琰。召马腾于关中，定关西叛。为京兆尹抚民兴政。魏初，为尚书，出为雍州刺史。从征张鲁，说曹操拔汉中民实长安三辅。协曹洪破吴兰于下辩，又与夏侯渊平宋建，定临洮、狄道，安郡民，徙氐以利趋，拔汉中守。渔翁得利定凉州，兵行神速平胡乱。迁凉州刺史，封西乡侯。降苏衡反、邻戴众，修工事，抚民诛西平麹光。既政惠著闻，辟杨阜、胡遵等士，皆有名位，黄初四年薨。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张既.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_65e2', 'characters_of_the_three_kingdoms', '张既', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张既.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_65e2', 'ctk_5f20_65e2', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('zhang_zhao', '张昭', '子布', '徐州彭城', 156, 236, 'Male', 65, 49, 73, 55, 61, 'ctk_import,faction:东吴,general,source_backed', 'High', '三国时期孙吴重臣。东汉末年，张昭为避战乱而南渡至扬州。孙策创业时，任命其为长史、抚军中郎将，将文武之事都委任于张昭。孙策临死前，将其弟孙权托付给张昭，张昭率群僚辅立孙权，并安抚百姓、讨伐叛军，帮助孙权稳定局势。赤壁之战时，张昭持主降论。孙权代理车骑将军时，任命张昭为军师。孙权被封为吴王后，拜其为绥远将军，封由拳侯，此后曾参与撰定朝仪。孙权两次要设立丞相时，众人都推举张昭，孙权以张昭敢于直谏、性格刚直为由而不用他，先后用孙邵、顾雍。黄龙元年（229年），孙权称帝后，张昭以年老多病为由，上还官位及所统领部属，改拜辅吴将军、班亚三司，改封娄侯。晚年时一度不参与政事，在家著《春秋左氏传解》及《论语注》，今皆佚失。嘉禾五年（236年），张昭去世，年八十一，谥号“文”。张昭善隶书，其作品无存。唐张怀瓘在《书估》中将其书法列为第三等。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张昭.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('zhang_zhao', 'characters_of_the_three_kingdoms', '张昭', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张昭.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_6768', '张杨', '稚叔', '并州云中郡云中县', NULL, 198, 'Male', 72, 39, 66, 82, 64, 'administrator,ctk_import,faction:东汉,source_backed', 'High', '汉大司马、河内太守。以武勇给并州，为武猛从事。灵帝末，并州刺史丁原遣杨将兵诣蹇硕，为假司马。灵帝崩，硕为何进所杀。杨复为进所遣，归本州募兵，得千余人，因留上党，击山贼。及董卓作乱，杨遂以所将攻上党太守于壶关，不下，略诸县，众至数千人。袁绍至河内，杨与绍合，复与匈奴单于于夫罗屯漳水。单于欲叛，绍、杨不从。单于执杨至黎阳，卓以杨为建义将军、河内太守。天子之在河东，杨将兵至安邑，拜安国将军，封晋阳侯。杨欲迎天子还洛，诸将不听，杨还野王。建安元年，杨奉、董承、韩暹挟天子还旧京，粮乏。杨以粮迎道路，遂至洛阳，还野王。即拜为大司马。杨素与吕布善。太祖之围布，杨欲救之，不能。乃出兵东市，遥为之势。其将杨丑，杀杨以应太祖。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张杨.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_6768', 'characters_of_the_three_kingdoms', '张杨', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张杨.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_6768', 'ctk_5f20_6768', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('zhang_song', '张松', '子乔', '蜀郡成都', NULL, 212, 'Male', 51, 46, 65, 63, 56, 'ctk_import,faction:刘璋割据军阀, 蜀,source_backed', 'High', '刘璋的部下，益州别驾，为人短小，放荡不治节操，然而很有才干；他认为刘璋暗弱，在他手下不足以发挥自己的才能，经常叹息。赤壁之战前夕，张松奉命出使结交曹操，不被礼遇，因此怀恨曹操，劝刘璋改为结交刘备。进而，张松与好友法正一同密谋出卖刘璋，将益州献给刘备，劝说刘璋迎接刘备入蜀。后来，刘备假意离开益州，张松写信劝阻，被兄长张肃发现并告发，被杀。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张松.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('zhang_song', 'characters_of_the_three_kingdoms', '张松', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张松.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_zhang_song', 'zhang_song', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_6881', '张梁', NULL, '冀州钜鹿郡', NULL, 184, 'Male', 62, 63, 79, 60, 43, 'ctk_import,faction:起义军,source_backed', 'High', '光和七年（184年）随兄起事，太平道徒众称为“人公将军”。遭到朝廷所派左中郎将皇甫嵩进攻时，张梁率三万大军于广宗（今河北威县）进行反击。因疏忽，遭到汉军夜袭，兵败身亡，本部士兵皆被坑杀。至此，黄巾起义宣告失败。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张梁.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_6881', 'characters_of_the_three_kingdoms', '张梁', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张梁.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_6881', 'ctk_5f20_6881', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_6a2a', '张横', NULL, NULL, NULL, NULL, 'Male', 65, 64, 50, 53, 56, 'ctk_import,faction:东汉,source_backed', 'High', '韩遂麾下“旗本八骑”之一，协助马超复仇而借兵出战。却于渭水之战掉落陷马坑战死。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张横.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_6a2a', 'characters_of_the_three_kingdoms', '张横', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张横.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_6a2a', 'ctk_5f20_6a2a', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_6b66', '名不详', NULL, NULL, NULL, NULL, 'Male', 86, 48, 76, 56, 60, 'ctk_import,faction:东汉,source_backed', 'High', '张武是历史原型为不知所名的三国时期江夏贼二头领，出自三国演义。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张武.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_6b66', 'characters_of_the_three_kingdoms', '张武', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张武.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_6b66', 'ctk_5f20_6b66', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_6d25', '张津', '子云', '荆州南阳郡', NULL, NULL, 'Male', 84, 39, 56, 90, 53, 'administrator,ctk_import,faction:东汉,source_backed', 'High', '出身于荆州南阳郡的东汉末期官员，官至交州牧[2][注 1]。灵帝驾崩之后，在京城生活的他，成为让何进和袁绍在未来规划除掉宦官过程的关键人物。接着，受遣到京城以外地带，先后担任交阯刺史和交州牧。最后，被部领杀害而死。关于正史记载方面，《三国志》和《后汉书》皆描述其生平，却无单篇为他而撰的传记。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张津.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_6d25', 'characters_of_the_three_kingdoms', '张津', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张津.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_6d25', 'ctk_5f20_6d25', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_6d4e', '张济', NULL, '凉州武威郡租厉', NULL, 196, 'Male', 61, 62, 58, 76, 47, 'ctk_import,general,source_backed', 'High', '董卓的部下，被派往陈留、颍川等地劫掠。董卓被杀后，张济归无所依，于是伙同李傕、郭汜、樊稠等原董卓部曲将攻向长安，击败吕布，杀死王允等人，占领长安。张济外出屯于弘农。后诸将不和，李傕在会议上杀死了樊稠，又与郭汜分别劫持了汉献帝和公卿，相互交战，张济率兵赶来和解，于是二人罢兵，李傕出屯池阳，郭汜、张济等人随汉献帝东归前往弘农。后来，李傕、郭汜、张济反悔，联合起来追击汉献帝，与杨奉、董承等人几番交战。汉献帝一路逃亡，狼狈不堪，到达安邑，与李傕等人讲和。不久，汉献帝被曹操迎往许都。张济带兵从关中进入荆州地界，攻穰城，中流矢而死。张济的侄子张绣接管了部队，与刘表联合，屯于宛城。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张济.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_6d4e', 'characters_of_the_three_kingdoms', '张济', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张济.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_6d4e', 'ctk_5f20_6d4e', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_6e29_31', '张温', '伯慎', '南阳郡穰县', NULL, NULL, 'Male', 65, 55, 64, 79, 77, 'ctk_import,faction:东汉,source_backed', 'High', '东汉司空。率领孙坚、董卓等前去镇压边章、韩遂在凉州发动的叛乱。韩遂害怕张温的大军，暂时归顺。董卓专权时，因与袁术谋诛董卓，信件误下吕布处。于宴席间为卓所擒，旋即斩首。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张温1.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_6e29_31', 'characters_of_the_three_kingdoms', '张温1', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张温1.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_6e29_31', 'ctk_5f20_6e29_31', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_6e29_32', '张温', '惠恕', '吴郡吴县', 193, 230, 'Male', 63, 65, 70, 77, 67, 'administrator,ctk_import,faction:东吴,general,source_backed', 'High', '东吴孙权的幕僚，刘备死后出使蜀国，由于辩驳不过秦宓，遂遭降职。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张温2.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_6e29_32', 'characters_of_the_three_kingdoms', '张温2', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张温2.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_6e29_32', 'ctk_5f20_6e29_32', 211, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_71d5', '张燕', NULL, '常山郡真定', NULL, NULL, 'Male', 73, 64, 62, 41, 45, 'ctk_import,faction:魏国,general,source_backed', 'High', '魏平北将军。本姓褚。黄巾起，燕合聚少年为群盗，在山泽间转攻，还真定，众万余人。博陵张牛角亦起众，自号将兵从事，与燕合。燕推牛角为帅，俱攻瘿陶。牛角为飞矢所中。被创且死，令众奉燕，告曰：“必以燕为帅。”牛角死，众奉燕，故改姓张。燕剽捍捷速过人，故军中号曰飞燕。其后人众寝广，常山、赵郡、中山、上党、河内诸山谷皆相通，其小帅孙轻、王当等，各以部众从燕，众至百万，号曰黑山。灵帝不能征，河北诸郡被其害。燕遣人至京都乞降，拜燕平难中郎将。是后，董卓迁天子于长安，天下兵数起，燕遂以其众与豪杰相结。袁绍与公孙瓒争冀州，燕遣将杜长等助瓒，与绍战，为绍所败，人众稍散，太祖将定冀州，燕遣使求佐王师，拜平北将军；率众诣鄴，封安国亭侯，邑五百户。燕薨，子方嗣。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张燕.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_71d5', 'characters_of_the_three_kingdoms', '张燕', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张燕.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_71d5', 'ctk_5f20_71d5', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_723d', '张爽', NULL, NULL, NULL, NULL, 'Male', 74, 53, 45, 56, 54, 'ctk_import,faction:蜀,source_backed', 'High', '张爽，建安末年，为刘备劝学从事，参与劝备登帝位者之列。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张爽.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_723d', 'characters_of_the_three_kingdoms', '张爽', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张爽.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_723d', 'ctk_5f20_723d', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_7279', '张特', '子产', '涿郡', NULL, NULL, 'Male', 82, 48, 63, 75, 59, 'administrator,ctk_import,faction:魏,general,source_backed', 'High', '先时领牙门，给事镇东诸葛诞，诞不以为能，欲遣还护军。会毌丘俭代诞，使特屯守合肥新城。嘉平五年，吴诸葛恪围城，特与将军乐方军众合三千人固守。死者过半，城将陷，张诈言欲降，吴不攻。特乃夜彻诸屋材栅，补其缺为二重。次日复守。吴大怒攻之，不拔，遂引去。朝廷嘉之，加杂号将军，封列侯，迁安丰太守。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张特.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_7279', 'characters_of_the_three_kingdoms', '张特', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张特.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_7279', 'ctk_5f20_7279', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_7ea6', '张约', NULL, NULL, NULL, NULL, 'Male', 68, 72, 72, 70, 55, 'ctk_import,faction:吴,general,source_backed', 'High', '孙亮时，为散骑常侍。建兴二年（253），武卫将军孙峻谋杀诸葛恪，张约有所觉察，与朱恩密告告恪，恪未果断离开殿堂。孙峻刀斫恪之时，张约从一旁砍峻，伤左手，孙峻顺手斫约，断右臂。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张约.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_7ea6', 'characters_of_the_three_kingdoms', '张约', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张约.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_7ea6', 'ctk_5f20_7ea6', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('zhang_hong', '张纮', '子纲', '徐州广陵郡射阳', 153, 212, 'Male', 66, 65, 74, 71, 56, 'ctk_import,faction:东吴,source_backed', 'High', '孙策、孙权谋士。游学京都，还本郡，举茂才，公府辟，皆不就。避难江东。孙策创业，投策。表为正议校尉。从讨丹杨。谏策勿身临行陈。建安四年，纮奉章至许都，留为侍御史。谏曹公不以策薨伐吴。曹公从其言。出纮为会稽东部都尉。权以为长史，从征合肥。谏孙权勿亲临突击。次年，劝止吴军征合肥。建议宜出都秣陵，权从之。还吴迎家，途病卒。纮好文学，著诗赋铭诔十馀篇。书法亦为人所称道。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张纮.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('zhang_hong', 'characters_of_the_three_kingdoms', '张纮', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张纮.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_7eaf', '张纯', NULL, '幽州渔阳郡', NULL, 189, 'Male', 67, 65, 80, 81, 56, 'administrator,ctk_import,source_backed', 'High', '东汉的中山太守。联合乌丸族的丘力居发动了叛乱。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张纯.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_7eaf', 'characters_of_the_three_kingdoms', '张纯', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张纯.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_7eaf', 'ctk_5f20_7eaf', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_7ecd', '张绍', NULL, '幽州涿郡', NULL, NULL, 'Male', 66, 40, 48, 86, 58, 'administrator,ctk_import,faction:蜀, 魏, 西晋,source_backed', 'High', '张绍，蜀汉侍中、尚书仆射。父亲张飞，车骑将军。景耀六年，魏国大兴徒众，征西将军邓艾兵临成都。后主遣私署侍中张绍、光禄大夫谯周、驸马都尉邓良奉赍印缓，请命告诚，敬输忠款，存亡敕赐，惟所裁之。绍、良与邓艾相遇於雒县。邓艾得书，大喜，立即报书，遣绍、良先还。及归魏国，得封列侯。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张绍.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_7ecd', 'characters_of_the_three_kingdoms', '张绍', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张绍.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_7ecd', 'ctk_5f20_7ecd', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_7edf', '张统', NULL, '并州雁门郡马邑县', NULL, NULL, 'Male', 53, 46, 46, 78, 60, 'ctk_import,source_backed', 'High', '张辽之孙，其父张虎去世后，张统继承了晋阳侯的爵位。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张统.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_7edf', 'characters_of_the_three_kingdoms', '张统', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张统.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_7edf', 'ctk_5f20_7edf', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_7ee3', '张绣', NULL, '凉州武威郡祖厉县', NULL, 207, 'Male', 67, 58, 47, 79, 60, 'ctk_import,faction:魏国,general,source_backed', 'High', '曹操之破羌将军。族父济，汉骠骑将军。边章、韩遂为乱凉州，金城麹胜袭杀祖厉长刘隽。绣为县吏，间伺杀胜，郡内义之。遂招合少年，为邑中豪杰。董卓败，济与李傕等击吕布，为卓报仇。绣随济，以军功稍迁至建忠将军，封宣威侯。济为流矢所中死，绣领其众，屯宛，与刘表合。太祖南征，军淯水，绣等举众降。太祖纳济妻，绣恨之。太祖闻其不悦，密有杀绣之计。计漏，绣掩袭太祖。太祖军败，二子没。绣还保穰，太祖比年攻之，不克。太祖拒袁绍于官渡，绣从贾诩计，复以众降。绣至，太祖执其手，与欢宴，为子均取绣女，拜扬武将军。官渡之役，绣力战有功，迁破羌将军。从破袁谭于南皮，复增邑。从征乌丸于柳城，未至。五官将丕数因请会，发怒曰：“君杀吾兄，何忍持面视人邪！”绣心不自安，乃自杀。谥曰定侯，子泉嗣。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张绣.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_7ee3', 'characters_of_the_three_kingdoms', '张绣', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张绣.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_7ee3', 'ctk_5f20_7ee3', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_7f09', '张缉', '敬仲', '陕西西安市高陵县西南', NULL, 254, 'Male', 64, 60, 60, 54, 64, 'administrator,ctk_import,faction:魏,source_backed', 'High', '张缉继承了父亲张既的爵位，西乡侯。在太和年间，张缉曾任温县县令，因其管治能力而有名气。当时正值蜀汉丞相诸葛亮领兵北伐，张缉上书献计，魏明帝曹叡于是询问中书令孙资的意见，孙资则认为张缉有谋略。明帝于是命张缉为骑都尉，参与对蜀作战。战后改任尚书郎，因称职而获明帝所认识。后张缉以中书郎迁任东莞郡（今山东省沂水县东北）太守，期间多次向朝廷分析与东吴和蜀汉的形势。嘉平四年（252年），曹芳立张缉之女张氏为皇后，张缉任光禄大夫，位特进。不久曹魏进攻东吴，被东吴太傅诸葛恪于东兴击败；但此时张缉却向大将军司马师认为诸葛恪功高震主，不久必会被诛杀。次年诸葛恪围攻合肥，遇疫病被逼撤军后果然被孙峻杀死。司马师于是赞叹张缉比诸葛恪更聪明。张缉与中书令李丰是世交和同乡，彼此住处亦相近，李丰于是联合张缉和夏侯玄，打算推翻司马师，改以夏侯玄为大将军。张缉因在朝中不得意，而李丰掌握权力，彼此亦是同乡，他的儿子李韬又娶了齐长公主，于是听信。嘉平六年（254年），李丰设下伏兵打算诛杀司马师，以夏侯玄为大将军，张缉为骠骑将军，但司马师听到风声，召见李丰并将李丰杀害，及后又收捕夏侯玄和张缉等人，收归廷尉审理。最终张缉在狱中被赐死，诸子也被杀，但未被灭族，其孙张殷在西晋年间为梁州刺史。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张缉.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_7f09', 'characters_of_the_three_kingdoms', '张缉', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张缉.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_7f09', 'ctk_5f20_7f09', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_7ffc', '张翼', '伯恭', '益州犍为郡武阳县', NULL, 264, 'Male', 69, 81, 71, 89, 50, 'administrator,ctk_import,faction:蜀,general,source_backed', 'High', '三国时期蜀汉将领。历任梓潼、广汉、蜀郡三郡太守，出任庲降都督，后随诸葛亮和姜维北伐，官至左车骑将军，领冀州刺史。初封关内侯，进爵都亭侯。蜀汉灭亡后，魏将钟会密谋造反，成都大乱，张翼亦为乱兵所杀。张翼是蜀汉第三任庲降都督，由于执法严厉，不得南夷欢心。在北伐上，张翼认为国小民疲，不应滥用武力，是蜀汉朝廷当时极少敢当朝和姜维争辩北伐问题的大臣。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张翼.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_7ffc', 'characters_of_the_three_kingdoms', '张翼', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张翼.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_7ffc', 'ctk_5f20_7ffc', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_8083', '张肃', '君矫', '蜀郡成都', NULL, NULL, 'Male', 70, 46, 60, 68, 62, 'administrator,ctk_import,faction:刘璋割据军阀,source_backed', 'High', '张肃，字君矫，张松的兄长，蜀郡乃至益州世家大族，益州别驾从事，刘璋的部下。长得很伟岸，气度威严。曾奉命出使结交曹操，被辟为丞相府椽，拜广汉太守。后来，他发现弟弟张松密谋卖主，联络刘备，害怕牵连自己，于是告发，张松因此被杀。刘备入蜀后曾效命于帐下，后弃用。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张肃.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_8083', 'characters_of_the_three_kingdoms', '张肃', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张肃.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_8083', 'ctk_5f20_8083', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_82de', '张苞', NULL, '幽州涿郡涿县', NULL, NULL, 'Male', 53, 72, 60, 64, 45, 'ctk_import,faction:蜀,source_backed', 'High', '张苞，字兴国，三国时代的蜀汉将领张飞的长子，早殁。正史上对于张苞几乎没有任何其它记述。因张飞西乡侯爵位传于次子张绍，张苞很可能死于父亲张飞之前。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张苞.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_82de', 'characters_of_the_three_kingdoms', '张苞', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张苞.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_82de', 'ctk_5f20_82de', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_82f1', '张英', NULL, NULL, NULL, NULL, 'Male', 82, 54, 64, 67, 47, 'ctk_import,general,source_backed', 'High', '扬州牧刘繇部将。繇遣英、樊能屯江边以拒之。以吴景、孙贲为袁术所授用，乃迫逐使去。于是术乃自置扬州刺史，与景、贲并力攻英、能等，岁余不下。汉命加繇为牧，振武将军，众数万人，孙策东渡，破英、能等。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张英.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_82f1', 'characters_of_the_three_kingdoms', '张英', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张英.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_82f1', 'ctk_5f20_82f1', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_8302', '张茂', '彦林', '治今安徽省亳州市', NULL, NULL, 'Male', 58, 49, 68, 70, 48, 'ctk_import,faction:魏,source_backed', 'High', '青龙年间，魏明帝大兴土木，兴建宫室，太子舍人张茂以孙吴、蜀汉多次兴兵，诸将出征，上书劝谏，反对将士女许嫁为吏民为妻之人，再次还配于士。汉武帝好神仙，信方士，掘地为海，封土为山，当时天下为一，没有敢与争锋之人。汉朝衰乱四五十年以来，马不舍鞍，士不释甲。皇帝不兢兢业业，念崇节约，以奢靡为务，炫燿后园，建承露之盘。请求皇帝振作，克灭蜀吴。自己时年五十，常恐至死无以报国。魏明帝对说左右：“张茂自恃是我们曹家的同乡。”以事付于散骑。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张茂.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_8302', 'characters_of_the_three_kingdoms', '张茂', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张茂.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_8302', 'ctk_5f20_8302', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_8457', '张著', NULL, NULL, NULL, NULL, 'Male', 68, 68, 60, 78, 43, 'ctk_import,faction:蜀,general,source_backed', 'High', '建安二十四年（219年），刘备与丞相曹操争夺汉中，曹操运米到北山下，有数千万囊。刘备部下征西将军黄忠以为可取，率赵云所部士兵去取米。过了约定的期限，黄忠还没回来，赵云率数十骑出营接应黄忠，正逢曹操军大出，赵云被其前锋攻击，正战斗时，曹操大军到了，赵云被迫且战且退。曹操军战败又复合围攻赵云，赵云突围。张著在作战中受伤，赵云又驰马回营接应张著。曹军追杀到赵云营寨，此时沔阳长张翼在赵云营内，想闭门拒守，赵云入营后却大开营门，偃旗息鼓。曹操军疑其有伏兵而退去，又被赵云军所射，惊骇，自相践踏，多堕汉水中而死。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张著.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_8457', 'characters_of_the_three_kingdoms', '张著', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张著.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_8457', 'ctk_5f20_8457', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_864e', '张虎', NULL, '并州雁门郡马邑县', NULL, NULL, 'Male', 75, 61, 46, 70, 53, 'ctk_import,general,source_backed', 'High', '东汉末至三国时代曹魏将领张辽之子。黄初三年（222年）其父张辽病逝于江都，张虎奉诏继袭父爵，最后官至偏将军。张虎死后，其子张统继袭其爵。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张虎.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_864e', 'characters_of_the_three_kingdoms', '张虎', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张虎.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_864e', 'ctk_5f20_864e', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_8861', '张衡', '平子', '南阳西鄂', 78, 139, 'Male', 81, 53, 83, 88, 75, 'administrator,ctk_import,faction:东汉,source_backed', 'High', '“五斗米道”创立者张陵之子。陵死，衡行其道。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张衡.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_8861', 'characters_of_the_three_kingdoms', '张衡', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张衡.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_8861', 'ctk_5f20_8861', 96, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_88d4', '张裔', '君嗣', NULL, 165, 230, 'Male', 70, 66, 62, 42, 76, 'ctk_import,faction:蜀,general,source_backed', 'High', '刘璋任益州牧时，张裔被举为孝廉，为鱼复长，后来任从事，领帐下司马。刘璋和刘备反目后，刘备部下将领张飞率军由荆州经垫江入益州，刘璋给张裔军队，于德阳陌下抵挡张飞，张裔被打败，撤回成都。刘备兵围成都后，张裔当刘璋的使节去见刘备，刘备承诺礼待刘璋及好好安置他，最后刘璋决定开城投降。刘备入主益州后，任命张裔为巴郡太守，后又任命为司金中郎将，负责制作农战的器具。益州郡发生叛乱，太守正昂被杀，耆帅雍闿叛蜀附吴。此时蜀汉政府任命张裔为益州太守，张裔亦奉命上任。雍闿就将张裔送至东吴。刘备死后，诸葛亮奉遗命主政，并派邓芝到东吴重修吴蜀关系，并且要求归还张裔。孙权当时未知张裔，于是答应。临行前孙权和张裔谈话，孙权大为赏识，他乘船走后，孙权后悔并派人追他，但张裔已入蜀汉境内。回蜀汉后，诸葛亮命他为参军，署丞相府事，又领益州治中从事。诸葛亮出驻汉中时，张裔又以射声校尉兼领留府长史。一次张裔要到汉中咨询诸葛亮，有数百人送行，马车泊得满街都是。后来加辅汉将军，继续领留府长史。建兴八年（230年）逝世。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张裔.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_88d4', 'characters_of_the_three_kingdoms', '张裔', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张裔.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_88d4', 'ctk_5f20_88d4', 183, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_89d2', '张角', NULL, '冀州钜鹿郡', NULL, 184, 'Male', 66, 71, 74, 79, 63, 'ctk_import,faction:起义军,source_backed', 'High', '东汉末农民起义领袖。汉灵帝时期在各地传播“太平道”，发展部众至几十万，遍及青、徐、幽、冀、荆、扬、兗、豫八州。184年二月，张角发动起义，打出了“苍天已死，黄天当立”的口号，矛头直指东汉政府，史称“黄巾起义”，影响遍及全国，极大地动摇了东汉政府的统治。同年十月，张角病死，不久其直接领导的黄巾军被镇压，但其后各地起义不断，最终导致东汉灭亡。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张角.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_89d2', 'characters_of_the_three_kingdoms', '张角', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张角.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_89d2', 'ctk_5f20_89d2', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_8ba9', '张让', NULL, '豫州颍川郡', 135, 189, 'Male', 84, 53, 55, 78, 80, 'ctk_import,faction:东汉,source_backed', 'High', '宦官。少给事者中，桓帝时为小黄门。灵帝时，让、赵忠并迁中常侍，封列侯，与曹节、王甫等相为表里。让有监奴典任家事，交通货赂，威形喧赫。是时，让、忠及夏恽、郭胜、孙璋、毕岚、栗嵩、段珪、高望、张恭、韩悝、宋典十二人，皆为中常侍，封侯贵宠，父兄子弟布列州郡，所在贪贱，为人蠹害。黄巾既作，盗贼糜沸，郎中张钧上书斩十常侍。让等诬奏钧学黄巾道，收掠死狱中。而让等实多与张角交通。明年，南宫灾。让、忠等说帝令敛天下田亩税十钱，以修宫室。有钱不毕者，或至自杀。其守清者，乞不之官，皆迫遣之。又造万金堂于西园，引司农金钱缯帛，仞积其中。宦者得志，无所惮畏，并起第宅，拟则宫室。明年，遂使钩盾令宋典缮修南宫玉堂。六年，帝崩。中军校尉袁绍说大将军何进，令诛中官以悦天下。谋泄，让、忠等因进入省，遂共杀进。而绍勒兵斩忠，捕宦官无少长悉斩之。让等数十人劫质天子走河上。追急，让等悲哭辞曰：“臣等殄灭，天下乱矣。惟陛下自爱！”皆投河而死。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张让.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_8ba9', 'characters_of_the_three_kingdoms', '张让', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张让.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_8ba9', 'ctk_5f20_8ba9', 153, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_8c61', '张象', NULL, NULL, NULL, NULL, 'Male', 80, 54, 53, 52, 50, 'ctk_import,faction:东吴, 西晋,general,source_backed', 'High', '吴游击将军。天纪四年，晋伐吴，吴主孙晧遣象帅舟师万人御之，象众望旗而降。时晋龙骧将军王濬兵甲满江，旌旗烛天，军势甚盛，吴人大惧。（资料取自晋书，三国志未记载其人。）', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张象.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_8c61', 'characters_of_the_three_kingdoms', '张象', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张象.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_8c61', 'ctk_5f20_8c61', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_8d85', '张超', '孟高', '兖州东平郡寿张县', NULL, 195, 'Male', 52, 71, 74, 54, 77, 'ctk_import,faction:魏国,source_backed', 'High', '张超为广陵郡太守，征用名士臧洪、袁绥等。初平元年（190年）正月，与兄张邈会同其他诸侯参加讨伐董卓同盟。张超推荐臧洪为诸侯同盟宣誓者。兴平元年（194年）夏，曹操讨陶谦，远征徐州。张超、张邈和陈宫共谋推戴吕布为兖州牧，攻打曹操的根据地兖州。兴平二年（195年）春，曹操回军，吕布渐渐处于劣势。同年八月，张超在兄长命令下保家族守雍丘笼城，曹操猛攻雍丘。十二月，雍丘陷落，张超自杀（一说被曹操捕杀）。张邈、张超三族被灭。张邈向袁术求救途中，被部下杀害。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张超.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_8d85', 'characters_of_the_three_kingdoms', '张超', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张超.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_8d85', 'ctk_5f20_8d85', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('zhang_liao', '张辽', '文远', '并州雁门郡马邑县', 170, 222, 'Male', 72, 61, 73, 55, 57, 'ctk_import,general,source_backed', 'High', '以武力过人被召为从事。先后从属丁原、董卓、吕布，及至吕布败后才归于曹操。从曹操以后，辽随军征讨，多有战功，与关羽解白马之围，围降昌豨于东海，攻袁尚于邺城，斩乌丸单于蹋顿于白狼山，又讨平梅成、陈兰贼寇。赤壁战后，曹操留张辽、乐进、李典等守合肥，以御孙权。后孙权果引十万大军入寇，张辽率八百步兵冲阵让孙权军丧失士气，后孙权退兵，张辽观察后令军断桥，追击，差点活捉孙权。威震江东，江东儿啼不肯止者，其父母以辽名吓之。被曹操拜为征东将军。曹丕称帝仍令张辽守御孙权。张辽屯雍丘，却在此得病。但张辽不负众望，以抱病之躯击退吴将吕范。同年（黄初三年），张辽病逝于江都，谥曰刚侯。后人根据三国志记载“布为李傕所败，从布东奔徐州，领鲁相，时年二十八。”一句，多推测张辽生于169年，此说存疑，只能确定其生年范围为169-171年。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张辽.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('zhang_liao', 'characters_of_the_three_kingdoms', '张辽', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张辽.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_8fbe', '张达', NULL, NULL, NULL, NULL, 'Male', 73, 61, 74, 55, 57, 'ctk_import,faction:蜀,东吴,general,source_backed', 'High', '史书对其二人杀张飞的经过描述非常简略。起因可能是张飞刑罚过严，又常虐待下属，还将这些人留在左右，以往刘备常劝张飞这样会惹祸上身，但张飞不听从。章武元年秋七月，刘备讨伐东吴之时，张飞率军从阆中出发，准备前往江州。在出发前，部下张达、范彊杀死了张飞，并带着张飞的首级投奔孙权。[1]之后下落史书记载阙如。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张达.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_8fbe', 'characters_of_the_three_kingdoms', '张达', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张达.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_8fbe', 'ctk_5f20_8fbe', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_9053_9675', '张道陵，初名张陵', '辅汉', '东汉沛国丰县', 34, 156, 'Male', 54, 49, 71, 45, 46, 'ctk_import,source_backed', 'High', '五斗米道的创始人。张鲁祖父。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张道陵.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_9053_9675', 'characters_of_the_three_kingdoms', '张道陵', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张道陵.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_9053_9675', 'ctk_5f20_9053_9675', 52, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_9075', '张遵', NULL, '幽州涿郡', NULL, 263, 'Male', 60, 42, 78, 51, 73, 'administrator,ctk_import,faction:蜀,source_backed', 'High', '张遵，张飞之孙、张苞之子，官至蜀尚书，随诸葛瞻与邓艾在绵竹交战，战死。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张遵.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_9075', 'characters_of_the_three_kingdoms', '张遵', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张遵.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_9075', 'ctk_5f20_9075', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_9088', '张邈', '孟卓', '兖州东平郡寿张县', NULL, 195, 'Male', 48, 48, 82, 54, 75, 'administrator,ctk_import,faction:魏国,source_backed', 'High', '东汉末年陈留太守，汉末群雄之一，曾参与讨伐董卓。在汴水之战后归附曹操。此前因为与袁绍有隙，又曾与吕布交往，袁绍几次叫曹操杀张邈，但曹操都未听从，跟张邈更为亲近。兴平元年（194年），曹操带兵讨伐陶谦时，张邈与陈宫叛曹迎吕布为兖州牧。后吕布被曹操击败，张邈跟随吕布投奔刘备，全家及弟弟张超都被曹操杀于雍丘。张邈在向袁术借兵的路上，被部下所杀。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张邈.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_9088', 'characters_of_the_three_kingdoms', '张邈', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张邈.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_9088', 'ctk_5f20_9088', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('zhang_he', '张郃', '儁乂', '冀州河间国鄚', NULL, 231, 'Male', 55, 78, 50, 53, 44, 'ctk_import,faction:魏国,general,source_backed', 'High', '征西车骑将军。先从韩馥，后投袁绍，在与公孙瓒的交战中多有功劳。官渡之战时，张郃受郭图陷害，率众投降于曹操，得以重用，随曹操平定北方，远征乌丸，平马超，斩宋建，灭张鲁，多有战功。后来，张郃随夏侯渊驻守汉中，在夏侯渊被杀后暂代主帅，维持败兵。魏明帝时诸葛亮第一次北伐，张郃奉命救援陇右，在街亭大败蜀将马谡，导致诸葛亮撤兵；诸葛亮第四次时，张郃随司马懿前往相拒。后诸葛亮粮尽退兵，张郃追至木门，与诸葛亮军交战，被飞矢射中右膝而亡。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张郃.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('zhang_he', 'characters_of_the_three_kingdoms', '张郃', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张郃.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_94a7', '张钧', NULL, '冀州中山国', NULL, 184, 'Male', 61, 50, 59, 87, 73, 'ctk_import,faction:东汉,source_backed', 'High', '中平元年夏四月，黄巾既作，盗贼糜沸，钧上书言宜斩十常侍，县头南郊，以谢百姓，又遣使者布告天下，可不须师旅，而大寇自消。天子以钧章示让等，皆免冠徒跣顿首，乞自致洛阳诏狱，并出家财以助军费。帝怒钧曰：“此真狂子也。十常侍固当有一人善者不？”钧复重上，犹如前章，辄寝不报。诏使廷尉、侍御史考为张角道者，御史承让等旨，遂诬奏钧学黄巾道，收掠死狱中。（资料取自后汉书，三国志中未记载此人。）', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张钧.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_94a7', 'characters_of_the_three_kingdoms', '张钧', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张钧.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_94a7', 'ctk_5f20_94a7', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_95ff', '张闿', NULL, '冀州', NULL, NULL, 'Male', 52, 60, 52, 44, 58, 'ctk_import,source_backed', 'High', '陶谦帐下都尉。太祖父嵩在泰山华县。太祖令泰山太守应劭送家诣兖州，辎重百余辆。陶谦遣闿将骑二百卫送，闿于泰山华、费间杀嵩，取财物，因奔淮南。后投奔淮南的袁术，并奉袁术之命假装路过陈国，并刺杀了当时的陈王刘宠和陈国国相骆俊。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张闿.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_95ff', 'characters_of_the_three_kingdoms', '张闿', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张闿.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_95ff', 'ctk_5f20_95ff', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_97f3', '张音', NULL, NULL, NULL, NULL, 'Male', 68, 51, 76, 87, 78, 'ctk_import,faction:东汉,source_backed', 'High', '汉帝以众望在魏，乃召群公卿士，告祠高庙。使兼御史大夫张音持节奉玺绶禅位，册曰：“咨尔魏王：昔者帝尧禅位於虞舜，舜亦以命禹，天命不于常，惟归有德。汉道陵迟，世失其序，降及朕躬，大乱兹昏，群凶肆逆，宇内颠覆。赖武王神武，拯兹难於四方，惟清区夏，以保绥我宗庙，岂予一人获乂，俾九服实受其赐。今王钦承前绪，光于乃德，恢文武之大业，昭尔考之弘烈。皇灵降瑞，人神告徵，诞惟亮采，师锡朕命，佥曰尔度克协于虞舜，用率我唐典，敬逊尔位。於戏，天之历数在尔躬，允执其中，天禄永终；君其祗顺大礼，飨兹万国，以肃承天命。”', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张音.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_97f3', 'characters_of_the_three_kingdoms', '张音', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张音.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_97f3', 'ctk_5f20_97f3', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('zhang_fei', '张飞', '益德', '幽州涿郡', NULL, 221, 'Male', 84, 70, 47, 69, 53, 'ctk_import,faction:蜀,general,source_backed', 'High', '车骑将军、司隶校尉。年少时与关羽投靠刘备，三人恩如兄弟。刘备被公孙瓒表为平原相后刘备以其为別部司马。袁术讨刘备，刘备以张飞守下邳，但他与陶谦旧部曹豹发生冲突，招致丹杨兵开门引吕布破徐州。娶十三、四岁的夏侯氏为妻。曹操降荊州后引骑追击，刘备于长阪败逃，张飞引二十余骑，拆水断桥，终退曹军。从征赤壁和南郡之战，后被封为宜都太守、征虏将军、新亭侯。入蜀讨刘璋，于江州擒巴郡严颜，再定德阳、巴西等地。刘备得益州后赐张飞金五百斤，银千斤，钱五千万，锦千匹，兼领巴西太守。引军迂回至张郃军背后，于瓦口击败他。汉中之战时屯下辩，但友军吴兰为曹洪所破，被迫退走。刘备为汉中王，拜为右将军，假节。刘备称帝，封张飞为车骑将军，兼任司隶校尉，进封为西乡侯。刘备准备伐吴，张飞本率万人从中汇合，但为帐下将张达、范强所杀，他们持其首顺流而奔孙权。谥号为桓侯。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张飞.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('zhang_fei', 'characters_of_the_three_kingdoms', '张飞', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张飞.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('zhang_lu', '张鲁', '公祺，一说字公旗', '沛国丰县', NULL, 216, 'Male', 92, 65, 43, 54, 76, 'ctk_import,faction:东汉,general,source_backed', 'High', '祖父陵，客蜀，创五斗米教。陵死，子衡行其道。衡死，鲁复行之。益州牧刘焉以鲁为督义司马，与别部司马张修将兵击汉中太守苏固，鲁遂袭修杀之，夺其众。焉死，子璋代立，以鲁不顺，尽杀鲁母家室。鲁遂据汉中，以鬼道教民，自号“师君”。汉末，力不能征，遂就宠鲁为镇民中郎将，领汉宁太守，通贡献而已。民有地中得玉印者，群下欲尊鲁为汉宁王。功曹阎圃劝免。建安二十年，太祖乃自散关出武都征之，至阳平关。鲁欲举汉中降，其弟卫不肯，率众数万人拒关坚守。太祖攻破之，遂入蜀。鲁闻阳平已陷，将稽颡，圃劝奔南山入巴中。左右欲悉烧宝货仓库，鲁不从，遂封藏而去。太祖入南郑，甚嘉之。又以鲁本有善意，遣人慰喻。鲁尽将家出，太祖逆拜鲁镇南将军，待以客礼，封阆中侯，邑万户。封鲁五子及阎圃等皆为列侯。为子彭祖取鲁女。鲁薨，谥之曰原侯。子富嗣。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张鲁.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('zhang_lu', 'characters_of_the_three_kingdoms', '张鲁', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张鲁.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('ctk_5f20_2b5ae', '张𫖮', NULL, NULL, NULL, NULL, 'Male', 55, 54, 72, 47, 75, 'ctk_import,general,source_backed', 'High', '建安九年（204年），袁尚再攻平原，曹操趁机攻邺。袁尚领兵来救，马延、张𫖮临阵倒戈，令袁尚军溃不成军，期后下落不明。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 张𫖮.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('ctk_5f20_2b5ae', 'characters_of_the_three_kingdoms', '张𫖮', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/张𫖮.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES ('ctk_start_ctk_5f20_2b5ae', 'ctk_5f20_2b5ae', 190, 1, 'Appear', NULL, NULL, 76, 'Characters_of_the_Three_Kingdoms 导入人物初始登场事件');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('cao_cao', '曹操', '孟德', '沛国谯县', 155, 220, 'Male', 69, 66, 63, 52, 68, 'ctk_import,faction:魏国,source_backed', 'High', '政治家、军事家、诗人，统一了北方、挟天子以令诸侯，戎马一生。曹操父亲曹嵩为宦官曹腾养子，曹腾为汉相曹参之后。曹操谥武王，曹丕称帝后，追尊他为武皇帝，史称魏武帝。曹操在北方屯田，兴修水利，解决了军粮缺乏的问题，对农业生产的恢复有一定作用；用人唯才，罗致地主阶级中下层人物，抑制豪强，加强集权。颂令收田租亩四升，戶出绢二匹、锦二斤，为日后的租庸调之始。所统治的地区社会经济得到恢复和发展。草创九品官人法。精兵法，著《孙子略解》、《兵书接要》等书。善诗歌，《蒿里行》、《观沧海》等篇，抒发自己的政治抱负，并反映汉末人民的苦难生活，气魄雄伟，慷慨悲凉。散文亦清峻整洁。著作有《魏武帝集》。葬于高陵。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 曹操.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('cao_cao', 'characters_of_the_three_kingdoms', '曹操', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/曹操.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES ('zhuge_liang', '诸葛亮', '孔明', '徐州琅琊阳都', 181, 234, 'Male', 76, 65, 83, 90, 84, 'administrator,ctk_import,faction:蜀汉,source_backed', 'High', '政治家、军事家，被誉为“千古良相”的典范。父母早亡，由叔父玄抚养长大，后因徐州之乱，避乱荆州，潜心向学，淡泊明志。后受刘备三顾之礼，提出著名的《隆中对》，策动孙、刘联盟，于赤壁之战中大破曹操，奠定三国鼎立的基础。蜀汉建立，拜为丞相。刘备伐吴失败，受托孤于永安，辅佐幼主，外联东吴，内修政理，南征平叛，北抗强魏。为完成统一中原，兴复汉室的大业，先后五次进攻魏国，在治国、治军等方面发挥了非凡的才能，是以民用其力，百姓不忿；又推演兵法，作“八阵图”，造损益连弩、木牛流马，与名将司马懿、张郃等交锋，屡操胜算，最后一次北伐时采取分兵屯田之策，与司马懿大军相持百余日，但不幸因积劳成疾而逝世，享年五十四岁，谥曰忠武侯。其“鞠躬尽力，死而后已”的高尚品格，千百年来一直为人们所敬仰和怀念。', '来源: fthux/Characters_of_the_Three_Kingdoms MIT 语料；源文件 诸葛亮.json')
ON CONFLICT(id) DO UPDATE SET
    courtesy_name = COALESCE(excluded.courtesy_name, officers.courtesy_name),
    native_place = COALESCE(excluded.native_place, officers.native_place),
    birth_year = COALESCE(officers.birth_year, excluded.birth_year),
    death_year = COALESCE(officers.death_year, excluded.death_year),
    gender = excluded.gender,
    tags = CASE WHEN instr(officers.tags, 'ctk_import') = 0 THEN trim(officers.tags || ',ctk_import', ',') ELSE officers.tags END,
    biography = CASE WHEN excluded.biography <> '' THEN excluded.biography ELSE officers.biography END,
    notes = CASE WHEN instr(officers.notes, 'Characters_of_the_Three_Kingdoms') = 0 THEN trim(officers.notes || '；' || excluded.notes, '；') ELSE officers.notes END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES ('zhuge_liang', 'characters_of_the_three_kingdoms', '诸葛亮', 'https://github.com/fthux/Characters_of_the_Three_Kingdoms/blob/master/characters/诸葛亮.json', 'High', 'MIT licensed game-oriented character corpus');

INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('cao_cao', 'cao_pi', 'ParentChild', 'Medium', '亲子: 卞皇后长子，220年称帝为魏文帝。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5218_5b8f', 'han_xian_di', 'ParentChild', 'Medium', '亲子: 汉献帝刘协，王美人生', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5218_5ba0', 'ctk_5218_8206', 'Sibling', 'Medium', '兄弟: 刘舆兄长，齐孝王刘将闾之子牟平共侯刘渫的后代。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5218_5cb1', 'ctk_5218_8206', 'ParentChild', 'Medium', '父子/父女: 又作刘方，东汉山阳太守。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5218_5cb1', 'liu_yao', 'Sibling', 'Medium', '兄弟: 兖州刺史。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5218_5cb1', 'liu_yao', 'Sibling', 'Medium', '兄弟: 扬州牧，刘岱弟。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5218_7109', 'liu_zhang', 'ParentChild', 'Medium', '亲子: 幼子，字季玉，为人懦弱多疑，袭职益州牧。后为刘备所夺位，迁居公安。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5218_7426', 'liu_biao', 'ParentChild', 'Medium', '父子/父女: 镇南将军、荆州牧，与当地七贤人号称“江夏八俊”。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5218_8206', 'ctk_5218_5ba0', 'Sibling', 'Medium', '兄弟: 刘舆兄长，齐孝王刘将闾之子牟平共侯刘渫的后代。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5218_8206', 'ctk_5218_5cb1', 'ParentChild', 'Medium', '亲子: 兖州刺史。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5218_8206', 'liu_yao', 'ParentChild', 'Medium', '亲子: 扬州牧，刘岱弟。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_4f11', 'ctk_5f20_627f_32', 'Sibling', 'Medium', '兄弟: 张休兄长，字仲嗣。官至濡须都督、奋威将军，封都乡侯。死后谥定侯。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_4f11', 'zhang_zhao', 'ParentChild', 'Medium', '父子/父女: 孙吴重臣', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_536b', 'ctk_5f20_8861', 'ParentChild', 'Medium', '父子/父女: 字灵真', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_536b', 'zhang_lu', 'Sibling', 'Medium', '兄弟: 字公则，张鲁之弟、张愧之兄，随张鲁降曹后为昭义将军。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_536b', 'zhang_lu', 'Sibling', 'Medium', '兄弟: 张卫之兄', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_5b9d', 'ctk_5f20_6881', 'Sibling', 'Medium', '兄弟: 二哥，黄巾起义首领之一，号称“地公将军”。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_5b9d', 'ctk_5f20_6881', 'Sibling', 'Medium', '兄弟: 弟弟，黄巾起义首领之一，号称“人公将军”。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_5b9d', 'ctk_5f20_89d2', 'Sibling', 'Medium', '兄弟: 二弟，黄巾起义首领之一，号称“地公将军”。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_5b9d', 'ctk_5f20_89d2', 'Sibling', 'Medium', '兄弟: 哥哥，太平道首领，号称“天公将军”。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_627f_32', 'ctk_5f20_4f11', 'Sibling', 'Medium', '兄弟: 张休兄长，字仲嗣。官至濡须都督、奋威将军，封都乡侯。死后谥定侯。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_65e2', 'ctk_5f20_7f09', 'ParentChild', 'Medium', '亲子: 继承西乡侯爵位。曹魏官员，官至光禄大夫，位特进。被司马师赐死狱中。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_6881', 'ctk_5f20_5b9d', 'Sibling', 'Medium', '兄弟: 二哥，黄巾起义首领之一，号称“地公将军”。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_6881', 'ctk_5f20_5b9d', 'Sibling', 'Medium', '兄弟: 弟弟，黄巾起义首领之一，号称“人公将军”。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_6881', 'ctk_5f20_89d2', 'Sibling', 'Medium', '兄弟: 三弟，黄巾起义首领之一，号称“人公将军”。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_6881', 'ctk_5f20_89d2', 'Sibling', 'Medium', '兄弟: 大哥，太平道首领，号称“天公将军”。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_6d4e', 'ctk_5f20_7ee3', 'ParentChild', 'Medium', '亲子: 张济从子，张济死后领导余众', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_6e29_32', 'ctk_5f20_5141', 'ParentChild', 'Medium', '父子/父女: 东汉末期刘表的外甥和属下，曾与蔡瑁共同推刘琮继承刘表的地位。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_7ecd', 'ctk_5f20_82de', 'Sibling', 'Medium', '兄弟: 兄长，早卒', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_7ecd', 'ctk_5f20_82de', 'Sibling', 'Medium', '兄弟: 张苞之弟，官至侍中尚书仆射。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_7ecd', 'zhang_fei', 'ParentChild', 'Medium', '父子/父女: 父子/父女', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_7edf', 'ctk_5f20_864e', 'ParentChild', 'Medium', '父子/父女: 张辽之子', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_7f09', 'ctk_5f20_65e2', 'ParentChild', 'Medium', '父子/父女: 曹魏凉州刺史', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_7f09', 'ctk_5f20_9088', 'ParentChild', 'Medium', '亲子: 张缉之子，曾被张缉派遗与李丰通讯，事败时张邈被诛杀。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_8083', 'zhang_song', 'Sibling', 'Medium', '兄弟: 兄，字君矫，广汉太守。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_8083', 'zhang_song', 'Sibling', 'Medium', '兄弟: 刘璋属下大臣。为了祖止张鲁进攻蜀地，欲使刘备替代软弱的刘璋为蜀主。但是，计划败露，被视同叛臣处斩。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_82de', 'ctk_5f20_7ecd', 'Sibling', 'Medium', '兄弟: 兄长，早卒', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_82de', 'ctk_5f20_7ecd', 'Sibling', 'Medium', '兄弟: 张苞之弟，官至侍中尚书仆射。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_82de', 'ctk_5f20_9075', 'ParentChild', 'Medium', '亲子: 张苞之子，为尚书。魏灭蜀之战时，随诸葛瞻守于绵竹关，与邓艾交战，战死。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_82de', 'zhang_fei', 'ParentChild', 'Medium', '父子/父女: 三国时代蜀汉重要将领', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_864e', 'ctk_5f20_7edf', 'ParentChild', 'Medium', '亲子: 亲子', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_864e', 'zhang_liao', 'ParentChild', 'Medium', '父子/父女: 曹魏名将，五子良将之首', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_89d2', 'ctk_5f20_5b9d', 'Sibling', 'Medium', '兄弟: 二弟，黄巾起义首领之一，号称“地公将军”。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_89d2', 'ctk_5f20_5b9d', 'Sibling', 'Medium', '兄弟: 哥哥，太平道首领，号称“天公将军”。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_89d2', 'ctk_5f20_6881', 'Sibling', 'Medium', '兄弟: 三弟，黄巾起义首领之一，号称“人公将军”。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_89d2', 'ctk_5f20_6881', 'Sibling', 'Medium', '兄弟: 大哥，太平道首领，号称“天公将军”。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_8d85', 'ctk_5f20_9088', 'Sibling', 'Medium', '兄弟: 张超之兄', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_8d85', 'ctk_5f20_9088', 'Sibling', 'Medium', '兄弟: 张邈之弟', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_9053_9675', 'ctk_5f20_8861', 'ParentChild', 'Medium', '亲子: 张鲁之父', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_9075', 'ctk_5f20_82de', 'ParentChild', 'Medium', '父子/父女: 张飞之子', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_9088', 'ctk_5f20_8d85', 'Sibling', 'Medium', '兄弟: 张超之兄', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('ctk_5f20_9088', 'ctk_5f20_8d85', 'Sibling', 'Medium', '兄弟: 张邈之弟', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('liu_bei', 'liu_shan', 'ParentChild', 'Medium', '亲子: 字公嗣，刘备长子。后登上皇位。乳名阿斗。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('liu_biao', 'ctk_5218_7426', 'ParentChild', 'Medium', '亲子: 长子，官至荆州刺史。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('liu_yao', 'ctk_5218_5cb1', 'Sibling', 'Medium', '兄弟: 兖州刺史。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('liu_yao', 'ctk_5218_5cb1', 'Sibling', 'Medium', '兄弟: 扬州牧，刘岱弟。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('liu_yao', 'ctk_5218_8206', 'ParentChild', 'Medium', '父子/父女: 又作刘方，东汉山阳太守。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('sun_ce', 'sun_quan', 'Sibling', 'Medium', '兄弟: 长兄，东吴奠基者，孙权称帝后追谥为长沙桓王', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('sun_quan', 'sun_ce', 'Sibling', 'Medium', '兄弟: 长兄，东吴奠基者，孙权称帝后追谥为长沙桓王', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('sun_quan', 'sun_jian', 'ParentChild', 'Medium', '父子/父女: 孙权称帝后追谥为武烈皇帝', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('zhang_fei', 'ctk_5f20_7ecd', 'ParentChild', 'Medium', '亲子: 张飞次子，官至侍中、尚书仆射。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('zhang_fei', 'ctk_5f20_82de', 'ParentChild', 'Medium', '亲子: 张飞长子，早逝，但有一子张遵。《三国演义》中虚构其参与诸葛亮北伐，坠崖伤重而亡。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('zhang_liao', 'ctk_5f20_864e', 'ParentChild', 'Medium', '亲子: 在张辽死后嗣继其爵位，后来任至偏将军。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('zhang_lu', 'ctk_5f20_536b', 'Sibling', 'Medium', '兄弟: 字公则，张鲁之弟、张愧之兄，随张鲁降曹后为昭义将军。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('zhang_lu', 'ctk_5f20_536b', 'Sibling', 'Medium', '兄弟: 张卫之兄', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('zhang_lu', 'ctk_5f20_8861', 'ParentChild', 'Medium', '父子/父女: 字灵真，五斗米道嗣师。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('zhang_song', 'ctk_5f20_8083', 'Sibling', 'Medium', '兄弟: 兄，字君矫，广汉太守。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('zhang_song', 'ctk_5f20_8083', 'Sibling', 'Medium', '兄弟: 刘璋属下大臣。为了祖止张鲁进攻蜀地，欲使刘备替代软弱的刘璋为蜀主。但是，计划败露，被视同叛臣处斩。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('zhang_zhao', 'ctk_5f20_4f11', 'ParentChild', 'Medium', '亲子: 次子，字叔嗣。官至扬武将军，袭父张昭爵娄侯。', 'characters_of_the_three_kingdoms');
INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES ('zhang_zhao', 'ctk_5f20_627f_32', 'ParentChild', 'Medium', '亲子: 长子，字仲嗣。官至濡须都督、奋威将军，封都乡侯。死后谥定侯。', 'characters_of_the_three_kingdoms');

-- Curated relationship overlay for high-value gameplay and UI validation.
-- Import-generated relationships may add more rows; this seed keeps known
-- canonical ties stable across source updates.

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
VALUES
('lady_gan','甘夫人',NULL,'沛郡',NULL,NULL,'Female',18,12,58,62,72,'spouse,shu_han,source_backed','Medium','甘夫人，刘备夫人，刘禅生母。长坂之乱时与刘备家属失散，赵云护送其母子脱险；后被追谥为昭烈皇后。','关系校订补充人物，用于刘备家族关系'),
('lady_mi','糜夫人',NULL,'东海朐县',NULL,NULL,'Female',16,10,55,60,70,'spouse,shu_han,source_backed','Medium','糜夫人，糜竺之妹。刘备在徐州、海西一带困顿时，糜竺以妹嫁刘备并资助军资，是刘备早期集团的重要姻亲纽带。','关系校订补充人物，用于刘备家族关系'),
('lady_sun','孙夫人',NULL,'吴郡富春',NULL,NULL,'Female',42,48,64,58,82,'spouse,eastern_wu,source_backed','Medium','孙夫人，孙权之妹。赤壁后孙刘联盟期间与刘备成婚，后刘备入蜀，孙权遣人迎回，成为吴蜀政治婚姻的代表人物。','关系校订补充人物，用于刘备家族关系'),
('empress_mu','穆皇后','吴氏','陈留',NULL,245,'Female',25,18,66,78,76,'spouse,shu_han,source_backed','Medium','穆皇后吴氏，吴懿之妹，先嫁刘瑁，后为刘备夫人。刘备称汉中王后立为王后，刘禅即位后尊为皇太后。','关系校订补充人物，用于刘备家族关系'),
('lu_meng','吕蒙','子明','汝南富陂',178,220,'Male',88,82,84,72,76,'general,eastern_wu,source_backed','High','吕蒙，东吴名将。早年以勇武从军，后折节读书，参与经营江淮与荆州战线；建安二十四年袭取荆州，迫使关羽败亡，是孙权集团扩张荆州的关键将领。','关系校订补充人物，用于关羽仇敌关系')
ON CONFLICT(id) DO UPDATE SET
    gender = excluded.gender,
    biography = excluded.biography,
    tags = CASE WHEN instr(officers.tags, 'source_backed') = 0 THEN trim(officers.tags || ',source_backed', ',') ELSE officers.tags END;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES
('lady_gan','manual_curated','甘夫人','', 'Medium','刘备家族关系校订'),
('lady_mi','manual_curated','糜夫人','', 'Medium','刘备家族关系校订'),
('lady_sun','manual_curated','孙夫人','', 'Medium','刘备家族关系校订'),
('empress_mu','manual_curated','穆皇后','', 'Medium','刘备家族关系校订'),
('lu_meng','manual_curated','吕蒙','', 'High','关羽仇敌关系校订');

INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES
('liu_bei','guan_yu','SwornSibling','Medium','演义桃园结义；史书亦载二人恩若兄弟','manual_curated'),
('guan_yu','liu_bei','SwornSibling','Medium','演义桃园结义；史书亦载二人恩若兄弟','manual_curated'),
('liu_bei','zhang_fei','SwornSibling','Medium','演义桃园结义；史书亦载二人恩若兄弟','manual_curated'),
('zhang_fei','liu_bei','SwornSibling','Medium','演义桃园结义；史书亦载二人恩若兄弟','manual_curated'),
('guan_yu','zhang_fei','SwornSibling','Medium','演义桃园结义','manual_curated'),
('zhang_fei','guan_yu','SwornSibling','Medium','演义桃园结义','manual_curated'),
('sun_jian','sun_ce','ParentChild','High','孙坚为孙策之父','manual_curated'),
('sun_ce','sun_jian','ParentChild','High','孙策为孙坚之子','manual_curated'),
('sun_jian','sun_quan','ParentChild','High','孙坚为孙权之父','manual_curated'),
('sun_quan','sun_jian','ParentChild','High','孙权为孙坚之子','manual_curated'),
('sun_ce','sun_quan','Sibling','High','孙策、孙权为兄弟','manual_curated'),
('sun_quan','sun_ce','Sibling','High','孙策、孙权为兄弟','manual_curated'),
('cao_cao','cao_pi','ParentChild','High','曹操为曹丕之父','manual_curated'),
('cao_pi','cao_cao','ParentChild','High','曹丕为曹操之子','manual_curated'),
('liu_bei','liu_shan','ParentChild','High','刘备为刘禅之父','manual_curated'),
('liu_shan','liu_bei','ParentChild','High','刘禅为刘备之子','manual_curated'),
('liu_bei','lady_gan','Spouse','Medium','刘备夫人，刘禅生母','manual_curated'),
('lady_gan','liu_bei','Spouse','Medium','刘备夫人，刘禅生母','manual_curated'),
('liu_bei','lady_mi','Spouse','Medium','糜竺之妹，刘备早期姻亲','manual_curated'),
('lady_mi','liu_bei','Spouse','Medium','糜竺之妹，刘备早期姻亲','manual_curated'),
('liu_bei','lady_sun','Spouse','Medium','孙刘联盟政治婚姻','manual_curated'),
('lady_sun','liu_bei','Spouse','Medium','孙刘联盟政治婚姻','manual_curated'),
('liu_bei','empress_mu','Spouse','Medium','刘备汉中王后，后为蜀汉皇太后','manual_curated'),
('empress_mu','liu_bei','Spouse','Medium','刘备汉中王后，后为蜀汉皇太后','manual_curated'),
('liu_bei','zhuge_liang','RulerSubject','High','刘备三顾茅庐，诸葛亮后受托孤','manual_curated'),
('zhuge_liang','liu_bei','RulerSubject','High','诸葛亮辅佐刘备并受托孤','manual_curated'),
('cao_cao','xun_yu','RulerSubject','High','荀彧为曹操重要谋臣','manual_curated'),
('xun_yu','cao_cao','RulerSubject','High','荀彧辅佐曹操经营中原','manual_curated'),
('sun_quan','zhou_yu','RulerSubject','High','周瑜辅佐孙权并主持赤壁战事','manual_curated'),
('zhou_yu','sun_quan','RulerSubject','High','周瑜为孙权核心将领','manual_curated'),
('cao_cao','lu_bu','Enemy','High','曹操与吕布在兖徐战场长期敌对，最终擒杀吕布','manual_curated'),
('lu_bu','cao_cao','Enemy','High','吕布与曹操在兖徐战场长期敌对，最终为曹操所灭','manual_curated'),
('guan_yu','lu_meng','Enemy','High','吕蒙袭取荆州，导致关羽败亡','manual_curated'),
('lu_meng','guan_yu','Enemy','High','吕蒙袭取荆州，导致关羽败亡','manual_curated'),
('zhuge_liang','sima_yi','Enemy','Medium','诸葛亮北伐与司马懿在魏蜀前线长期对峙','manual_curated'),
('sima_yi','zhuge_liang','Enemy','Medium','司马懿与诸葛亮在魏蜀前线长期对峙','manual_curated');
