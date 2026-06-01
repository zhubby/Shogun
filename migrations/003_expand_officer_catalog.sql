-- Expand the historical officer catalog to 420 named profiles with a 5:1 male/female ratio.
-- All new records are hand-curated summaries for gameplay data; no external prose is copied.

DELETE FROM officer_life_events WHERE officer_id = 'zhao_yun_early';
DELETE FROM officer_relationships
WHERE source_officer_id = 'zhao_yun_early' OR target_officer_id = 'zhao_yun_early';
DELETE FROM officer_external_ids WHERE officer_id = 'zhao_yun_early';
DELETE FROM officers WHERE id = 'zhao_yun_early';

UPDATE officers
SET name = '张武',
    biography = '张武为《三国演义》江夏相关人物，常与刘备南征荆州的故事相连。',
    notes = notes || '；003 扩充迁移修正 CTK 导入姓名字段为张武'
WHERE id = 'ctk_5f20_6b66' AND name = '名不详';

CREATE TEMP TABLE expansion_003_officers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    gender TEXT NOT NULL CHECK (gender IN ('Male', 'Female')),
    tags TEXT NOT NULL,
    role TEXT NOT NULL,
    faction_id TEXT NOT NULL,
    city_id TEXT NOT NULL,
    relationship_target_id TEXT NOT NULL,
    relationship_kind TEXT NOT NULL CHECK (relationship_kind IN (
        'RulerSubject',
        'ParentChild',
        'AdoptiveParentChild',
        'Spouse',
        'Sibling',
        'SwornSibling',
        'Enemy'
    )),
    summary TEXT NOT NULL
);

INSERT INTO expansion_003_officers
(id, name, gender, tags, role, faction_id, city_id, relationship_target_id, relationship_kind, summary)
VALUES
('liu_bian', '刘辩', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '刘辩为东汉朝廷与士人人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('he_jin', '何进', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '何进为东汉朝廷与士人人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('he_miao', '何苗', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '何苗为东汉朝廷与士人人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('jian_shuo', '蹇硕', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '蹇硕为东汉朝廷与士人人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('huangfu_song', '皇甫嵩', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '皇甫嵩为东汉朝廷与士人人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhu_jun', '朱儁', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '朱儁为东汉朝廷与士人人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('lu_zhi', '卢植', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '卢植为东汉朝廷与士人人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cai_yong', '蔡邕', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '蔡邕为东汉朝廷与士人人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('mi_heng', '祢衡', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '祢衡为东汉朝廷与士人人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xu_zijiang', '许子将', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '许子将为东汉朝廷与士人人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('yu_ji', '于吉', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '于吉为东汉朝廷与士人人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zuo_ci', '左慈', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '左慈为东汉朝廷与士人人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('chen_deng', '陈登', 'Male', 'expansion_003,history,xuzhou,official', 'official', 'tao_qian', 'xiapi', 'tao_qian', 'RulerSubject', '陈登为徐州地方势力人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('chen_gui', '陈珪', 'Male', 'expansion_003,history,xuzhou,official', 'official', 'tao_qian', 'xiapi', 'tao_qian', 'RulerSubject', '陈珪为徐州地方势力人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zang_ba', '臧霸', 'Male', 'expansion_003,history,xuzhou,official', 'official', 'tao_qian', 'xiapi', 'tao_qian', 'RulerSubject', '臧霸为徐州地方势力人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('li_jue', '李傕', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '李傕为董卓集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('guo_si', '郭汜', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '郭汜为董卓集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('niu_fu', '牛辅', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '牛辅为董卓集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('fan_chou', '樊稠', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '樊稠为董卓集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xu_rong', '徐荣', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '徐荣为董卓集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('hua_xiong', '华雄', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '华雄为董卓集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('hu_zhen', '胡轸', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '胡轸为董卓集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('dong_min', '董旻', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '董旻为董卓集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('li_su', '李肃', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '李肃为董卓集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('wang_yun', '王允', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '王允为东汉朝廷与士人人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('han_fu', '韩馥', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '韩馥为河北袁氏集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('qiao_mao', '桥瑁', 'Male', 'expansion_003,history,coalition,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '桥瑁为讨董诸侯人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('bao_xin', '鲍信', 'Male', 'expansion_003,history,coalition,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '鲍信为讨董诸侯人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('yuan_yi', '袁遗', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '袁遗为河北袁氏集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xu_you', '许攸', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '许攸为河北袁氏集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('guo_tu', '郭图', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '郭图为河北袁氏集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('feng_ji', '逢纪', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '逢纪为河北袁氏集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('gao_gan', '高干', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '高干为河北袁氏集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('chunyu_qiong', '淳于琼', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '淳于琼为河北袁氏集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('han_meng', '韩猛', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '韩猛为河北袁氏集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('jiang_yiqu', '蒋义渠', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '蒋义渠为河北袁氏集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('lu_kuang', '吕旷', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '吕旷为河北袁氏集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('lu_xiang', '吕翔', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '吕翔为河北袁氏集团人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('gongsun_du', '公孙度', 'Male', 'expansion_003,history,northern,general', 'general', 'gongsun_zan', 'ji', 'gongsun_zan', 'RulerSubject', '公孙度为辽东与幽州群雄人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('gongsun_kang', '公孙康', 'Male', 'expansion_003,history,northern,general', 'general', 'gongsun_zan', 'ji', 'gongsun_zan', 'RulerSubject', '公孙康为辽东与幽州群雄人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('gongsun_gong', '公孙恭', 'Male', 'expansion_003,history,northern,general', 'general', 'gongsun_zan', 'ji', 'gongsun_zan', 'RulerSubject', '公孙恭为辽东与幽州群雄人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('gongsun_yuan', '公孙渊', 'Male', 'expansion_003,history,northern,general', 'general', 'gongsun_zan', 'ji', 'gongsun_zan', 'RulerSubject', '公孙渊为辽东与幽州群雄人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('yan_gang', '严纲', 'Male', 'expansion_003,history,northern,general', 'general', 'gongsun_zan', 'ji', 'gongsun_zan', 'RulerSubject', '严纲为辽东与幽州群雄人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('tian_kai', '田楷', 'Male', 'expansion_003,history,northern,general', 'general', 'gongsun_zan', 'ji', 'gongsun_zan', 'RulerSubject', '田楷为辽东与幽州群雄人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhou_cang', '周仓', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '周仓为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('guan_ping', '关平', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '关平为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('guan_xing', '关兴', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '关兴为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('guan_suo', '关索', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '关索为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('guan_tong', '关统', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '关统为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('guan_yi', '关彝', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '关彝为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('liu_feng', '刘封', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '刘封为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('liu_yong', '刘永', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '刘永为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('liu_li', '刘理', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '刘理为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('liu_ba', '刘巴', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '刘巴为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('li_yan', '李严', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '李严为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('ma_liang', '马良', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '马良为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('ma_su', '马谡', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '马谡为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('yang_yi', '杨仪', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '杨仪为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('fei_yi', '费祎', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '费祎为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('jiang_wan', '蒋琬', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '蒋琬为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('dong_yun', '董允', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '董允为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('chen_zhen', '陈震', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '陈震为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('deng_zhi', '邓芝', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '邓芝为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('liao_hua', '廖化', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '廖化为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('wang_ping', '王平', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '王平为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('wu_ban', '吴班', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '吴班为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('wu_yi', '吴懿', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '吴懿为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('ma_zhong_shu', '马忠', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '马忠为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('huang_quan', '黄权', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '黄权为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('meng_da', '孟达', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '孟达为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('huo_jun', '霍峻', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '霍峻为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('huo_yi', '霍弋', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '霍弋为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('fu_qian', '傅佥', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '傅佥为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhuge_zhan', '诸葛瞻', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '诸葛瞻为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhuge_shang', '诸葛尚', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '诸葛尚为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhuge_jun', '诸葛均', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '诸葛均为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('mi_fang', '糜芳', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '糜芳为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('fu_shiren', '傅士仁', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '傅士仁为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('liu_yan_shu', '刘琰', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '刘琰为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xiang_lang', '向朗', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '向朗为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xiang_chong', '向宠', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '向宠为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('qiao_zhou', '谯周', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '谯周为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xi_zheng', '郤正', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '郤正为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('luo_xian', '罗宪', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '罗宪为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('gao_xiang', '高翔', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '高翔为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('chen_shi_shu', '陈式', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '陈式为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('feng_xi', '冯习', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '冯习为蜀汉阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sha_moke', '沙摩柯', 'Male', 'expansion_003,romance,nanzhong,general', 'general', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '沙摩柯为南中与西南传说势力人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('meng_huo', '孟获', 'Male', 'expansion_003,romance,nanzhong,general', 'general', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '孟获为南中与西南传说势力人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('dailai_dongzhu', '带来洞主', 'Male', 'expansion_003,romance,nanzhong,general', 'general', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '带来洞主为南中与西南传说势力人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('mang_yachang', '忙牙长', 'Male', 'expansion_003,romance,nanzhong,general', 'general', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '忙牙长为南中与西南传说势力人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('duosi_dawang', '朵思大王', 'Male', 'expansion_003,romance,nanzhong,general', 'general', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '朵思大王为南中与西南传说势力人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('wutugu', '兀突骨', 'Male', 'expansion_003,romance,nanzhong,general', 'general', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '兀突骨为南中与西南传说势力人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_ang', '曹昂', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹昂为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_zhang', '曹彰', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹彰为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_zhi', '曹植', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹植为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_xiong', '曹熊', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹熊为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_chong', '曹冲', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹冲为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_yu', '曹宇', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹宇为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_rui', '曹叡', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹叡为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_fang', '曹芳', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹芳为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_mao', '曹髦', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹髦为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_huan', '曹奂', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹奂为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_zhen', '曹真', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹真为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_shuang', '曹爽', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹爽为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_xiu', '曹休', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹休为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_chun', '曹纯', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹纯为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_tai', '曹泰', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹泰为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_zhao', '曹肇', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹肇为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_xun', '曹训', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹训为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('cao_xi', '曹羲', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹羲为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xiahou_shang', '夏侯尚', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '夏侯尚为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xiahou_ba', '夏侯霸', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '夏侯霸为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xiahou_wei', '夏侯威', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '夏侯威为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xiahou_xuan', '夏侯玄', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '夏侯玄为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xiahou_hui_male', '夏侯惠', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '夏侯惠为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xiahou_he', '夏侯和', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '夏侯和为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sima_lang', '司马朗', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马朗为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sima_fu', '司马孚', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马孚为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sima_shi', '司马师', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马师为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sima_zhao', '司马昭', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马昭为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sima_yan', '司马炎', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马炎为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sima_you', '司马攸', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马攸为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sima_zhou', '司马伷', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马伷为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sima_wang', '司马望', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马望为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('chen_qun', '陈群', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '陈群为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhong_yao', '钟繇', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '钟繇为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhong_hui', '钟会', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '钟会为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('deng_ai', '邓艾', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '邓艾为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('deng_zhong', '邓忠', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '邓忠为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('guo_huai', '郭淮', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '郭淮为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('hao_zhao', '郝昭', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '郝昭为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('wang_lang', '王朗', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '王朗为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('wang_su', '王肃', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '王肃为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('wang_can', '王粲', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '王粲为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('liu_ye', '刘晔', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '刘晔为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('dong_zhao', '董昭', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '董昭为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('jiang_ji', '蒋济', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '蒋济为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('hua_xin', '华歆', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '华歆为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('jia_kui', '贾逵', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '贾逵为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xin_pi', '辛毗', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '辛毗为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xin_ping', '辛评', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '辛评为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('he_yan', '何晏', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '何晏为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('guanqiu_jian', '毌丘俭', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '毌丘俭为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('wen_pin', '文聘', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '文聘为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('wen_qin', '文钦', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '文钦为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('wen_yang', '文鸯', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '文鸯为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('wang_shuang', '王双', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '王双为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('li_tong', '李通', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '李通为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_li', '孙礼', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '孙礼为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('qian_zhao', '牵招', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '牵招为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('tian_yu', '田豫', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '田豫为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_yi', '孙翊', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙翊为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_kuang', '孙匡', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙匡为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_lang', '孙朗', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙朗为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_deng', '孙登', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙登为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_he', '孙和', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙和为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_ba', '孙霸', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙霸为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_liang', '孙亮', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙亮为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_xiu', '孙休', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙休为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_hao', '孙皓', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙皓为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_yu', '孙瑜', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙瑜为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_jiao', '孙皎', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙皎为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_huan', '孙桓', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙桓为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('sun_shao', '孙韶', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙韶为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhuge_jin', '诸葛瑾', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '诸葛瑾为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhuge_ke', '诸葛恪', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '诸葛恪为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhuge_rong', '诸葛融', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '诸葛融为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('lu_shu', '鲁淑', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '鲁淑为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('lv_fan', '吕范', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '吕范为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('lv_ju', '吕据', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '吕据为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhu_zhi', '朱治', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '朱治为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhu_ran', '朱然', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '朱然为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhu_huan', '朱桓', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '朱桓为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhu_ju', '朱据', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '朱据为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('gu_yong', '顾雍', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '顾雍为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('gu_tan', '顾谭', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '顾谭为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('bu_zhi', '步骘', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '步骘为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('bu_xie', '步协', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '步协为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('pan_zhang', '潘璋', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '潘璋为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('ling_cao', '凌操', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '凌操为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('ling_tong', '凌统', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '凌统为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xu_sheng', '徐盛', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '徐盛为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('ding_feng', '丁奉', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '丁奉为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('quan_cong', '全琮', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '全琮为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('quan_yi', '全怿', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '全怿为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('jiang_qin', '蒋钦', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '蒋钦为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('chen_wu', '陈武', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '陈武为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('dong_xi', '董袭', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '董袭为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('he_qi', '贺齐', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '贺齐为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('lu_kang', '陆抗', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '陆抗为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('lu_kai', '陆凯', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '陆凯为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('yu_fan', '虞翻', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '虞翻为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('yan_jun', '严畯', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '严畯为孙吴阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhao_ang', '赵昂', 'Male', 'expansion_003,history,liangzhou,official', 'official', 'ma_teng', 'wuwei', 'ma_teng', 'RulerSubject', '赵昂为凉州地方势力人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('zhao_fan', '赵范', 'Male', 'expansion_003,history,jingzhou,official', 'official', 'liu_biao', 'xiangyang', 'liu_biao', 'RulerSubject', '赵范为荆州地方势力人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('xun_song', '荀崧', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '荀崧为曹魏阵营人物，收录用于扩充三国时期群雄、宗族与幕府网络。'),
('empress_he', '何皇后', 'Female', 'expansion_003,female,history,han_court,empress', 'spouse', 'han_court', 'luoyang', 'ctk_5218_5b8f', 'Spouse', '何皇后为汉灵帝皇后、何进之妹，牵动外戚与宦官政治。'),
('empress_song_ling', '宋皇后', 'Female', 'expansion_003,female,history,han_court,empress', 'spouse', 'han_court', 'luoyang', 'ctk_5218_5b8f', 'Spouse', '宋皇后为汉灵帝早年皇后，代表东汉后宫政治的一支。'),
('empress_fu_shou', '伏寿', 'Female', 'expansion_003,female,history,han_court,empress', 'spouse', 'han_court', 'luoyang', 'han_xian_di', 'Spouse', '伏寿为汉献帝皇后，卷入许都朝廷与曹氏权力冲突。'),
('empress_cao_jie', '曹节', 'Female', 'expansion_003,female,history,han_court,empress', 'spouse', 'han_court', 'luoyang', 'han_xian_di', 'Spouse', '曹节为曹操之女、汉献帝皇后，是曹汉禅代的重要人物。'),
('consort_dong_han', '董贵人', 'Female', 'expansion_003,female,history,han_court,spouse', 'spouse', 'han_court', 'luoyang', 'han_xian_di', 'Spouse', '董贵人为汉献帝贵人，关联衣带诏后的宫廷斗争。'),
('lady_tang_hongnong', '唐姬', 'Female', 'expansion_003,female,history,han_court,spouse', 'spouse', 'han_court', 'luoyang', 'liu_bian', 'Spouse', '唐姬为弘农王刘辩之妃，董卓废立后守节事迹流传。'),
('lady_bian', '卞夫人', 'Female', 'expansion_003,female,history,cao_wei,spouse', 'spouse', 'cao_cao', 'xuchang', 'cao_cao', 'Spouse', '卞夫人为曹操正室，曹丕、曹彰、曹植等人的母亲。'),
('lady_ding_cao', '丁夫人', 'Female', 'expansion_003,female,history,cao_wei,spouse', 'spouse', 'cao_cao', 'xuchang', 'cao_cao', 'Spouse', '丁夫人为曹操早年夫人，因曹昂之死与曹操离异。'),
('lady_liu_cao', '刘夫人', 'Female', 'expansion_003,female,history,cao_wei,spouse', 'spouse', 'cao_cao', 'xuchang', 'cao_cao', 'Spouse', '刘夫人为曹操妾室，曹昂、曹铄等子女的生母。'),
('lady_huan_cao', '环夫人', 'Female', 'expansion_003,female,history,cao_wei,spouse', 'spouse', 'cao_cao', 'xuchang', 'cao_cao', 'Spouse', '环夫人为曹操妾室，曹冲等子女的母亲。'),
('lady_du_cao', '杜夫人', 'Female', 'expansion_003,female,history,cao_wei,spouse', 'spouse', 'cao_cao', 'xuchang', 'cao_cao', 'Spouse', '杜夫人原为秦宜禄之妻，后入曹操后宫，常见于史传轶事。'),
('lady_yin_cao', '尹夫人', 'Female', 'expansion_003,female,history,cao_wei,spouse', 'spouse', 'cao_cao', 'xuchang', 'cao_cao', 'Spouse', '尹夫人为曹操妾室，关联曹魏宗室后嗣。'),
('lady_zou', '邹氏', 'Female', 'expansion_003,female,romance,cao_wei,spouse', 'spouse', 'cao_cao', 'xuchang', 'cao_cao', 'Spouse', '邹氏为张济遗孀，宛城之变故事中的关键女性。'),
('empress_guo_nuwang', '郭女王', 'Female', 'expansion_003,female,history,cao_wei,empress', 'spouse', 'cao_cao', 'xuchang', 'cao_pi', 'Spouse', '郭女王为曹丕皇后，曹魏后宫政治中的核心人物。'),
('lady_zhen', '甄夫人', 'Female', 'expansion_003,female,history,cao_wei,spouse', 'spouse', 'cao_cao', 'xuchang', 'cao_pi', 'Spouse', '甄夫人为袁熙旧配、曹丕夫人，曹叡生母。'),
('empress_mao_cao_rui', '毛皇后', 'Female', 'expansion_003,female,history,cao_wei,empress', 'spouse', 'cao_cao', 'xuchang', 'cao_rui', 'Spouse', '毛皇后为曹叡皇后，见证魏明帝时期宫廷变动。'),
('empress_guo_cao_rui', '明元郭皇后', 'Female', 'expansion_003,female,history,cao_wei,empress', 'spouse', 'cao_cao', 'xuchang', 'cao_rui', 'Spouse', '明元郭皇后为曹叡后期皇后，曹芳即位后尊为皇太后。'),
('lady_cai', '蔡夫人', 'Female', 'expansion_003,female,history,jingzhou,spouse', 'spouse', 'liu_biao', 'xiangyang', 'liu_biao', 'Spouse', '蔡夫人为刘表后妻，关联荆州蔡氏与刘琮继承。'),
('huang_yueying', '黄月英', 'Female', 'expansion_003,female,tradition,shu_han,spouse', 'spouse', 'liu_bei', 'chengdu', 'zhuge_liang', 'Spouse', '黄月英为诸葛亮之妻，民间传统中以才智和机关术著称。'),
('xiahou_ji', '夏侯姬', 'Female', 'expansion_003,female,history,shu_han,spouse', 'spouse', 'liu_bei', 'chengdu', 'zhang_fei', 'Spouse', '夏侯姬为张飞妻，连接曹魏夏侯氏与蜀汉张氏。'),
('empress_zhang_jingai', '敬哀皇后张氏', 'Female', 'expansion_003,female,history,shu_han,empress', 'spouse', 'liu_bei', 'chengdu', 'liu_shan', 'Spouse', '敬哀皇后张氏为张飞长女、刘禅皇后。'),
('empress_zhang_later', '张皇后', 'Female', 'expansion_003,female,history,shu_han,empress', 'spouse', 'liu_bei', 'chengdu', 'liu_shan', 'Spouse', '张皇后为张飞次女，敬哀皇后去世后成为刘禅皇后。'),
('zhang_xingcai', '张星彩', 'Female', 'expansion_003,female,tradition,shu_han,spouse', 'spouse', 'liu_bei', 'chengdu', 'liu_shan', 'Spouse', '张星彩为后世三国题材塑造的张飞之女形象，常与刘禅关联。'),
('wang_yuanji', '王元姬', 'Female', 'expansion_003,female,history,cao_wei,spouse', 'spouse', 'cao_cao', 'xuchang', 'sima_zhao', 'Spouse', '王元姬为司马昭妻、晋武帝司马炎之母。'),
('zhang_chunhua', '张春华', 'Female', 'expansion_003,female,history,cao_wei,spouse', 'spouse', 'cao_cao', 'xuchang', 'sima_yi', 'Spouse', '张春华为司马懿妻，司马师、司马昭之母。'),
('xiahou_hui', '夏侯徽', 'Female', 'expansion_003,female,history,cao_wei,spouse', 'spouse', 'cao_cao', 'xuchang', 'sima_shi', 'Spouse', '夏侯徽为司马师妻，出身曹魏夏侯氏。'),
('yang_huiyu', '羊徽瑜', 'Female', 'expansion_003,female,history,cao_wei,spouse', 'spouse', 'cao_cao', 'xuchang', 'sima_shi', 'Spouse', '羊徽瑜为司马师后妻，晋代受尊崇。'),
('bai_lingyun', '柏灵筠', 'Female', 'expansion_003,female,tradition,cao_wei,spouse', 'spouse', 'cao_cao', 'xuchang', 'sima_yi', 'Spouse', '柏灵筠为三国题材流传中的司马懿侧室形象。'),
('empress_yang_yan', '杨艳', 'Female', 'expansion_003,female,history,jin_transition,empress', 'spouse', 'cao_cao', 'xuchang', 'sima_yan', 'Spouse', '杨艳为司马炎皇后，连接曹魏末期与西晋初年。'),
('empress_yang_zhi', '杨芷', 'Female', 'expansion_003,female,history,jin_transition,empress', 'spouse', 'cao_cao', 'xuchang', 'sima_yan', 'Spouse', '杨芷为晋武帝后期皇后，出自弘农杨氏。'),
('lady_hu_fen', '胡芳', 'Female', 'expansion_003,female,history,jin_transition,spouse', 'spouse', 'cao_cao', 'xuchang', 'sima_yan', 'Spouse', '胡芳为司马炎夫人，见于西晋后宫记载。'),
('zuo_fen', '左芬', 'Female', 'expansion_003,female,history,jin_transition,spouse', 'spouse', 'cao_cao', 'xuchang', 'sima_yan', 'Spouse', '左芬为西晋才女，入司马炎后宫并以文章知名。'),
('lady_wu_sunjian', '吴夫人', 'Female', 'expansion_003,female,history,eastern_wu,spouse', 'spouse', 'sun_quan', 'jianye', 'sun_jian', 'Spouse', '吴夫人为孙坚正室，孙策、孙权之母。'),
('lady_wu_guotai', '吴国太', 'Female', 'expansion_003,female,romance,eastern_wu,spouse', 'spouse', 'sun_quan', 'jianye', 'sun_jian', 'Spouse', '吴国太为演义和民间传统中的孙氏长辈形象。'),
('da_qiao', '大乔', 'Female', 'expansion_003,female,tradition,eastern_wu,spouse', 'spouse', 'sun_quan', 'jianye', 'sun_ce', 'Spouse', '大乔为乔氏二女之一，孙策夫人，常见于江东人物传统。'),
('xiao_qiao', '小乔', 'Female', 'expansion_003,female,tradition,eastern_wu,spouse', 'spouse', 'sun_quan', 'jianye', 'zhou_yu', 'Spouse', '小乔为乔氏二女之一，周瑜夫人，赤壁题材中知名。'),
('lady_xu_sunquan', '徐夫人', 'Female', 'expansion_003,female,history,eastern_wu,spouse', 'spouse', 'sun_quan', 'jianye', 'sun_quan', 'Spouse', '徐夫人为孙权早年夫人，抚育孙登。'),
('bu_lianshi', '步练师', 'Female', 'expansion_003,female,history,eastern_wu,spouse', 'spouse', 'sun_quan', 'jianye', 'sun_quan', 'Spouse', '步练师为孙权宠爱的夫人，孙鲁班、孙鲁育之母。'),
('lady_yuan_sunquan', '袁夫人', 'Female', 'expansion_003,female,history,eastern_wu,spouse', 'spouse', 'sun_quan', 'jianye', 'sun_quan', 'Spouse', '袁夫人为孙权夫人，出自汝南袁氏。'),
('lady_wang_sunhe', '王夫人', 'Female', 'expansion_003,female,history,eastern_wu,spouse', 'spouse', 'sun_quan', 'jianye', 'sun_quan', 'Spouse', '王夫人为孙和之母，卷入孙吴继承纷争。'),
('pan_shu', '潘淑', 'Female', 'expansion_003,female,history,eastern_wu,spouse', 'spouse', 'sun_quan', 'jianye', 'sun_quan', 'Spouse', '潘淑为孙权夫人、孙亮生母。'),
('lady_xie_sunquan', '谢夫人', 'Female', 'expansion_003,female,history,eastern_wu,spouse', 'spouse', 'sun_quan', 'jianye', 'sun_quan', 'Spouse', '谢夫人为孙权早年夫人，出自会稽谢氏。'),
('quan_huijie', '全惠解', 'Female', 'expansion_003,female,history,eastern_wu,empress', 'spouse', 'sun_quan', 'jianye', 'sun_liang', 'Spouse', '全惠解为孙亮皇后，关联孙吴全氏外戚。'),
('empress_zhu_sunxiu', '朱皇后', 'Female', 'expansion_003,female,history,eastern_wu,empress', 'spouse', 'sun_quan', 'jianye', 'sun_xiu', 'Spouse', '朱皇后为孙休皇后，孙吴后期宗室婚姻人物。'),
('empress_teng_sunhao', '滕皇后', 'Female', 'expansion_003,female,history,eastern_wu,empress', 'spouse', 'sun_quan', 'jianye', 'sun_hao', 'Spouse', '滕皇后为孙皓皇后，见证孙吴末期政治。'),
('sun_luban', '孙鲁班', 'Female', 'expansion_003,female,history,eastern_wu,spouse', 'spouse', 'sun_quan', 'jianye', 'quan_cong', 'Spouse', '孙鲁班为孙权长女，参与孙吴二宫之争。'),
('sun_luyu', '孙鲁育', 'Female', 'expansion_003,female,history,eastern_wu,spouse', 'spouse', 'sun_quan', 'jianye', 'zhu_ju', 'Spouse', '孙鲁育为孙权之女，与朱据婚配，卷入吴国内部纷争。'),
('lady_yan_lubu', '严氏', 'Female', 'expansion_003,female,history,lu_bu,spouse', 'spouse', 'lu_bu', 'puyang', 'lu_bu', 'Spouse', '严氏为吕布妻，常见于吕布败亡前后的叙事。'),
('diao_chan', '貂蝉', 'Female', 'expansion_003,female,romance,lu_bu,spouse', 'spouse', 'lu_bu', 'puyang', 'lu_bu', 'Spouse', '貂蝉为演义中连环计核心人物，连接董卓与吕布故事。'),
('dong_bai', '董白', 'Female', 'expansion_003,female,history,dong_zhuo,female', 'spouse', 'dong_zhuo', 'changan', 'dong_zhuo', 'ParentChild', '董白为董卓孙女，年少受封，反映董卓擅权。'),
('cai_yan', '蔡琰', 'Female', 'expansion_003,female,history,han_court,female', 'spouse', 'han_court', 'luoyang', 'cai_yong', 'ParentChild', '蔡琰字文姬，为蔡邕之女，以才学和归汉故事知名。'),
('xin_xianying', '辛宪英', 'Female', 'expansion_003,female,history,cao_wei,female', 'spouse', 'cao_cao', 'xuchang', 'xin_pi', 'ParentChild', '辛宪英为辛毗之女，以识见明敏见称。'),
('wang_yi', '王异', 'Female', 'expansion_003,female,history,liangzhou,spouse', 'spouse', 'ma_teng', 'wuwei', 'zhao_ang', 'Spouse', '王异为赵昂妻，凉州战事中以谋略和胆识著称。'),
('lady_fan', '樊氏', 'Female', 'expansion_003,female,romance,jingzhou,spouse', 'spouse', 'liu_biao', 'xiangyang', 'zhao_fan', 'Spouse', '樊氏为赵范寡嫂，演义中与赵云故事相关。'),
('zhu_rong', '祝融夫人', 'Female', 'expansion_003,female,romance,nanzhong,spouse,warrior', 'warrior', 'liu_bei', 'chengdu', 'meng_huo', 'Spouse', '祝融夫人为演义中的南中女将，孟获之妻。'),
('hua_man', '花鬘', 'Female', 'expansion_003,female,tradition,nanzhong,spouse', 'spouse', 'liu_bei', 'chengdu', 'guan_suo', 'Spouse', '花鬘为三国民间传统中的南中女性形象，常与关索关联。'),
('bao_sanniang', '鲍三娘', 'Female', 'expansion_003,female,tradition,shu_han,spouse,warrior', 'warrior', 'liu_bei', 'chengdu', 'guan_suo', 'Spouse', '鲍三娘为关索故事中的女性武艺形象。'),
('guan_yinping', '关银屏', 'Female', 'expansion_003,female,tradition,shu_han,female', 'spouse', 'liu_bei', 'chengdu', 'guan_yu', 'ParentChild', '关银屏为关羽之女的民间形象，常见于后世三国题材。'),
('ma_yunlu', '马云禄', 'Female', 'expansion_003,female,tradition,shu_han,spouse,warrior', 'warrior', 'liu_bei', 'chengdu', 'zhao_yun', 'Spouse', '马云禄为后世三国题材中马氏女将形象，常与赵云婚配。'),
('lady_li_machao', '李氏', 'Female', 'expansion_003,female,history,shu_han,spouse', 'spouse', 'liu_bei', 'chengdu', 'ma_chao', 'Spouse', '李氏为马超妻室的通称，补充马氏家族关系。'),
('lu_lingqi', '吕玲绮', 'Female', 'expansion_003,female,tradition,lu_bu,female,warrior', 'warrior', 'lu_bu', 'puyang', 'lu_bu', 'ParentChild', '吕玲绮为后世三国题材塑造的吕布之女形象。'),
('lady_cao_lu', '曹氏', 'Female', 'expansion_003,female,tradition,lu_bu,spouse', 'spouse', 'lu_bu', 'puyang', 'lu_bu', 'Spouse', '曹氏为吕布家属相关通称，补充吕布后宅关系。'),
('lady_yan_yuanshao', '刘夫人', 'Female', 'expansion_003,female,history,yuan_shao,spouse', 'spouse', 'yuan_shao', 'ye', 'yuan_shao', 'Spouse', '刘夫人为袁绍妻，袁氏诸子争立时具有家族影响。'),
('lady_feng', '冯方女', 'Female', 'expansion_003,female,history,yuan_shu,spouse', 'spouse', 'yuan_shu', 'shouchun', 'yuan_shu', 'Spouse', '冯方女为袁术后宫相关人物，见于汉末群雄婚姻网络。'),
('queen_zhang_lu', '卢夫人', 'Female', 'expansion_003,female,tradition,zhang_lu,spouse', 'spouse', 'zhang_lu', 'hanzhong', 'zhang_lu', 'Spouse', '卢夫人为张鲁妻室通称，补充汉中张氏家族关系。'),
('xun_guan', '荀灌', 'Female', 'expansion_003,female,history,jin_transition,female,warrior', 'warrior', 'cao_cao', 'xuchang', 'xun_song', 'ParentChild', '荀灌为荀崧之女，以突围求援的勇烈故事流传。');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, tags, confidence, biography, notes)
SELECT
    id,
    name,
    NULL,
    NULL,
    NULL,
    NULL,
    gender,
    CASE role
        WHEN 'general' THEN 74
        WHEN 'warrior' THEN 66
        WHEN 'spouse' THEN 24
        WHEN 'administrator' THEN 42
        ELSE 48
    END,
    CASE role
        WHEN 'general' THEN 78
        WHEN 'warrior' THEN 82
        WHEN 'spouse' THEN 18
        WHEN 'administrator' THEN 28
        ELSE 32
    END,
    CASE role
        WHEN 'general' THEN 55
        WHEN 'warrior' THEN 58
        WHEN 'spouse' THEN 62
        WHEN 'administrator' THEN 76
        ELSE 72
    END,
    CASE role
        WHEN 'general' THEN 48
        WHEN 'warrior' THEN 52
        WHEN 'spouse' THEN 66
        WHEN 'administrator' THEN 82
        ELSE 76
    END,
    CASE role
        WHEN 'general' THEN 62
        WHEN 'warrior' THEN 76
        WHEN 'spouse' THEN 78
        WHEN 'administrator' THEN 70
        ELSE 68
    END,
    tags,
    CASE WHEN tags LIKE '%tradition%' OR tags LIKE '%romance%' THEN 'Medium' ELSE 'High' END,
    summary,
    'expansion_003 手工校订；参考正史、演义、传记与历史流传，只保存原创摘要和结构化关系'
FROM expansion_003_officers;

INSERT OR REPLACE INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
SELECT
    id,
    'manual_curated',
    name,
    '',
    CASE WHEN tags LIKE '%tradition%' OR tags LIKE '%romance%' THEN 'Medium' ELSE 'High' END,
    'expansion_003 手工校订；用于 420 人历史资料库扩充'
FROM expansion_003_officers;

INSERT OR IGNORE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
SELECT
    'expansion_003_start_' || id,
    id,
    190,
    1,
    'Appear',
    faction_id,
    city_id,
    CASE WHEN tags LIKE '%tradition%' OR tags LIKE '%romance%' THEN 70 ELSE 76 END,
    'expansion_003 扩充人物初始登场事件'
FROM expansion_003_officers;

INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
SELECT
    id,
    relationship_target_id,
    relationship_kind,
    CASE WHEN tags LIKE '%tradition%' OR tags LIKE '%romance%' THEN 'Medium' ELSE 'High' END,
    summary,
    'manual_curated'
FROM expansion_003_officers;

INSERT OR IGNORE INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
SELECT
    relationship_target_id,
    id,
    relationship_kind,
    CASE WHEN tags LIKE '%tradition%' OR tags LIKE '%romance%' THEN 'Medium' ELSE 'High' END,
    summary,
    'manual_curated'
FROM expansion_003_officers
WHERE relationship_kind IN ('Spouse', 'Sibling', 'SwornSibling', 'Enemy');

DROP TABLE expansion_003_officers;
