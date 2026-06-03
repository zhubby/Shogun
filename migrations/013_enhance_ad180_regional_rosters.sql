PRAGMA foreign_keys = ON;

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, gender, leadership, strength, intelligence, politics, charm, confidence, biography, notes)
VALUES
('shi_yi_jiaozhou', '士壹', NULL, '交州苍梧郡广信县', 145, NULL, 'Male', 50, 42, 68, 74, 66, 'Medium', '士壹为交州士氏人物，士燮之弟，东汉末年与士氏家族共同经营岭南。', 'ad180 剧本补充人物；用于增强士燮势力班底，生卒年按游戏平衡估算。'),
('shi_wei_jiaozhou', '士䵋', NULL, '交州苍梧郡广信县', 148, NULL, 'Male', 46, 40, 66, 70, 62, 'Medium', '士䵋为交州士氏人物，士燮之弟，代表士氏在交州郡县中的宗族网络。', 'ad180 剧本补充人物；用于增强士燮势力班底，生卒年按游戏平衡估算。'),
('shi_wu_jiaozhou', '士武', NULL, '交州苍梧郡广信县', 151, NULL, 'Male', 54, 52, 58, 62, 60, 'Medium', '士武为交州士氏人物，士燮之弟，补足交州地方武备和郡县控制。', 'ad180 剧本补充人物；用于增强士燮势力班底，生卒年按游戏平衡估算。'),
('shi_hui_jiaozhou', '士徽', NULL, '交州苍梧郡广信县', 165, 226, 'Male', 58, 56, 54, 58, 55, 'Medium', '士徽为士燮之子，士燮死后卷入交州继承与孙吴南方经营。', 'ad180 剧本补充人物；用于增强士燮势力班底，早期登场按游戏平衡处理。'),
('zhao_zhong_eunuch', '赵忠', NULL, '豫州颍川郡', 137, 189, 'Male', 28, 18, 58, 78, 62, 'Medium', '赵忠为汉灵帝时中常侍，与张让并为宦官集团核心，牵动黄巾前夜的朝廷政治。', 'ad180 剧本补充人物；生年按游戏平衡估算。'),
('duan_gui_eunuch', '段珪', NULL, '司隶河南尹', 150, 189, 'Male', 24, 20, 52, 70, 56, 'Medium', '段珪为十常侍人物，灵帝末年卷入宦官与外戚士人的冲突。', 'ad180 剧本补充人物；生年按游戏平衡估算。'),
('cao_jie_eunuch', '曹节', NULL, '荆州南阳郡', 130, 181, 'Male', 26, 18, 60, 82, 64, 'Medium', '曹节为东汉后期权宦，桓灵之际累居要职，是灵帝朝宦官政治的重要人物。', 'ad180 剧本补充人物；为避免与曹魏曹节皇后重名，使用独立 id。'),
('lv_qiang_eunuch', '吕强', NULL, '司隶河南尹', 145, 184, 'Male', 32, 20, 76, 82, 68, 'Medium', '吕强为汉灵帝时宦官，以屡次谏诤、批评奢侈和宦官弊政见称。', 'ad180 剧本补充人物；生年按游戏平衡估算。'),
('yuan_wei_han', '袁隗', '次阳', '豫州汝南郡汝阳县', 140, 190, 'Male', 34, 22, 78, 90, 72, 'High', '袁隗为汝南袁氏重臣，汉末位居三公，亦连接袁绍、袁术等士族网络。', 'ad180 剧本补充人物；生年按游戏平衡估算。'),
('yang_ci_han', '杨赐', '伯献', '司隶弘农郡华阴县', 120, 185, 'Male', 36, 20, 82, 90, 76, 'High', '杨赐为东汉名臣，灵帝时历任三公，并曾就朝政灾异进谏。', 'ad180 剧本补充人物；生年按游戏平衡估算。'),
('yang_biao_han', '杨彪', '文先', '司隶弘农郡华阴县', 142, 225, 'Male', 32, 20, 80, 88, 74, 'High', '杨彪为弘农杨氏名臣，汉末长期在朝，后来见证董卓、曹操与汉魏嬗代。', 'ad180 剧本补充人物。'),
('qiao_xuan_han', '桥玄', '公祖', '梁国睢阳县', 110, 184, 'Male', 42, 28, 82, 88, 78, 'High', '桥玄为东汉名臣，以清正知人闻名，汉末士人政治网络中声望很高。', 'ad180 剧本补充人物。'),
('fu_xie_han', '傅燮', '南容', '凉州北地郡灵州县', 143, 187, 'Male', 74, 70, 72, 68, 70, 'High', '傅燮为东汉凉州名臣，曾参与平定黄巾和西北叛乱，以守节死难著称。', 'ad180 剧本补充人物；生年按游戏平衡估算。'),
('zhang_jun_han', '张钧', NULL, '司隶河南尹', 150, 184, 'Male', 30, 22, 72, 76, 62, 'Medium', '张钧为灵帝时郎中，黄巾事起后曾上书弹劾十常侍与张角交通，旋遭构陷。', 'ad180 剧本补充人物；生年按游戏平衡估算。');

INSERT INTO officer_tags (officer_id, tag_id)
VALUES
('shi_yi_jiaozhou', 'role:administrator'),
('shi_yi_jiaozhou', 'role:official'),
('shi_yi_jiaozhou', 'basis:history'),
('shi_yi_jiaozhou', 'source:manual_curated'),
('shi_wei_jiaozhou', 'role:administrator'),
('shi_wei_jiaozhou', 'role:official'),
('shi_wei_jiaozhou', 'basis:history'),
('shi_wei_jiaozhou', 'source:manual_curated'),
('shi_wu_jiaozhou', 'role:general'),
('shi_wu_jiaozhou', 'role:official'),
('shi_wu_jiaozhou', 'basis:history'),
('shi_wu_jiaozhou', 'source:manual_curated'),
('shi_hui_jiaozhou', 'role:general'),
('shi_hui_jiaozhou', 'role:official'),
('shi_hui_jiaozhou', 'basis:history'),
('shi_hui_jiaozhou', 'source:manual_curated'),
('zhao_zhong_eunuch', 'role:official'),
('zhao_zhong_eunuch', 'affiliation:han_court'),
('zhao_zhong_eunuch', 'basis:history'),
('zhao_zhong_eunuch', 'context:taipingdao'),
('zhao_zhong_eunuch', 'source:manual_curated'),
('duan_gui_eunuch', 'role:official'),
('duan_gui_eunuch', 'affiliation:han_court'),
('duan_gui_eunuch', 'basis:history'),
('duan_gui_eunuch', 'context:taipingdao'),
('duan_gui_eunuch', 'source:manual_curated'),
('cao_jie_eunuch', 'role:official'),
('cao_jie_eunuch', 'affiliation:han_court'),
('cao_jie_eunuch', 'basis:history'),
('cao_jie_eunuch', 'context:taipingdao'),
('cao_jie_eunuch', 'source:manual_curated'),
('lv_qiang_eunuch', 'role:official'),
('lv_qiang_eunuch', 'affiliation:han_court'),
('lv_qiang_eunuch', 'basis:history'),
('lv_qiang_eunuch', 'context:taipingdao'),
('lv_qiang_eunuch', 'source:manual_curated'),
('yuan_wei_han', 'role:official'),
('yuan_wei_han', 'affiliation:han_court'),
('yuan_wei_han', 'basis:history'),
('yuan_wei_han', 'source:manual_curated'),
('yang_ci_han', 'role:official'),
('yang_ci_han', 'affiliation:han_court'),
('yang_ci_han', 'basis:history'),
('yang_ci_han', 'source:manual_curated'),
('yang_biao_han', 'role:official'),
('yang_biao_han', 'affiliation:han_court'),
('yang_biao_han', 'basis:history'),
('yang_biao_han', 'source:manual_curated'),
('qiao_xuan_han', 'role:official'),
('qiao_xuan_han', 'role:scholar'),
('qiao_xuan_han', 'affiliation:han_court'),
('qiao_xuan_han', 'basis:history'),
('qiao_xuan_han', 'source:manual_curated'),
('fu_xie_han', 'role:general'),
('fu_xie_han', 'role:official'),
('fu_xie_han', 'affiliation:han_court'),
('fu_xie_han', 'basis:history'),
('fu_xie_han', 'source:manual_curated'),
('zhang_jun_han', 'role:official'),
('zhang_jun_han', 'affiliation:han_court'),
('zhang_jun_han', 'basis:history'),
('zhang_jun_han', 'context:taipingdao'),
('zhang_jun_han', 'source:manual_curated');

INSERT INTO officer_external_ids
(officer_id, source, external_id, source_url, confidence, notes)
VALUES
('shi_yi_jiaozhou', 'manual_curated', '士壹', '', 'Medium', 'ad180 交州士氏班底补充'),
('shi_wei_jiaozhou', 'manual_curated', '士䵋', '', 'Medium', 'ad180 交州士氏班底补充'),
('shi_wu_jiaozhou', 'manual_curated', '士武', '', 'Medium', 'ad180 交州士氏班底补充'),
('shi_hui_jiaozhou', 'manual_curated', '士徽', '', 'Medium', 'ad180 交州士氏班底补充'),
('zhao_zhong_eunuch', 'manual_curated', '赵忠', '', 'Medium', 'ad180 灵帝朝宦官补充'),
('duan_gui_eunuch', 'manual_curated', '段珪', '', 'Medium', 'ad180 灵帝朝宦官补充'),
('cao_jie_eunuch', 'manual_curated', '曹节宦官', '', 'Medium', 'ad180 灵帝朝宦官补充'),
('lv_qiang_eunuch', 'manual_curated', '吕强', '', 'Medium', 'ad180 灵帝朝宦官补充'),
('yuan_wei_han', 'manual_curated', '袁隗', '', 'High', 'ad180 东汉朝臣补充'),
('yang_ci_han', 'manual_curated', '杨赐', '', 'High', 'ad180 东汉朝臣补充'),
('yang_biao_han', 'manual_curated', '杨彪', '', 'High', 'ad180 东汉朝臣补充'),
('qiao_xuan_han', 'manual_curated', '桥玄', '', 'High', 'ad180 东汉朝臣补充'),
('fu_xie_han', 'manual_curated', '傅燮', '', 'High', 'ad180 东汉朝臣补充'),
('zhang_jun_han', 'manual_curated', '张钧', '', 'Medium', 'ad180 黄巾前夜朝臣补充');

INSERT INTO officer_relationships
(source_officer_id, target_officer_id, relationship_kind, confidence, notes, source)
VALUES
('shi_yi_jiaozhou', 'shi_xie', 'RulerSubject', 'Medium', 'ad180 剧本补充士氏亲族与士燮的从属关系。', 'manual_curated'),
('shi_wei_jiaozhou', 'shi_xie', 'RulerSubject', 'Medium', 'ad180 剧本补充士氏亲族与士燮的从属关系。', 'manual_curated'),
('shi_wu_jiaozhou', 'shi_xie', 'RulerSubject', 'Medium', 'ad180 剧本补充士氏亲族与士燮的从属关系。', 'manual_curated'),
('shi_hui_jiaozhou', 'shi_xie', 'ParentChild', 'Medium', 'ad180 剧本补充士徽与士燮的父子关系。', 'manual_curated'),
('zhao_zhong_eunuch', 'ctk_5218_5b8f', 'RulerSubject', 'Medium', '赵忠为汉灵帝时中常侍。', 'manual_curated'),
('duan_gui_eunuch', 'ctk_5218_5b8f', 'RulerSubject', 'Medium', '段珪为汉灵帝末年宦官人物。', 'manual_curated'),
('cao_jie_eunuch', 'ctk_5218_5b8f', 'RulerSubject', 'Medium', '曹节为桓灵之际权宦。', 'manual_curated'),
('lv_qiang_eunuch', 'ctk_5218_5b8f', 'RulerSubject', 'Medium', '吕强为汉灵帝时宦官。', 'manual_curated'),
('yuan_wei_han', 'ctk_5218_5b8f', 'RulerSubject', 'High', '袁隗为东汉朝臣。', 'manual_curated'),
('yang_ci_han', 'ctk_5218_5b8f', 'RulerSubject', 'High', '杨赐为东汉朝臣。', 'manual_curated'),
('yang_biao_han', 'ctk_5218_5b8f', 'RulerSubject', 'High', '杨彪为东汉朝臣。', 'manual_curated'),
('qiao_xuan_han', 'ctk_5218_5b8f', 'RulerSubject', 'High', '桥玄为东汉朝臣。', 'manual_curated'),
('fu_xie_han', 'ctk_5218_5b8f', 'RulerSubject', 'High', '傅燮为东汉朝臣。', 'manual_curated'),
('zhang_jun_han', 'ctk_5218_5b8f', 'RulerSubject', 'Medium', '张钧为汉灵帝时郎中。', 'manual_curated'),
('yang_biao_han', 'yang_ci_han', 'ParentChild', 'High', '杨彪为杨赐之子。', 'manual_curated');

INSERT OR REPLACE INTO officer_life_events
(id, officer_id, event_year, event_month, event_kind, faction_id, city_id, loyalty, notes)
VALUES
('ad180_zhang_rang_luoyang', 'ctk_5f20_8ba9', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 72, '太平道将兴剧本灵帝朝初始归属'),
('ad180_zhao_zhong_luoyang', 'zhao_zhong_eunuch', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 72, '太平道将兴剧本灵帝朝初始归属'),
('ad180_duan_gui_luoyang', 'duan_gui_eunuch', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 70, '太平道将兴剧本灵帝朝初始归属'),
('ad180_cao_jie_eunuch_luoyang', 'cao_jie_eunuch', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 74, '太平道将兴剧本灵帝朝初始归属'),
('ad180_lv_qiang_luoyang', 'lv_qiang_eunuch', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 78, '太平道将兴剧本灵帝朝初始归属'),
('ad180_he_jin_luoyang', 'he_jin', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 82, '太平道将兴剧本灵帝朝初始归属'),
('ad180_he_miao_luoyang', 'he_miao', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 74, '太平道将兴剧本灵帝朝初始归属'),
('ad180_jian_shuo_luoyang', 'jian_shuo', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 72, '太平道将兴剧本灵帝朝初始归属'),
('ad180_huangfu_song_luoyang', 'huangfu_song', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 88, '太平道将兴剧本讨黄巾将领初始归属'),
('ad180_zhu_jun_luoyang', 'zhu_jun', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 86, '太平道将兴剧本讨黄巾将领初始归属'),
('ad180_lu_zhi_luoyang', 'lu_zhi', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 86, '太平道将兴剧本讨黄巾将领初始归属'),
('ad180_wang_yun_luoyang', 'wang_yun', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 80, '太平道将兴剧本灵帝朝初始归属'),
('ad180_cai_yong_luoyang', 'cai_yong', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 78, '太平道将兴剧本灵帝朝初始归属'),
('ad180_liu_yu_luoyang', 'ctk_5218_865e', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 82, '太平道将兴剧本灵帝朝初始归属'),
('ad180_zhang_wen_luoyang', 'ctk_5f20_6e29_31', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 78, '太平道将兴剧本灵帝朝初始归属'),
('ad180_yuan_wei_luoyang', 'yuan_wei_han', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 82, '太平道将兴剧本灵帝朝初始归属'),
('ad180_yang_ci_luoyang', 'yang_ci_han', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 84, '太平道将兴剧本灵帝朝初始归属'),
('ad180_yang_biao_luoyang', 'yang_biao_han', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 82, '太平道将兴剧本灵帝朝初始归属'),
('ad180_qiao_xuan_luoyang', 'qiao_xuan_han', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 82, '太平道将兴剧本灵帝朝初始归属'),
('ad180_fu_xie_luoyang', 'fu_xie_han', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 84, '太平道将兴剧本讨黄巾将领初始归属'),
('ad180_zhang_jun_luoyang', 'zhang_jun_han', 180, 1, 'ServeFaction', 'han_court', 'luoyang', 76, '太平道将兴剧本灵帝朝初始归属'),

('ad180_dong_zhuo_anding', 'dong_zhuo', 180, 1, 'ServeFaction', 'dong_zhuo', 'anding', 90, '太平道将兴剧本初始归属'),
('ad180_li_ru_tianshui', 'li_ru', 180, 1, 'ServeFaction', 'dong_zhuo', 'tianshui', 84, '太平道将兴剧本初始归属'),
('ad180_li_jue_anding', 'li_jue', 180, 1, 'ServeFaction', 'dong_zhuo', 'anding', 78, '太平道将兴剧本初始归属'),
('ad180_guo_si_anding', 'guo_si', 180, 1, 'ServeFaction', 'dong_zhuo', 'anding', 76, '太平道将兴剧本初始归属'),
('ad180_niu_fu_longxi', 'niu_fu', 180, 1, 'ServeFaction', 'dong_zhuo', 'longxi', 78, '太平道将兴剧本初始归属'),
('ad180_fan_chou_tianshui', 'fan_chou', 180, 1, 'ServeFaction', 'dong_zhuo', 'tianshui', 74, '太平道将兴剧本初始归属'),
('ad180_xu_rong_tianshui', 'xu_rong', 180, 1, 'ServeFaction', 'dong_zhuo', 'tianshui', 82, '太平道将兴剧本初始归属'),
('ad180_hua_xiong_longxi', 'hua_xiong', 180, 1, 'ServeFaction', 'dong_zhuo', 'longxi', 76, '太平道将兴剧本初始归属'),
('ad180_hu_zhen_longxi', 'hu_zhen', 180, 1, 'ServeFaction', 'dong_zhuo', 'longxi', 72, '太平道将兴剧本初始归属'),
('ad180_dong_min_anding', 'dong_min', 180, 1, 'ServeFaction', 'dong_zhuo', 'anding', 74, '太平道将兴剧本初始归属'),
('ad180_li_su_tianshui', 'li_su', 180, 1, 'ServeFaction', 'dong_zhuo', 'tianshui', 70, '太平道将兴剧本初始归属'),

('ad180_sun_jian_wu', 'sun_jian', 180, 1, 'ServeFaction', 'sun_quan', 'wu', 92, '太平道将兴剧本初始归属'),
('ad180_zhu_zhi_jianye', 'zhu_zhi', 180, 1, 'ServeFaction', 'sun_quan', 'jianye', 80, '太平道将兴剧本初始归属'),
('ad180_lv_fan_wu', 'lv_fan', 180, 1, 'ServeFaction', 'sun_quan', 'wu', 78, '太平道将兴剧本初始归属'),
('ad180_ling_cao_danyang', 'ling_cao', 180, 1, 'ServeFaction', 'sun_quan', 'danyang', 76, '太平道将兴剧本初始归属'),
('ad180_jiang_qin_lujiang', 'jiang_qin', 180, 1, 'ServeFaction', 'sun_quan', 'lujiang', 74, '太平道将兴剧本初始归属'),
('ad180_chen_wu_hefei', 'chen_wu', 180, 1, 'ServeFaction', 'sun_quan', 'hefei', 74, '太平道将兴剧本初始归属'),
('ad180_dong_xi_danyang', 'dong_xi', 180, 1, 'ServeFaction', 'sun_quan', 'danyang', 72, '太平道将兴剧本初始归属'),
('ad180_he_qi_kuaiji', 'he_qi', 180, 1, 'ServeFaction', 'sun_quan', 'kuaiji', 72, '太平道将兴剧本初始归属'),
('ad180_xu_sheng_yuzhang', 'xu_sheng', 180, 1, 'ServeFaction', 'sun_quan', 'yuzhang', 70, '太平道将兴剧本初始归属'),

('ad180_gongsun_zan_youbeiping', 'gongsun_zan', 180, 1, 'ServeFaction', 'gongsun_zan', 'youbeiping', 88, '太平道将兴剧本初始归属'),
('ad180_yan_gang_youbeiping', 'yan_gang', 180, 1, 'ServeFaction', 'gongsun_zan', 'youbeiping', 76, '太平道将兴剧本初始归属'),
('ad180_tian_kai_liaoxi', 'tian_kai', 180, 1, 'ServeFaction', 'gongsun_zan', 'liaoxi', 76, '太平道将兴剧本初始归属'),
('ad180_gongsun_du_xiangping', 'gongsun_du', 180, 1, 'ServeFaction', 'gongsun_zan', 'xiangping', 78, '太平道将兴剧本初始归属'),
('ad180_gongsun_kang_xiangping', 'gongsun_kang', 180, 1, 'ServeFaction', 'gongsun_zan', 'xiangping', 70, '太平道将兴剧本初始归属'),
('ad180_gongsun_gong_liaoxi', 'gongsun_gong', 180, 1, 'ServeFaction', 'gongsun_zan', 'liaoxi', 70, '太平道将兴剧本初始归属'),

('ad180_tao_qian_xiapi', 'tao_qian', 180, 1, 'ServeFaction', 'tao_qian', 'xiapi', 88, '太平道将兴剧本初始归属'),
('ad180_chen_deng_pengcheng', 'chen_deng', 180, 1, 'ServeFaction', 'tao_qian', 'pengcheng', 82, '太平道将兴剧本初始归属'),
('ad180_chen_gui_donghai', 'chen_gui', 180, 1, 'ServeFaction', 'tao_qian', 'donghai', 80, '太平道将兴剧本初始归属'),
('ad180_zang_ba_langya', 'zang_ba', 180, 1, 'ServeFaction', 'tao_qian', 'langya', 76, '太平道将兴剧本初始归属'),
('ad180_mi_zhu_guangling', 'mi_zhu', 180, 1, 'ServeFaction', 'tao_qian', 'guangling', 78, '太平道将兴剧本初始归属'),
('ad180_mi_fang_xiapi', 'mi_fang', 180, 1, 'ServeFaction', 'tao_qian', 'xiapi', 72, '太平道将兴剧本初始归属'),

('ad180_liu_biao_xiangyang', 'liu_biao', 180, 1, 'ServeFaction', 'liu_biao', 'xiangyang', 88, '太平道将兴剧本初始归属'),
('ad180_kuai_liang_jiangling', 'kuai_liang', 180, 1, 'ServeFaction', 'liu_biao', 'jiangling', 84, '太平道将兴剧本初始归属'),
('ad180_kuai_yue_xiangyang', 'kuai_yue', 180, 1, 'ServeFaction', 'liu_biao', 'xiangyang', 84, '太平道将兴剧本初始归属'),
('ad180_huang_zu_jiangxia', 'huang_zu', 180, 1, 'ServeFaction', 'liu_biao', 'jiangxia', 78, '太平道将兴剧本初始归属'),
('ad180_zhao_fan_changsha', 'zhao_fan', 180, 1, 'ServeFaction', 'liu_biao', 'changsha', 74, '太平道将兴剧本初始归属'),
('ad180_wen_pin_nanyang', 'wen_pin', 180, 1, 'ServeFaction', 'liu_biao', 'nanyang', 80, '太平道将兴剧本初始归属'),
('ad180_liu_qi_jiangxia', 'ctk_5218_7426', 180, 1, 'ServeFaction', 'liu_biao', 'jiangxia', 76, '太平道将兴剧本初始归属'),
('ad180_zhang_yun_nanyang', 'ctk_5f20_5141', 180, 1, 'ServeFaction', 'liu_biao', 'nanyang', 76, '太平道将兴剧本初始归属'),

('ad180_ma_teng_wuwei', 'ma_teng', 180, 1, 'ServeFaction', 'ma_teng', 'wuwei', 88, '太平道将兴剧本初始归属'),
('ad180_han_sui_jiuquan', 'han_sui', 180, 1, 'ServeFaction', 'ma_teng', 'jiuquan', 82, '太平道将兴剧本初始归属'),
('ad180_zhao_ang_dunhuang', 'zhao_ang', 180, 1, 'ServeFaction', 'ma_teng', 'dunhuang', 76, '太平道将兴剧本初始归属'),
('ad180_wang_yi_wuwei', 'wang_yi', 180, 1, 'ServeFaction', 'ma_teng', 'wuwei', 78, '太平道将兴剧本初始归属'),
('ad180_ma_dai_jiuquan', 'ma_dai', 180, 1, 'ServeFaction', 'ma_teng', 'jiuquan', 72, '太平道将兴剧本初始归属'),

('ad180_liu_zhang_zitong', 'liu_zhang', 180, 1, 'ServeFaction', 'liu_yan', 'zitong', 82, '太平道将兴剧本初始归属'),
('ad180_yan_yan_jiangzhou', 'yan_yan', 180, 1, 'ServeFaction', 'liu_yan', 'jiangzhou', 80, '太平道将兴剧本初始归属'),
('ad180_zhang_ren_hanzhong', 'zhang_ren', 180, 1, 'ServeFaction', 'liu_yan', 'hanzhong', 76, '太平道将兴剧本初始归属'),
('ad180_zhang_song_yongan', 'zhang_song', 180, 1, 'ServeFaction', 'liu_yan', 'yongan', 74, '太平道将兴剧本初始归属'),
('ad180_zhang_su_chengdu', 'ctk_5f20_8083', 180, 1, 'ServeFaction', 'liu_yan', 'chengdu', 74, '太平道将兴剧本初始归属'),

('ad180_shi_xie_jiaozhi', 'shi_xie', 180, 1, 'ServeFaction', 'shi_xie', 'jiaozhi', 88, '太平道将兴剧本初始归属'),
('ad180_shi_yi_nanhai', 'shi_yi_jiaozhou', 180, 1, 'ServeFaction', 'shi_xie', 'nanhai', 80, '太平道将兴剧本初始归属'),
('ad180_shi_wei_cangwu', 'shi_wei_jiaozhou', 180, 1, 'ServeFaction', 'shi_xie', 'cangwu', 78, '太平道将兴剧本初始归属'),
('ad180_shi_wu_yulin', 'shi_wu_jiaozhou', 180, 1, 'ServeFaction', 'shi_xie', 'yulin', 76, '太平道将兴剧本初始归属'),
('ad180_shi_hui_jiaozhi', 'shi_hui_jiaozhou', 180, 1, 'ServeFaction', 'shi_xie', 'jiaozhi', 72, '太平道将兴剧本初始归属'),

('ad181_die_cao_jie_eunuch', 'cao_jie_eunuch', 181, 12, 'Die', NULL, NULL, NULL, 'ad180 补充人物离场'),
('ad184_die_lv_qiang', 'lv_qiang_eunuch', 184, 12, 'Die', NULL, NULL, NULL, 'ad180 补充人物离场'),
('ad184_die_qiao_xuan', 'qiao_xuan_han', 184, 12, 'Die', NULL, NULL, NULL, 'ad180 补充人物离场'),
('ad184_die_zhang_jun', 'zhang_jun_han', 184, 12, 'Die', NULL, NULL, NULL, 'ad180 补充人物离场'),
('ad185_die_yang_ci', 'yang_ci_han', 185, 12, 'Die', NULL, NULL, NULL, 'ad180 补充人物离场'),
('ad187_die_fu_xie', 'fu_xie_han', 187, 12, 'Die', NULL, NULL, NULL, 'ad180 补充人物离场'),
('ad189_die_zhang_rang', 'ctk_5f20_8ba9', 189, 12, 'Die', NULL, NULL, NULL, 'ad180 补充张让离场'),
('ad189_die_zhao_zhong', 'zhao_zhong_eunuch', 189, 12, 'Die', NULL, NULL, NULL, 'ad180 补充人物离场'),
('ad189_die_duan_gui', 'duan_gui_eunuch', 189, 12, 'Die', NULL, NULL, NULL, 'ad180 补充人物离场'),
('ad190_die_yuan_wei', 'yuan_wei_han', 190, 12, 'Die', NULL, NULL, NULL, 'ad180 补充人物离场'),
('ad225_die_yang_biao', 'yang_biao_han', 225, 12, 'Die', NULL, NULL, NULL, 'ad180 补充人物离场'),
('ad226_die_shi_hui', 'shi_hui_jiaozhou', 226, 12, 'Die', NULL, NULL, NULL, 'ad180 补充人物离场'),

('ad190_restore_liu_zhang_chengdu', 'liu_zhang', 190, 1, 'ServeFaction', 'liu_zhang', 'chengdu', 80, '太平道将兴剧本后续剧本归属恢复'),
('ad190_restore_yan_yan_chengdu', 'yan_yan', 190, 1, 'ServeFaction', 'liu_zhang', 'chengdu', 78, '太平道将兴剧本后续剧本归属恢复');

UPDATE scenario_city_states
SET governor_id = CASE city_id
        WHEN 'anding' THEN 'dong_zhuo'
        WHEN 'tianshui' THEN 'li_ru'
        WHEN 'longxi' THEN 'niu_fu'
        WHEN 'wuwei' THEN 'ma_teng'
        WHEN 'jiuquan' THEN 'han_sui'
        WHEN 'dunhuang' THEN 'zhao_ang'
        WHEN 'youbeiping' THEN 'gongsun_zan'
        WHEN 'liaoxi' THEN 'tian_kai'
        WHEN 'xiangping' THEN 'gongsun_du'
        WHEN 'xiapi' THEN 'tao_qian'
        WHEN 'pengcheng' THEN 'chen_deng'
        WHEN 'langya' THEN 'zang_ba'
        WHEN 'guangling' THEN 'mi_zhu'
        WHEN 'donghai' THEN 'chen_gui'
        WHEN 'nanyang' THEN 'wen_pin'
        WHEN 'xiangyang' THEN 'liu_biao'
        WHEN 'jiangling' THEN 'kuai_liang'
        WHEN 'jiangxia' THEN 'huang_zu'
        WHEN 'changsha' THEN 'zhao_fan'
        WHEN 'chengdu' THEN 'ctk_5218_7109'
        WHEN 'zitong' THEN 'liu_zhang'
        WHEN 'jiangzhou' THEN 'yan_yan'
        WHEN 'hanzhong' THEN 'zhang_ren'
        WHEN 'yongan' THEN 'zhang_song'
        WHEN 'jianye' THEN 'zhu_zhi'
        WHEN 'danyang' THEN 'ling_cao'
        WHEN 'wu' THEN 'sun_jian'
        WHEN 'lujiang' THEN 'jiang_qin'
        WHEN 'kuaiji' THEN 'he_qi'
        WHEN 'yuzhang' THEN 'xu_sheng'
        WHEN 'jiaozhi' THEN 'shi_xie'
        WHEN 'nanhai' THEN 'shi_yi_jiaozhou'
        WHEN 'cangwu' THEN 'shi_wei_jiaozhou'
        WHEN 'yulin' THEN 'shi_wu_jiaozhou'
        ELSE governor_id
    END
WHERE scenario_id = 'ad180'
  AND city_id IN (
      'anding',
      'tianshui',
      'longxi',
      'wuwei',
      'jiuquan',
      'dunhuang',
      'youbeiping',
      'liaoxi',
      'xiangping',
      'xiapi',
      'pengcheng',
      'langya',
      'guangling',
      'donghai',
      'nanyang',
      'xiangyang',
      'jiangling',
      'jiangxia',
      'changsha',
      'chengdu',
      'zitong',
      'jiangzhou',
      'hanzhong',
      'yongan',
      'jianye',
      'danyang',
      'wu',
      'lujiang',
      'kuaiji',
      'yuzhang',
      'jiaozhi',
      'nanhai',
      'cangwu',
      'yulin'
  );
