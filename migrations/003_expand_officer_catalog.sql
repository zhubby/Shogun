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
('liu_bian', '刘辩', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '刘辩即汉少帝，董卓入洛阳后被废为弘农王，随后被李儒奉命毒杀。'),
('he_jin', '何进', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '何进为灵思皇后之兄，官至大将军，谋诛宦官失败引发洛阳政局崩坏。'),
('he_miao', '何苗', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '何苗为何进之弟，官至车骑将军，在宦官之乱后被袁绍等人杀死。'),
('jian_shuo', '蹇硕', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '蹇硕为汉灵帝亲信宦官，统领西园军，在灵帝死后的继承斗争中被何进诛杀。'),
('huangfu_song', '皇甫嵩', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '皇甫嵩为东汉名将，平定黄巾主力，汉末仍以宿将声望维系朝廷军威。'),
('zhu_jun', '朱儁', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '朱儁为东汉讨黄巾名将，后参与关东局势和长安政争，是汉末重臣之一。'),
('lu_zhi', '卢植', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '卢植为经学名臣和讨黄巾将领，刘备曾受业于其门下。'),
('cai_yong', '蔡邕', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '蔡邕为东汉文学家和书法家，董卓征辟入朝，亦为蔡琰之父。'),
('mi_heng', '祢衡', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '祢衡为汉末名士，以才气和狂狷著称，因触怒权贵辗转至荆州，终被黄祖杀害。'),
('xu_zijiang', '许子将', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '许劭字子将，以月旦评品评人物闻名，曾评价曹操为治世能臣、乱世奸雄。'),
('yu_ji', '于吉', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '于吉为江东方士，传说因民间声望过高被孙策处死，后来成为孙策故事中的异术人物。'),
('zuo_ci', '左慈', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '左慈为方士传说人物，常见于曹操相关轶事，以幻术和隐逸形象流传。'),
('chen_deng', '陈登', 'Male', 'expansion_003,history,xuzhou,official', 'official', 'tao_qian', 'xiapi', 'tao_qian', 'RulerSubject', '陈登字元龙，为徐州名士和广陵太守，辅佐刘备、曹操对抗吕布。'),
('chen_gui', '陈珪', 'Male', 'expansion_003,history,xuzhou,official', 'official', 'tao_qian', 'xiapi', 'tao_qian', 'RulerSubject', '陈珪为徐州士族长者，曾与陈登父子合谋离间吕布和袁术。'),
('zang_ba', '臧霸', 'Male', 'expansion_003,history,xuzhou,official', 'official', 'tao_qian', 'xiapi', 'tao_qian', 'RulerSubject', '臧霸为泰山豪强出身的将领，先附吕布，后归曹操并长期镇守青徐。'),
('li_jue', '李傕', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '李傕为董卓旧部，董卓死后攻入长安，挟持汉献帝并与郭汜争权。'),
('guo_si', '郭汜', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '郭汜为董卓旧将，参与攻取长安，后与李傕内斗使关中大乱。'),
('niu_fu', '牛辅', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '牛辅为董卓女婿和部将，董卓死后部众溃散，自己也被部下所杀。'),
('fan_chou', '樊稠', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '樊稠为董卓旧将，随李傕等攻入长安，后因关中军阀内斗被杀。'),
('xu_rong', '徐荣', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '徐荣为董卓军将领，曾在汴水、梁东一带击败曹操和孙坚等关东军。'),
('hua_xiong', '华雄', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '华雄为董卓军武将，史书载其为孙坚所斩，演义中则因关羽温酒斩华雄而知名。'),
('hu_zhen', '胡轸', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '胡轸为董卓部将，曾与吕布、华雄等对抗孙坚，因军中不和失利。'),
('dong_min', '董旻', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '董旻为董卓之弟，董卓败亡后受牵连被诛。'),
('li_su', '李肃', 'Male', 'expansion_003,history,dong_zhuo,general', 'general', 'dong_zhuo', 'changan', 'dong_zhuo', 'RulerSubject', '李肃为董卓部属，演义中以说降吕布、献赤兔马的情节知名。'),
('wang_yun', '王允', 'Male', 'expansion_003,history,han_court,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '王允为司徒，联合吕布刺杀董卓，短暂主持长安朝政后被李傕郭汜所杀。'),
('han_fu', '韩馥', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '韩馥为冀州牧，关东讨董诸侯之一，后将冀州让给袁绍。'),
('qiao_mao', '桥瑁', 'Male', 'expansion_003,history,coalition,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '桥瑁为东郡太守，曾矫诏号召关东诸侯讨董。'),
('bao_xin', '鲍信', 'Male', 'expansion_003,history,coalition,official', 'official', 'han_court', 'luoyang', 'han_xian_di', 'RulerSubject', '鲍信为济北相，早期支持曹操讨董，后来在兖州对黄巾作战中战死。'),
('yuan_yi', '袁遗', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '袁遗为袁氏族人和山阳太守，参与关东联盟，后来死于群雄混战。'),
('xu_you', '许攸', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '许攸为袁绍谋士，官渡之战中投奔曹操，献计袭取乌巢。'),
('guo_tu', '郭图', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '郭图为袁绍谋士，官渡前后多参与河北军政决策，常与沮授、田丰意见相左。'),
('feng_ji', '逢纪', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '逢纪为袁绍谋臣，支持袁尚继位，卷入袁氏诸子内争。'),
('gao_gan', '高干', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '高干为袁绍外甥，任并州刺史，袁氏败亡后据并州反曹。'),
('chunyu_qiong', '淳于琼', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '淳于琼为袁绍将领，官渡之战守乌巢粮仓，被曹操夜袭击败。'),
('han_meng', '韩猛', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '韩猛为袁绍部将，官渡战役中负责运粮和前线作战。'),
('jiang_yiqu', '蒋义渠', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '蒋义渠为袁绍部将，在袁绍官渡败退后收拢残军迎回主帅。'),
('lu_kuang', '吕旷', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '吕旷为袁氏旧将，袁谭、袁尚败后转附曹操。'),
('lu_xiang', '吕翔', 'Male', 'expansion_003,history,yuan_shaos,official', 'official', 'yuan_shao', 'ye', 'yuan_shao', 'RulerSubject', '吕翔为袁氏旧将，与吕旷一同归附曹操，见于河北平定过程。'),
('gongsun_du', '公孙度', 'Male', 'expansion_003,history,northern,general', 'general', 'gongsun_zan', 'ji', 'gongsun_zan', 'RulerSubject', '公孙度割据辽东，自立威权，奠定公孙氏在东北的地方政权。'),
('gongsun_kang', '公孙康', 'Male', 'expansion_003,history,northern,general', 'general', 'gongsun_zan', 'ji', 'gongsun_zan', 'RulerSubject', '公孙康继承辽东，斩送袁尚、袁熙首级给曹操以保全割据。'),
('gongsun_gong', '公孙恭', 'Male', 'expansion_003,history,northern,general', 'general', 'gongsun_zan', 'ji', 'gongsun_zan', 'RulerSubject', '公孙恭为辽东公孙氏继承者之一，后被侄子公孙渊夺权。'),
('gongsun_yuan', '公孙渊', 'Male', 'expansion_003,history,northern,general', 'general', 'gongsun_zan', 'ji', 'gongsun_zan', 'RulerSubject', '公孙渊据辽东叛魏，最终被司马懿远征平定。'),
('yan_gang', '严纲', 'Male', 'expansion_003,history,northern,general', 'general', 'gongsun_zan', 'ji', 'gongsun_zan', 'RulerSubject', '严纲为公孙瓒部将，参与幽州和河北战事。'),
('tian_kai', '田楷', 'Male', 'expansion_003,history,northern,general', 'general', 'gongsun_zan', 'ji', 'gongsun_zan', 'RulerSubject', '田楷为公孙瓒所置青州刺史，曾与袁绍势力争夺青州。'),
('zhou_cang', '周仓', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '周仓为演义和民间传说中的关羽部将，以忠勇追随关羽形象流传。'),
('guan_ping', '关平', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '关平为关羽之子或义子，随关羽镇守荆州，败亡时一同被杀。'),
('guan_xing', '关兴', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '关兴为关羽之子，蜀汉年轻将领，演义中与张苞并称。'),
('guan_suo', '关索', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '关索为后世关羽传说中的子嗣人物，常见于南中和民间三国故事。'),
('guan_tong', '关统', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '关统为关羽后裔，承袭汉寿亭侯爵位。'),
('guan_yi', '关彝', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '关彝为关羽后裔，蜀汉灭亡时遭庞会报复牵连。'),
('liu_feng', '刘封', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '刘封为刘备养子，曾镇守上庸，因未援关羽和失地被刘备赐死。'),
('liu_yong', '刘永', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '刘永为刘备之子，蜀汉宗室，封鲁王后改封甘陵王。'),
('liu_li', '刘理', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '刘理为刘备之子，蜀汉宗室，封梁王后改封安平王。'),
('liu_ba', '刘巴', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '刘巴为蜀汉尚书令，长于财政文书，刘备入蜀后参与制度建设。'),
('li_yan', '李严', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '李严又名李平，为刘备托孤大臣之一，后因北伐粮运失误被废。'),
('ma_liang', '马良', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '马良字季常，眉有白毛，以才名辅佐刘备，夷陵之战后遇害。'),
('ma_su', '马谡', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '马谡为马良之弟，深受诸葛亮信任，街亭失守后被处置。'),
('yang_yi', '杨仪', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '杨仪为蜀汉长史，诸葛亮死后整军退还，与魏延矛盾激化。'),
('fei_yi', '费祎', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '费祎为蜀汉后期执政重臣，继蒋琬之后主持国政，后遇刺身亡。'),
('jiang_wan', '蒋琬', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '蒋琬为蜀汉丞相府重臣，诸葛亮死后接掌政务并稳定朝局。'),
('dong_yun', '董允', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '董允为蜀汉侍中，以直谏约束后主近臣，与诸葛亮、蒋琬、费祎并受称誉。'),
('chen_zhen', '陈震', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '陈震为蜀汉外交官，曾出使孙吴，参与维系吴蜀联盟。'),
('deng_zhi', '邓芝', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '邓芝为蜀汉重臣，奉命出使孙吴，促成刘备死后的吴蜀复交。'),
('liao_hua', '廖化', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '廖化为蜀汉老将，从关羽部曲延续至蜀汉末年，长期参与北伐和边防。'),
('wang_ping', '王平', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '王平为蜀汉将领，街亭失利时保全军势，后镇守汉中。'),
('wu_ban', '吴班', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '吴班为蜀汉将领，随刘备伐吴并参与诸葛亮北伐。'),
('wu_yi', '吴懿', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '吴懿为蜀汉外戚和将领，穆皇后之兄，曾参与北伐。'),
('ma_zhong_shu', '马忠', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '马忠为蜀汉南中重臣，长期镇抚南中并处理地方叛乱。'),
('huang_quan', '黄权', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '黄权原为刘璋、刘备部属，夷陵战后归魏，仍以忠于故主见称。'),
('meng_da', '孟达', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '孟达由刘璋转事刘备，后降魏又谋反，最终被司马懿迅速平定。'),
('huo_jun', '霍峻', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '霍峻为刘备将领，守葭萌关以少御众，战后受刘备嘉奖。'),
('huo_yi', '霍弋', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '霍弋为霍峻之子，蜀汉末年镇守南中，入晋后仍任地方官。'),
('fu_qian', '傅佥', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '傅佥为蜀汉末年将领，魏军伐蜀时守阳安关，力战而死。'),
('zhuge_zhan', '诸葛瞻', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '诸葛瞻为诸葛亮之子，蜀汉末年在绵竹抵抗邓艾而战死。'),
('zhuge_shang', '诸葛尚', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '诸葛尚为诸葛瞻之子，随父在绵竹抗魏战死。'),
('zhuge_jun', '诸葛均', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '诸葛均为诸葛亮之弟，后入蜀仕官，是诸葛氏家族成员。'),
('mi_fang', '糜芳', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '糜芳为糜竺之弟，曾任南郡太守，关羽北伐时降吴导致荆州失守。'),
('fu_shiren', '傅士仁', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '傅士仁为关羽麾下将领，荆州危局中与糜芳一同降吴。'),
('liu_yan_shu', '刘琰', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '刘琰为蜀汉旧臣，性格放诞，后因家事获罪被处死。'),
('xiang_lang', '向朗', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '向朗为荆州名士，入蜀后任文职重臣，曾因马谡事件受牵连。'),
('xiang_chong', '向宠', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '向宠为蜀汉将领，诸葛亮在《出师表》中称其性行淑均、晓畅军事。'),
('qiao_zhou', '谯周', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '谯周为蜀汉学者和官员，魏军入蜀时主张刘禅投降。'),
('xi_zheng', '郤正', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '郤正为蜀汉文臣，蜀亡后随刘禅入洛，以文辞和应对见称。'),
('luo_xian', '罗宪', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '罗宪为蜀汉巴东守将，蜀亡后坚守永安抵御吴军。'),
('gao_xiang', '高翔', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '高翔为蜀汉将领，参与诸葛亮北伐，常与魏延、吴班等同列。'),
('chen_shi_shu', '陈式', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '陈式为蜀汉将领，参与汉中和北伐战事，曾因作战失利受责。'),
('feng_xi', '冯习', 'Male', 'expansion_003,history,shu_han,official', 'official', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '冯习为刘备伐吴时将领，夷陵之战中战死。'),
('sha_moke', '沙摩柯', 'Male', 'expansion_003,romance,nanzhong,general', 'general', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '沙摩柯为武陵蛮首领，刘备伐吴时助蜀作战，夷陵败后被杀。'),
('meng_huo', '孟获', 'Male', 'expansion_003,romance,nanzhong,general', 'general', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '孟获为南中叛乱首领，演义中以诸葛亮七擒七纵故事最为知名。'),
('dailai_dongzhu', '带来洞主', 'Male', 'expansion_003,romance,nanzhong,general', 'general', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '带来洞主为《三国演义》南中人物，常与孟获阵营和祝融夫人故事相连。'),
('mang_yachang', '忙牙长', 'Male', 'expansion_003,romance,nanzhong,general', 'general', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '忙牙长为《三国演义》南中武将，随孟获对抗诸葛亮南征。'),
('duosi_dawang', '朵思大王', 'Male', 'expansion_003,romance,nanzhong,general', 'general', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '朵思大王为《三国演义》南中洞主，以秃龙洞毒泉情节知名。'),
('wutugu', '兀突骨', 'Male', 'expansion_003,romance,nanzhong,general', 'general', 'liu_bei', 'chengdu', 'liu_bei', 'RulerSubject', '兀突骨为《三国演义》乌戈国主，率藤甲军助孟获抗蜀。'),
('cao_ang', '曹昂', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹昂为曹操长子，宛城之战中让马救父，自己遇害。'),
('cao_zhang', '曹彰', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹彰为曹操之子，勇武善战，因黄须形象和北征乌桓余部知名。'),
('cao_zhi', '曹植', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹植为曹操之子，建安文学代表人物，曾与曹丕争夺继承权。'),
('cao_xiong', '曹熊', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹熊为曹操之子，早卒，常作为曹氏宗室谱系人物出现。'),
('cao_chong', '曹冲', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹冲为曹操幼子，以称象故事和早慧形象知名，十三岁早卒。'),
('cao_yu', '曹宇', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹宇为曹操之子，魏明帝临终时曾被考虑参与辅政。'),
('cao_rui', '曹叡', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹叡为魏明帝，曹丕之子，在位时抗衡蜀吴并大兴宫室。'),
('cao_fang', '曹芳', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹芳为曹魏第三任皇帝，高平陵之变后受司马氏控制，后被废。'),
('cao_mao', '曹髦', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹髦为曹魏皇帝，不甘司马昭专权，亲率宫中兵讨伐而死。'),
('cao_huan', '曹奂', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹奂为曹魏末代皇帝，最终禅位司马炎。'),
('cao_zhen', '曹真', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹真为曹魏宗室大将，长期负责西线防务，对抗诸葛亮北伐。'),
('cao_shuang', '曹爽', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹爽为曹真之子，魏明帝托孤辅政大臣，高平陵之变中败亡。'),
('cao_xiu', '曹休', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹休为曹魏宗室名将，长期统领东线，石亭之战为陆逊所败。'),
('cao_chun', '曹纯', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹纯为曹操族弟，统领虎豹骑，参与北方统一战争。'),
('cao_tai', '曹泰', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹泰为曹真之子，曹魏宗室将领，承袭曹真一支军政地位。'),
('cao_zhao', '曹肇', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹肇为曹魏宗室，明帝末年曾进入辅政安排但未能掌权。'),
('cao_xun', '曹训', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹训为曹爽之弟，高平陵之变后随曹爽集团被诛。'),
('cao_xi', '曹羲', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '曹羲为曹爽之弟，曾任曹魏重职，高平陵之变后被司马懿诛杀。'),
('xiahou_shang', '夏侯尚', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '夏侯尚为曹魏宗室名将，曹丕亲信，曾镇守荆襄方向。'),
('xiahou_ba', '夏侯霸', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '夏侯霸为夏侯渊之子，因司马氏掌权而投奔蜀汉，参与北伐。'),
('xiahou_wei', '夏侯威', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '夏侯威为夏侯渊之子，曹魏宗室将领，与司马氏有姻亲往来。'),
('xiahou_xuan', '夏侯玄', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '夏侯玄为曹魏名士和官员，反对司马氏专权失败后被杀。'),
('xiahou_hui_male', '夏侯惠', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '夏侯惠为夏侯渊后裔，曹魏宗室文士，以才学见称。'),
('xiahou_he', '夏侯和', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '夏侯和为夏侯渊后裔，曹魏官员，入晋后仍任职。'),
('sima_lang', '司马朗', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马朗为司马懿之兄，早年仕曹操，以政务和地方治理见称。'),
('sima_fu', '司马孚', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马孚为司马懿之弟，历仕魏晋，虽助司马氏仍以魏臣自处。'),
('sima_shi', '司马师', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马师为司马懿长子，执掌曹魏大权，废曹芳并镇压毌丘俭文钦。'),
('sima_zhao', '司马昭', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马昭为司马懿次子，灭蜀前后掌握魏政，其权势为西晋建立铺路。'),
('sima_yan', '司马炎', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马炎为晋武帝，受魏禅建立西晋，并最终灭吴统一。'),
('sima_you', '司马攸', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马攸为司马昭之子、司马炎之弟，封齐王，以声望和宗室地位受关注。'),
('sima_zhou', '司马伷', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马伷为司马懿之子，西晋宗室，参与魏晋之际军事政治。'),
('sima_wang', '司马望', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '司马望为司马氏宗族将领，魏晋之际镇守关中和西线。'),
('chen_qun', '陈群', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '陈群为曹魏重臣，创设九品中正制，奠定魏晋选官制度。'),
('zhong_yao', '钟繇', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '钟繇为曹魏元老重臣和书法名家，长期参与朝廷中枢政务。'),
('zhong_hui', '钟会', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '钟会为钟繇之子，参与灭蜀，后在成都谋反失败。'),
('deng_ai', '邓艾', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '邓艾为曹魏名将，偷渡阴平灭蜀，随后与钟会内争中被杀。'),
('deng_zhong', '邓忠', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '邓忠为邓艾之子，随父参与灭蜀，后同遭钟会乱局牵连。'),
('guo_huai', '郭淮', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '郭淮为曹魏西线名将，长期抵御蜀汉北伐并经营雍凉。'),
('hao_zhao', '郝昭', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '郝昭为曹魏将领，陈仓之战中坚守城池击退诸葛亮。'),
('wang_lang', '王朗', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '王朗为汉末魏初重臣，曾任会稽太守，后入魏官至司徒。'),
('wang_su', '王肃', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '王肃为曹魏经学家和官员，王元姬之父，参与礼制学术争论。'),
('wang_can', '王粲', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '王粲为建安七子之一，依附曹操后参与文学和幕府文书。'),
('liu_ye', '刘晔', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '刘晔为曹魏谋臣，出身汉室宗亲，以识见和战略建议著称。'),
('dong_zhao', '董昭', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '董昭为曹操谋臣，推动曹操受魏公、魏王封号。'),
('jiang_ji', '蒋济', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '蒋济为曹魏谋臣，历仕曹操至曹芳时期，参与高平陵前后政局。'),
('hua_xin', '华歆', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '华歆为汉魏重臣，曹丕代汉时位居三公，声望很高。'),
('jia_kui', '贾逵', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '贾逵为曹魏地方和军事重臣，治理豫州并参与对吴战事。'),
('xin_pi', '辛毗', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '辛毗为曹魏谏臣，原属袁绍阵营，后归曹操并以直言见称。'),
('xin_ping', '辛评', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '辛评为袁绍谋士，辛毗之兄，卷入袁谭、袁尚继承纷争。'),
('he_yan', '何晏', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '何晏为曹魏玄学名士，依附曹爽，高平陵之变后被司马懿诛杀。'),
('guanqiu_jian', '毌丘俭', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '毌丘俭为曹魏将领，曾征高句丽，后起兵反对司马师失败。'),
('wen_pin', '文聘', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '文聘原为刘表部将，后归曹操，长期镇守江夏抵御孙吴。'),
('wen_qin', '文钦', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '文钦为曹魏将领，随毌丘俭反司马氏，失败后投奔孙吴。'),
('wen_yang', '文鸯', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '文鸯为文钦之子，以勇武闻名，先后经历魏、吴、晋政权。'),
('wang_shuang', '王双', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '王双为曹魏猛将，演义中与诸葛亮北伐战事相关，后被魏延所斩。'),
('li_tong', '李通', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '李通为曹操早期将领，汝南一带归附曹操并屡立战功。'),
('sun_li', '孙礼', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '孙礼为曹魏将领和官员，历仕多朝，以刚直和边防经验见称。'),
('qian_zhao', '牵招', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '牵招为曹魏北边将领，长期经营幽并边郡并安抚胡汉。'),
('tian_yu', '田豫', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '田豫曾从公孙瓒、刘备，后仕曹魏，长期镇守北疆。'),
('sun_yi', '孙翊', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙翊为孙坚之子、孙权之弟，任丹杨太守时遇刺身亡。'),
('sun_kuang', '孙匡', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙匡为孙坚之子、孙权之弟，早卒，属江东孙氏宗室。'),
('sun_lang', '孙朗', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙朗为孙坚庶子，孙吴宗室，曾因军中失火受罚。'),
('sun_deng', '孙登', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙登为孙权长子和太子，以仁厚和礼贤名士著称，早逝使吴国继承局势转坏。'),
('sun_he', '孙和', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙和为孙权太子，二宫之争中被废，后来被孙峻赐死。'),
('sun_ba', '孙霸', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙霸为孙权鲁王，与太子孙和相争，引发孙吴二宫之争。'),
('sun_liang', '孙亮', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙亮为孙权幼子、吴少帝，受权臣辅政，后被废为会稽王。'),
('sun_xiu', '孙休', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙休为吴景帝，诛杀孙綝后亲政，重用濮阳兴、张布等人。'),
('sun_hao', '孙皓', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙皓为孙吴末代皇帝，暴虐失政，最终向西晋投降。'),
('sun_yu', '孙瑜', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙瑜为孙静之子，孙吴宗室将领，早年随孙权经营江东。'),
('sun_jiao', '孙皎', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙皎为孙静之子，孙吴宗室将领，曾督江夏方向军务。'),
('sun_huan', '孙桓', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙桓为孙河之子，孙吴宗室将领，夷陵之战中据守夷道。'),
('sun_shao', '孙韶', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '孙韶为孙河养子，孙吴宗室将领，长期守卫吴郡和建业外围。'),
('zhuge_jin', '诸葛瑾', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '诸葛瑾为诸葛亮之兄，仕孙权，以宽厚稳重和吴蜀沟通见称。'),
('zhuge_ke', '诸葛恪', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '诸葛恪为诸葛瑾之子，孙权托孤重臣，东兴大捷后因专权失败被杀。'),
('zhuge_rong', '诸葛融', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '诸葛融为诸葛瑾之子、诸葛恪之弟，随诸葛恪败亡受诛。'),
('lu_shu', '鲁淑', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '鲁淑为鲁肃之子，继承父业仕吴，官至昭武将军。'),
('lv_fan', '吕范', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '吕范为孙氏元老重臣，早年追随孙策，后掌吴国财赋和军政。'),
('lv_ju', '吕据', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '吕据为吕范之子，孙吴后期将领，反对孙綝专权失败。'),
('zhu_zhi', '朱治', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '朱治为孙氏老臣，抚养孙权并长期治理吴郡。'),
('zhu_ran', '朱然', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '朱然本姓施，为朱治养子，孙吴名将，江陵防御战中成名。'),
('zhu_huan', '朱桓', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '朱桓为孙吴将领，濡须等战役中多次抵御曹魏。'),
('zhu_ju', '朱据', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '朱据为孙吴将领，娶孙鲁育，卷入二宫之争后被害。'),
('gu_yong', '顾雍', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '顾雍为江东顾氏名臣，长期任吴国丞相，以沉稳清正著称。'),
('gu_tan', '顾谭', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '顾谭为顾雍之孙，孙吴才俊，二宫之争中遭流放。'),
('bu_zhi', '步骘', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '步骘为孙吴重臣，治理交州、荆州等地，后官至丞相。'),
('bu_xie', '步协', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '步协为步骘之子，继承临湘侯爵位并仕吴。'),
('pan_zhang', '潘璋', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '潘璋为孙吴将领，参与擒杀关羽，后在江陵、夷陵等战事中活跃。'),
('ling_cao', '凌操', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '凌操为孙氏将领，在江夏攻黄祖时战死。'),
('ling_tong', '凌统', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '凌统为凌操之子，孙吴勇将，合肥战役中护卫孙权突围。'),
('xu_sheng', '徐盛', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '徐盛为孙吴名将，曾以疑城计震慑曹丕南征。'),
('ding_feng', '丁奉', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '丁奉为孙吴后期名将，从孙权时代战至吴末，参与诛杀孙綝。'),
('quan_cong', '全琮', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '全琮为孙吴将领，娶孙鲁班，后来成为全氏外戚核心。'),
('quan_yi', '全怿', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '全怿为全琮族人，孙吴后期将领，后来降魏。'),
('jiang_qin', '蒋钦', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '蒋钦为孙策、孙权麾下水军将领，以清廉和勇战见称。'),
('chen_wu', '陈武', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '陈武为孙吴将领，合肥之战中力战而死。'),
('dong_xi', '董袭', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '董袭为孙氏将领，讨伐山越和水战有功，濡须口风浪中殉职。'),
('he_qi', '贺齐', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '贺齐为孙吴将领，长期平定山越和江东地方叛乱。'),
('lu_kang', '陆抗', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '陆抗为陆逊之子，孙吴末期名将，长期对峙西晋羊祜。'),
('lu_kai', '陆凯', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '陆凯为孙吴后期重臣，敢于劝谏孙皓，留下直言形象。'),
('yu_fan', '虞翻', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '虞翻为会稽名士，仕孙权，以经学、占候和刚直言辞著称。'),
('yan_jun', '严畯', 'Male', 'expansion_003,history,eastern_wu,official', 'official', 'sun_quan', 'jianye', 'sun_quan', 'RulerSubject', '严畯为江东学者型官员，与诸葛瑾、步骘等交好，以儒雅见称。'),
('zhao_ang', '赵昂', 'Male', 'expansion_003,history,liangzhou,official', 'official', 'ma_teng', 'wuwei', 'ma_teng', 'RulerSubject', '赵昂为凉州官员，马超作乱时与妻王异坚守冀城并抗击马超。'),
('zhao_fan', '赵范', 'Male', 'expansion_003,history,jingzhou,official', 'official', 'liu_biao', 'xiangyang', 'liu_biao', 'RulerSubject', '赵范为桂阳太守，刘备平定荆南后归附，演义中有为赵云议婚情节。'),
('xun_song', '荀崧', 'Male', 'expansion_003,history,cao_wei,official', 'official', 'cao_cao', 'xuchang', 'cao_cao', 'RulerSubject', '荀崧为西晋官员，荀氏名门之后，其女荀灌突围求援的故事最知名。'),
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
