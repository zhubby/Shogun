PRAGMA foreign_keys = ON;

CREATE TABLE cities (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    province TEXT NOT NULL,
    commandery TEXT NOT NULL,
    x REAL NOT NULL,
    y REAL NOT NULL,
    scale TEXT NOT NULL CHECK (scale IN ('County', 'Commandery', 'RegionalCapital', 'ImperialCapital')),
    strategic_rank INTEGER NOT NULL CHECK (strategic_rank BETWEEN 1 AND 10),
    agriculture_base INTEGER NOT NULL,
    commerce_base INTEGER NOT NULL,
    defense_base INTEGER NOT NULL,
    population_min INTEGER NOT NULL,
    population_max INTEGER NOT NULL,
    confidence TEXT NOT NULL CHECK (confidence IN ('High', 'Medium', 'Low')),
    notes TEXT NOT NULL DEFAULT ''
);

CREATE TABLE factions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    default_ruler_id TEXT NOT NULL,
    color_r REAL NOT NULL,
    color_g REAL NOT NULL,
    color_b REAL NOT NULL
);

CREATE TABLE officers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    courtesy_name TEXT,
    native_place TEXT,
    birth_year INTEGER,
    death_year INTEGER,
    leadership INTEGER NOT NULL CHECK (leadership BETWEEN 1 AND 100),
    strength INTEGER NOT NULL CHECK (strength BETWEEN 1 AND 100),
    intelligence INTEGER NOT NULL CHECK (intelligence BETWEEN 1 AND 100),
    politics INTEGER NOT NULL CHECK (politics BETWEEN 1 AND 100),
    charm INTEGER NOT NULL CHECK (charm BETWEEN 1 AND 100),
    tags TEXT NOT NULL DEFAULT '',
    confidence TEXT NOT NULL CHECK (confidence IN ('High', 'Medium', 'Low')),
    notes TEXT NOT NULL DEFAULT ''
);

CREATE TABLE roads (
    from_city_id TEXT NOT NULL REFERENCES cities(id),
    to_city_id TEXT NOT NULL REFERENCES cities(id),
    PRIMARY KEY (from_city_id, to_city_id),
    CHECK (from_city_id <> to_city_id)
);

CREATE TABLE scenarios (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    year INTEGER NOT NULL,
    month INTEGER NOT NULL CHECK (month BETWEEN 1 AND 12)
);

CREATE TABLE scenario_faction_states (
    scenario_id TEXT NOT NULL REFERENCES scenarios(id),
    faction_id TEXT NOT NULL REFERENCES factions(id),
    exists_in_scenario INTEGER NOT NULL CHECK (exists_in_scenario IN (0, 1)),
    selectable INTEGER NOT NULL CHECK (selectable IN (0, 1)),
    ruler_id TEXT NOT NULL REFERENCES officers(id),
    PRIMARY KEY (scenario_id, faction_id)
);

CREATE TABLE scenario_city_states (
    scenario_id TEXT NOT NULL REFERENCES scenarios(id),
    city_id TEXT NOT NULL REFERENCES cities(id),
    faction_id TEXT NOT NULL REFERENCES factions(id),
    population INTEGER NOT NULL,
    gold INTEGER NOT NULL,
    food INTEGER NOT NULL,
    troops INTEGER NOT NULL,
    training INTEGER NOT NULL CHECK (training BETWEEN 0 AND 100),
    agriculture INTEGER NOT NULL,
    commerce INTEGER NOT NULL,
    defense INTEGER NOT NULL,
    city_order INTEGER NOT NULL CHECK (city_order BETWEEN 0 AND 100),
    governor_id TEXT REFERENCES officers(id),
    PRIMARY KEY (scenario_id, city_id)
);

CREATE TABLE officer_life_events (
    id TEXT PRIMARY KEY,
    officer_id TEXT NOT NULL REFERENCES officers(id),
    event_year INTEGER NOT NULL,
    event_month INTEGER NOT NULL CHECK (event_month BETWEEN 1 AND 12),
    event_kind TEXT NOT NULL CHECK (event_kind IN ('Appear', 'ServeFaction', 'MoveToCity', 'BecomeUnavailable', 'Die')),
    faction_id TEXT REFERENCES factions(id),
    city_id TEXT REFERENCES cities(id),
    notes TEXT NOT NULL DEFAULT ''
);

CREATE TABLE scenario_diplomacy (
    scenario_id TEXT NOT NULL REFERENCES scenarios(id),
    faction_a TEXT NOT NULL REFERENCES factions(id),
    faction_b TEXT NOT NULL REFERENCES factions(id),
    score INTEGER NOT NULL CHECK (score BETWEEN -100 AND 100),
    truce_until_turn INTEGER,
    PRIMARY KEY (scenario_id, faction_a, faction_b),
    CHECK (faction_a <> faction_b)
);

CREATE INDEX idx_roads_from ON roads(from_city_id);
CREATE INDEX idx_roads_to ON roads(to_city_id);
CREATE INDEX idx_scenario_city_states_scenario ON scenario_city_states(scenario_id);
CREATE INDEX idx_scenario_city_states_faction ON scenario_city_states(faction_id);
CREATE INDEX idx_scenario_faction_states_scenario ON scenario_faction_states(scenario_id);
CREATE INDEX idx_officer_life_events_date ON officer_life_events(event_year, event_month);
CREATE INDEX idx_officer_life_events_officer ON officer_life_events(officer_id);
CREATE INDEX idx_officers_name ON officers(name);
CREATE INDEX idx_cities_province ON cities(province, commandery);

INSERT INTO scenarios (id, name, year, month) VALUES
('ad190', '初平元年 讨董余波', 190, 1),
('ad200', '建安五年 官渡前夜', 200, 1),
('ad208', '建安十三年 赤壁前夜', 208, 1),
('ad220', '延康元年 魏晋序幕', 220, 1);

INSERT INTO factions (id, name, default_ruler_id, color_r, color_g, color_b) VALUES
('han_court', '汉室', 'han_xian_di', 0.72, 0.62, 0.36),
('dong_zhuo', '董卓军', 'dong_zhuo', 0.30, 0.28, 0.28),
('cao_cao', '曹操军', 'cao_cao', 0.82, 0.22, 0.18),
('liu_bei', '刘备军', 'liu_bei', 0.24, 0.47, 0.96),
('sun_quan', '孙氏军', 'sun_quan', 0.95, 0.66, 0.18),
('yuan_shao', '袁绍军', 'yuan_shao', 0.48, 0.34, 0.75),
('yuan_shu', '袁术军', 'yuan_shu', 0.56, 0.42, 0.20),
('liu_biao', '刘表军', 'liu_biao', 0.28, 0.62, 0.48),
('liu_zhang', '刘璋军', 'liu_zhang', 0.22, 0.55, 0.35),
('ma_teng', '马腾军', 'ma_teng', 0.62, 0.38, 0.26),
('zhang_lu', '张鲁军', 'zhang_lu', 0.54, 0.58, 0.32),
('gongsun_zan', '公孙瓒军', 'gongsun_zan', 0.72, 0.72, 0.76),
('lu_bu', '吕布军', 'lu_bu', 0.66, 0.16, 0.52),
('tao_qian', '陶谦军', 'tao_qian', 0.46, 0.50, 0.42),
('shi_xie', '士燮军', 'shi_xie', 0.36, 0.66, 0.64);

INSERT INTO cities
(id, name, province, commandery, x, y, scale, strategic_rank, agriculture_base, commerce_base, defense_base, population_min, population_max, confidence, notes)
VALUES
('luoyang','洛阳','司隶','河南尹',-210,-80,'ImperialCapital',10,220,330,300,180000,380000,'High','东汉旧都，战略与政治权重最高'),
('changan','长安','司隶','京兆尹',-410,-95,'ImperialCapital',10,230,280,320,160000,360000,'High','关中核心，汉末多次成为政治中心'),
('hongnong','弘农','司隶','弘农郡',-320,-85,'Commandery',7,190,180,240,80000,180000,'Medium','洛阳与关中之间要道'),
('henei','河内','司隶','河内郡',-190,30,'Commandery',7,210,190,210,90000,200000,'Medium','黄河北岸，连接冀司'),
('ye','邺','冀州','魏郡',-105,190,'RegionalCapital',9,260,250,230,160000,340000,'High','河北重镇，袁曹争衡核心'),
('nanpi','南皮','冀州','渤海郡',15,245,'Commandery',7,230,190,180,100000,220000,'Medium','渤海郡治，袁绍早期根基'),
('pingyuan','平原','冀州','平原郡',-245,180,'Commandery',6,190,140,125,70000,150000,'Medium','刘备早期任所之一'),
('ganling','甘陵','冀州','清河国',-45,130,'Commandery',6,210,160,160,80000,170000,'Medium','冀州腹地节点'),
('zhongshan','中山','冀州','中山国',-95,300,'Commandery',6,210,170,170,85000,190000,'Medium','河北北部要地'),
('hejian','河间','冀州','河间国',45,315,'Commandery',6,200,170,160,80000,180000,'Medium','冀幽之间节点'),
('ji','蓟','幽州','广阳郡',-40,430,'RegionalCapital',8,170,180,210,90000,210000,'High','幽州治所，北方战略节点'),
('zhuo','涿','幽州','涿郡',-115,370,'Commandery',6,175,155,165,70000,160000,'Medium','刘备、张飞等人物籍贯区域'),
('youbeiping','右北平','幽州','右北平郡',120,455,'Commandery',6,145,115,190,45000,110000,'Medium','北境防线'),
('liaoxi','辽西','幽州','辽西郡',250,430,'Commandery',5,130,110,170,35000,90000,'Medium','辽西节点'),
('xiangping','襄平','幽州','辽东郡',420,430,'RegionalCapital',6,160,150,180,70000,160000,'Medium','辽东公孙氏核心'),
('jinyang','晋阳','并州','太原郡',-260,310,'RegionalCapital',7,190,170,220,80000,190000,'Medium','并州核心'),
('shangdang','上党','并州','上党郡',-225,190,'Commandery',7,170,140,240,65000,150000,'Medium','太行山要冲'),
('xihenorth','西河','并州','西河郡',-355,230,'Commandery',5,145,110,180,40000,100000,'Low','并州西部节点'),
('wuwei','武威','凉州','武威郡',-720,155,'RegionalCapital',7,150,135,210,60000,150000,'Medium','凉州核心郡'),
('jiuquan','酒泉','凉州','酒泉郡',-880,135,'Commandery',5,120,110,170,30000,90000,'Medium','河西走廊重镇'),
('dunhuang','敦煌','凉州','敦煌郡',-1030,125,'Commandery',4,95,115,150,20000,70000,'Medium','西域门户'),
('tianshui','天水','凉州','汉阳郡',-560,50,'Commandery',7,170,145,205,70000,160000,'High','陇右重镇'),
('anding','安定','凉州','安定郡',-510,150,'Commandery',6,150,120,190,45000,110000,'Medium','关陇之间'),
('longxi','陇西','凉州','陇西郡',-600,-30,'Commandery',6,145,115,185,45000,110000,'Medium','陇右西南节点'),
('chenliu','陈留','兖州','陈留郡',-70,-10,'Commandery',8,230,240,190,130000,280000,'High','中原交通核心'),
('puyang','濮阳','兖州','东郡',20,85,'Commandery',7,215,200,175,100000,220000,'Medium','曹操吕布争夺地'),
('jiyin','济阴','兖州','济阴郡',35,10,'Commandery',6,200,170,160,80000,180000,'Medium','兖州腹地'),
('shanyang','山阳','兖州','山阳郡',75,-55,'Commandery',6,205,170,160,85000,190000,'Medium','鲁豫之间'),
('dongping','东平','兖州','东平国',105,55,'Commandery',5,190,150,150,70000,160000,'Medium','兖州东部'),
('xuchang','许昌','豫州','颍川郡',-90,-150,'RegionalCapital',9,255,320,230,150000,320000,'High','曹操迎帝后政治中心'),
('yingchuan','颍川','豫州','颍川郡',-135,-170,'Commandery',8,240,300,200,130000,280000,'High','名士辈出，中原核心'),
('runan','汝南','豫州','汝南郡',-45,-235,'Commandery',8,260,230,175,140000,300000,'High','豫州大郡'),
('qiao','谯','豫州','沛国',40,-190,'Commandery',7,220,200,170,100000,220000,'High','曹氏故里区域'),
('chen','陈','豫州','陈国',20,-270,'Commandery',6,210,180,160,90000,200000,'Medium','豫州东南节点'),
('xiapi','下邳','徐州','下邳国',165,-115,'RegionalCapital',8,230,190,170,100000,230000,'High','徐州核心，吕布刘备争夺地'),
('pengcheng','彭城','徐州','彭城国',120,-150,'Commandery',7,220,180,165,90000,210000,'High','徐州重镇'),
('langya','琅琊','徐州','琅琊国',245,-65,'Commandery',6,190,160,150,75000,170000,'Medium','徐州北部沿海'),
('guangling','广陵','徐州','广陵郡',270,-245,'Commandery',7,210,210,160,90000,210000,'Medium','江淮门户'),
('donghai','东海','徐州','东海郡',245,-140,'Commandery',6,195,165,155,80000,180000,'Medium','徐州东部'),
('linzi','临淄','青州','齐国',175,150,'RegionalCapital',7,220,220,165,100000,230000,'Medium','齐地旧都'),
('beihai','北海','青州','北海国',250,135,'Commandery',6,205,160,140,80000,180000,'Medium','孔融曾治北海'),
('donglai','东莱','青州','东莱郡',390,125,'Commandery',5,180,170,135,65000,150000,'Medium','山东半岛东部'),
('jimo','即墨','青州','齐郡',315,80,'County',4,170,150,120,45000,110000,'Low','齐地沿海节点'),
('nanyang','宛','荆州','南阳郡',-210,-300,'RegionalCapital',8,260,240,190,140000,310000,'High','南阳大郡'),
('xiangyang','襄阳','荆州','南郡',-160,-430,'RegionalCapital',9,260,240,240,130000,300000,'High','荆北要冲'),
('jiangling','江陵','荆州','南郡',-95,-510,'RegionalCapital',8,280,230,210,120000,270000,'High','荆州南郡核心'),
('jiangxia','江夏','荆州','江夏郡',45,-460,'Commandery',7,245,160,150,80000,190000,'Medium','江汉之间'),
('changsha','长沙','荆州','长沙郡',75,-650,'Commandery',7,245,165,140,85000,200000,'Medium','荆南重郡'),
('wuling','武陵','荆州','武陵郡',-115,-695,'Commandery',5,210,130,130,55000,140000,'Medium','荆南西部'),
('guiyang','桂阳','荆州','桂阳郡',100,-760,'Commandery',5,210,125,120,50000,130000,'Medium','荆南节点'),
('lingling','零陵','荆州','零陵郡',-25,-785,'Commandery',5,215,120,125,52000,135000,'Medium','荆南南部'),
('chengdu','成都','益州','蜀郡',-505,-520,'RegionalCapital',9,310,260,230,150000,330000,'High','益州核心'),
('zitong','梓潼','益州','梓潼郡',-465,-410,'Commandery',6,230,150,170,65000,150000,'Medium','入蜀北道'),
('jiangzhou','江州','益州','巴郡',-355,-560,'Commandery',7,240,165,180,80000,190000,'Medium','巴郡核心'),
('yongan','永安','益州','巴东郡',-250,-555,'Commandery',6,200,130,220,50000,130000,'Medium','三峡门户'),
('jiangyang','江阳','益州','犍为郡',-520,-650,'Commandery',5,220,135,145,55000,140000,'Medium','益南节点'),
('fuling','涪陵','益州','涪陵郡',-315,-635,'Commandery',5,190,120,145,42000,110000,'Medium','巴东南节点'),
('hanzhong','汉中','益州','汉中郡',-405,-285,'RegionalCapital',8,235,170,240,90000,210000,'High','秦蜀汉中门户'),
('shouchun','寿春','扬州','九江郡',125,-330,'RegionalCapital',8,230,210,180,100000,230000,'High','淮南核心'),
('lujiang','庐江','扬州','庐江郡',150,-405,'Commandery',7,225,190,165,85000,190000,'Medium','江淮间节点'),
('jianye','建业','扬州','丹阳郡',310,-435,'RegionalCapital',9,250,270,190,120000,280000,'High','孙吴后期核心'),
('danyang','丹阳','扬州','丹阳郡',270,-480,'Commandery',7,240,220,175,95000,220000,'Medium','江东重郡'),
('wu','吴','扬州','吴郡',405,-465,'RegionalCapital',8,260,260,170,120000,270000,'High','江东富庶核心'),
('kuaiji','会稽','扬州','会稽郡',445,-570,'Commandery',7,250,230,160,100000,230000,'Medium','江东南部'),
('yuzhang','豫章','扬州','豫章郡',170,-565,'Commandery',6,240,160,145,75000,180000,'Medium','江西腹地'),
('hefei','合肥','扬州','庐江郡',200,-350,'Commandery',8,220,180,210,80000,190000,'High','魏吴争夺前线'),
('jiaozhi','交趾','交州','交趾郡',105,-1040,'RegionalCapital',6,230,190,130,70000,170000,'Medium','交州核心'),
('nanhai','南海','交州','南海郡',230,-910,'Commandery',6,220,220,130,80000,190000,'Medium','岭南沿海'),
('cangwu','苍梧','交州','苍梧郡',45,-875,'Commandery',5,200,145,125,50000,130000,'Medium','岭南西北'),
('yulin','郁林','交州','郁林郡',-35,-935,'Commandery',4,190,125,120,40000,110000,'Low','交州西部节点');

INSERT INTO roads (from_city_id, to_city_id) VALUES
('changan','hongnong'),('hongnong','luoyang'),('luoyang','henei'),('henei','ye'),('ye','nanpi'),('ye','ganling'),('ganling','pingyuan'),('ganling','zhongshan'),('zhongshan','hejian'),('hejian','ji'),('ji','zhuo'),('ji','youbeiping'),('youbeiping','liaoxi'),('liaoxi','xiangping'),
('henei','shangdang'),('shangdang','jinyang'),('jinyang','xihenorth'),('changan','anding'),('anding','wuwei'),('wuwei','jiuquan'),('jiuquan','dunhuang'),('changan','tianshui'),('tianshui','longxi'),('tianshui','hanzhong'),
('luoyang','chenliu'),('chenliu','puyang'),('puyang','dongping'),('dongping','linzi'),('linzi','beihai'),('beihai','donglai'),('donglai','jimo'),('chenliu','jiyin'),('jiyin','shanyang'),('shanyang','pengcheng'),
('chenliu','yingchuan'),('yingchuan','xuchang'),('xuchang','runan'),('runan','chen'),('chen','qiao'),('qiao','pengcheng'),('pengcheng','xiapi'),('xiapi','donghai'),('donghai','langya'),('xiapi','guangling'),
('xuchang','nanyang'),('nanyang','xiangyang'),('xiangyang','jiangling'),('jiangling','jiangxia'),('jiangxia','shouchun'),('shouchun','hefei'),('hefei','lujiang'),('lujiang','jianye'),('jianye','danyang'),('danyang','wu'),('wu','kuaiji'),('jianye','yuzhang'),
('jiangling','wuling'),('wuling','lingling'),('lingling','guiyang'),('guiyang','changsha'),('changsha','jiangxia'),('jiangling','yongan'),('yongan','jiangzhou'),('jiangzhou','chengdu'),('chengdu','zitong'),('zitong','hanzhong'),('chengdu','jiangyang'),('jiangzhou','fuling'),
('yuzhang','nanhai'),('nanhai','cangwu'),('cangwu','yulin'),('yulin','jiaozhi');

INSERT INTO officers
(id, name, courtesy_name, native_place, birth_year, death_year, leadership, strength, intelligence, politics, charm, tags, confidence, notes)
VALUES
('han_xian_di','汉献帝','伯和','河南洛阳',181,234,24,18,62,70,68,'emperor', 'High','东汉末代皇帝'),
('dong_zhuo','董卓','仲颖','陇西临洮',132,192,82,86,61,45,38,'warlord', 'High','控制朝廷，后为吕布所杀'),
('li_ru','李儒',NULL,'凉州',150,198,42,24,88,76,45,'adviser', 'Medium','董卓谋士'),
('lu_bu','吕布','奉先','五原九原',156,199,86,100,42,26,62,'warrior,warlord', 'High','汉末名将'),
('zhang_liao','张辽','文远','雁门马邑',169,222,92,89,78,64,82,'general', 'High','曹魏名将'),
('gao_shun','高顺',NULL,'并州',160,198,84,80,62,45,58,'general', 'Medium','吕布部将'),
('chen_gong','陈宫','公台','东郡武阳',154,199,56,36,89,78,66,'adviser', 'High','吕布谋士'),
('cao_cao','曹操','孟德','沛国谯县',155,220,96,72,91,94,92,'ruler,poet', 'High','曹魏奠基者'),
('cao_pi','曹丕','子桓','沛国谯县',187,226,72,65,82,85,78,'ruler', 'High','魏文帝'),
('cao_ren','曹仁','子孝','沛国谯县',168,223,88,84,70,62,76,'general', 'High','曹魏宗室名将'),
('cao_hong','曹洪','子廉','沛国谯县',170,232,75,76,58,50,65,'general', 'High','曹魏宗室'),
('xiahou_dun','夏侯惇','元让','沛国谯县',157,220,87,90,62,58,78,'general', 'High','曹魏元老'),
('xiahou_yuan','夏侯渊','妙才','沛国谯县',158,219,86,88,60,48,70,'general', 'High','曹魏名将'),
('xun_yu','荀彧','文若','颍川颍阴',163,212,48,28,96,98,85,'adviser', 'High','王佐之才'),
('xun_you','荀攸','公达','颍川颍阴',157,214,52,30,95,86,74,'adviser', 'High','曹操谋主'),
('guo_jia','郭嘉','奉孝','颍川阳翟',170,207,52,24,98,84,74,'adviser', 'High','曹操早期谋士'),
('jia_xu','贾诩','文和','武威姑臧',147,223,62,32,97,88,65,'adviser', 'High','汉末三国谋士'),
('cheng_yu','程昱','仲德','东郡东阿',141,220,49,30,90,86,70,'adviser', 'High','曹魏谋臣'),
('man_chong','满宠','伯宁','山阳昌邑',170,242,78,70,82,78,70,'general,administrator', 'High','曹魏重臣'),
('yu_jin','于禁','文则','泰山钜平',159,221,84,78,62,46,58,'general', 'High','五子良将'),
('yue_jin','乐进','文谦','阳平卫国',160,218,80,82,56,45,66,'general', 'High','五子良将'),
('zhang_he','张郃','儁乂','河间鄚县',167,231,89,86,76,58,78,'general', 'High','五子良将'),
('xu_huang','徐晃','公明','河东杨县',169,227,88,84,68,56,70,'general', 'High','五子良将'),
('li_dian','李典','曼成','山阳钜野',174,209,74,72,76,66,72,'general', 'High','曹魏将领'),
('xu_chu','许褚','仲康','谯国谯县',170,230,72,96,36,24,62,'warrior', 'High','曹操宿卫名将'),
('dian_wei','典韦',NULL,'陈留己吾',160,197,66,97,32,20,58,'warrior', 'High','曹操宿卫'),
('sima_yi','司马懿','仲达','河内温县',179,251,93,63,98,94,84,'adviser,general', 'High','西晋奠基者'),
('liu_bei','刘备','玄德','涿郡涿县',161,223,76,72,78,80,99,'ruler', 'High','蜀汉昭烈帝'),
('guan_yu','关羽','云长','河东解县',160,220,93,97,75,62,86,'general', 'High','蜀汉名将'),
('zhang_fei','张飞','益德','涿郡',167,221,88,98,35,28,66,'general', 'High','蜀汉名将'),
('zhao_yun','赵云','子龙','常山真定',168,229,91,96,78,65,88,'general', 'High','蜀汉名将'),
('zhuge_liang','诸葛亮','孔明','琅琊阳都',181,234,92,38,100,98,92,'adviser,administrator', 'High','蜀汉丞相'),
('pang_tong','庞统','士元','襄阳',179,214,68,34,97,82,78,'adviser', 'High','凤雏'),
('fa_zheng','法正','孝直','扶风郿县',176,220,54,32,94,86,70,'adviser', 'High','刘备入蜀谋主'),
('jian_yong','简雍','宪和','涿郡',160,230,42,38,70,73,76,'diplomat', 'Medium','刘备旧臣'),
('mi_zhu','糜竺','子仲','东海朐县',165,221,36,30,68,78,80,'administrator', 'High','刘备重臣'),
('sun_qian','孙乾','公祐','北海',165,215,35,31,72,76,72,'diplomat', 'Medium','刘备幕僚'),
('chen_dao','陈到','叔至','汝南',170,230,75,78,60,45,68,'general', 'Medium','蜀汉将领'),
('ma_chao','马超','孟起','扶风茂陵',176,222,88,95,55,38,82,'general', 'High','蜀汉五虎将'),
('huang_zhong','黄忠','汉升','南阳',148,220,86,92,62,44,74,'general', 'High','蜀汉五虎将'),
('wei_yan','魏延','文长','义阳',175,234,86,88,70,48,66,'general', 'High','蜀汉将领'),
('jiang_wei','姜维','伯约','天水冀县',202,264,88,84,89,67,78,'general', 'High','蜀汉后期名将'),
('liu_shan','刘禅','公嗣','涿郡',207,271,28,22,45,55,54,'ruler', 'High','蜀汉后主'),
('sun_jian','孙坚','文台','吴郡富春',155,191,88,92,68,55,78,'ruler,general', 'High','孙氏奠基者'),
('sun_ce','孙策','伯符','吴郡富春',175,200,92,93,76,66,88,'ruler,general', 'High','小霸王'),
('sun_quan','孙权','仲谋','吴郡富春',182,252,78,68,82,89,91,'ruler', 'High','吴大帝'),
('zhou_yu','周瑜','公瑾','庐江舒县',175,210,94,72,96,86,90,'general,adviser', 'High','东吴名将'),
('lu_su','鲁肃','子敬','临淮东城',172,217,54,38,88,92,84,'adviser', 'High','东吴战略家'),
('lu_xun','陆逊','伯言','吴郡吴县',183,245,95,64,96,88,84,'general,adviser', 'High','东吴名将'),
('zhang_zhao','张昭','子布','彭城',156,236,38,22,84,94,77,'administrator', 'High','东吴重臣'),
('zhang_hong','张纮','子纲','广陵',153,212,35,22,83,90,74,'administrator', 'High','东吴重臣'),
('huang_gai','黄盖','公覆','零陵泉陵',154,215,82,84,66,52,76,'general', 'High','东吴老将'),
('cheng_pu','程普','德谋','右北平土垠',155,215,81,80,63,55,74,'general', 'High','孙氏老将'),
('gan_ning','甘宁','兴霸','巴郡临江',175,222,86,91,64,42,78,'general', 'High','东吴名将'),
('taishi_ci','太史慈','子义','东莱黄县',166,206,84,93,67,48,80,'general', 'High','东吴名将'),
('zhou_tai','周泰','幼平','九江下蔡',170,225,78,88,52,35,70,'general', 'Medium','东吴将领'),
('yuan_shao','袁绍','本初','汝南汝阳',154,202,82,70,70,74,86,'ruler', 'High','河北群雄'),
('yuan_tan','袁谭','显思','汝南汝阳',173,205,63,62,48,42,55,'ruler', 'Medium','袁绍长子'),
('yuan_shang','袁尚','显甫','汝南汝阳',177,207,58,56,45,40,52,'ruler', 'Medium','袁绍幼子'),
('ju_shou','沮授',NULL,'广平',150,200,50,28,91,88,75,'adviser', 'High','袁绍谋士'),
('tian_feng','田丰','元皓','钜鹿',150,200,46,24,92,86,68,'adviser', 'High','袁绍谋士'),
('shen_pei','审配','正南','魏郡阴安',155,204,54,35,82,80,62,'administrator', 'High','袁绍臣'),
('yan_liang','颜良',NULL,'河北',160,200,78,92,40,28,64,'general', 'Medium','袁绍部将'),
('wen_chou','文丑',NULL,'河北',160,200,77,91,42,30,63,'general', 'Medium','袁绍部将'),
('gao_lan','高览',NULL,'河北',165,210,76,79,55,42,60,'general', 'Medium','袁绍曹魏将领'),
('yuan_shu','袁术','公路','汝南汝阳',155,199,62,55,54,48,62,'ruler', 'High','淮南群雄'),
('ji_ling','纪灵',NULL,'汝南',160,199,74,82,48,38,58,'general', 'Medium','袁术部将'),
('liu_biao','刘表','景升','山阳高平',142,208,62,45,80,86,82,'ruler', 'High','荆州牧'),
('kuai_liang','蒯良','子柔','南郡中庐',155,210,38,24,84,88,74,'adviser', 'Medium','刘表谋臣'),
('kuai_yue','蒯越','异度','南郡中庐',155,214,40,25,86,86,72,'adviser', 'Medium','刘表谋臣'),
('huang_zu','黄祖',NULL,'江夏',150,208,72,70,52,44,52,'general', 'Medium','刘表部将'),
('liu_zhang','刘璋','季玉','江夏竟陵',160,219,42,36,58,62,60,'ruler', 'High','益州牧'),
('zhang_ren','张任',NULL,'蜀郡',165,213,82,85,70,46,66,'general', 'Medium','刘璋部将'),
('yan_yan','严颜',NULL,'巴郡',150,220,76,80,64,50,68,'general', 'Medium','益州老将'),
('ma_teng','马腾','寿成','扶风茂陵',156,212,78,84,55,46,72,'ruler', 'High','西凉群雄'),
('han_sui','韩遂','文约','金城',150,215,76,70,78,68,65,'ruler', 'High','西凉群雄'),
('ma_dai','马岱',NULL,'扶风',180,235,74,82,60,42,68,'general', 'Medium','马氏将领'),
('zhang_lu','张鲁','公祺','沛国丰县',160,216,50,42,70,82,76,'ruler', 'High','汉中五斗米道领袖'),
('yang_song','杨松',NULL,'汉中',165,215,22,18,48,44,30,'official', 'Low','张鲁部下'),
('gongsun_zan','公孙瓒','伯珪','辽西令支',155,199,80,82,54,42,68,'ruler,general', 'High','幽州群雄'),
('zhao_yun_early','赵云从兄',NULL,'常山',165,205,45,48,42,36,45,'supplemental', 'Low','低置信度补充人物'),
('shi_xie','士燮','威彦','苍梧广信',137,226,48,36,78,86,80,'ruler,administrator', 'High','交州士氏领袖'),
('kong_rong','孔融','文举','鲁国',153,208,35,20,82,80,86,'official,scholar', 'High','北海相'),
('tao_qian','陶谦','恭祖','丹阳',132,194,52,42,66,74,70,'ruler', 'High','徐州牧'),
('liu_yao','刘繇','正礼','东莱牟平',157,197,50,40,68,76,70,'ruler', 'High','扬州刺史');

INSERT INTO scenario_faction_states (scenario_id, faction_id, exists_in_scenario, selectable, ruler_id)
SELECT s.id, f.id,
       CASE
           WHEN s.id = 'ad220' AND f.id IN ('dong_zhuo','yuan_shao','yuan_shu','liu_biao','liu_zhang','ma_teng','zhang_lu','gongsun_zan','lu_bu') THEN 0
           WHEN s.id = 'ad208' AND f.id IN ('dong_zhuo','yuan_shu','gongsun_zan','lu_bu') THEN 0
           WHEN s.id = 'ad200' AND f.id IN ('dong_zhuo') THEN 0
           ELSE 1
       END,
       CASE
           WHEN s.id = 'ad190' AND f.id IN ('cao_cao','liu_bei','yuan_shao','yuan_shu','liu_biao','liu_zhang','ma_teng','gongsun_zan') THEN 1
           WHEN s.id = 'ad200' AND f.id IN ('cao_cao','liu_bei','sun_quan','yuan_shao','liu_biao','liu_zhang','ma_teng') THEN 1
           WHEN s.id = 'ad208' AND f.id IN ('cao_cao','liu_bei','sun_quan','liu_biao','liu_zhang','ma_teng') THEN 1
           WHEN s.id = 'ad220' AND f.id IN ('cao_cao','liu_bei','sun_quan') THEN 1
           ELSE 0
       END,
       CASE
           WHEN s.id = 'ad220' AND f.id = 'cao_cao' THEN 'cao_pi'
           WHEN s.id IN ('ad208','ad220') AND f.id = 'sun_quan' THEN 'sun_quan'
           WHEN f.id = 'sun_quan' AND s.id = 'ad190' THEN 'sun_jian'
           WHEN f.id = 'sun_quan' AND s.id = 'ad200' THEN 'sun_ce'
           ELSE f.default_ruler_id
       END
FROM scenarios s CROSS JOIN factions f;

INSERT INTO scenario_city_states
(scenario_id, city_id, faction_id, population, gold, food, troops, training, agriculture, commerce, defense, city_order, governor_id)
SELECT s.id,
       c.id,
       CASE
           WHEN s.id = 'ad190' AND c.id IN ('luoyang','changan','hongnong','henei') THEN 'dong_zhuo'
           WHEN s.id = 'ad190' AND c.id = 'pingyuan' THEN 'liu_bei'
           WHEN s.id = 'ad190' AND c.province IN ('冀州','青州') THEN 'yuan_shao'
           WHEN s.id = 'ad190' AND c.province IN ('幽州') THEN 'gongsun_zan'
           WHEN s.id = 'ad190' AND c.province IN ('兖州','豫州') THEN 'cao_cao'
           WHEN s.id = 'ad190' AND c.province = '徐州' THEN 'tao_qian'
           WHEN s.id = 'ad190' AND c.province = '荆州' THEN 'liu_biao'
           WHEN s.id = 'ad190' AND c.province = '益州' THEN 'liu_zhang'
           WHEN s.id = 'ad190' AND c.province = '凉州' THEN 'ma_teng'
           WHEN s.id = 'ad190' AND c.province = '扬州' THEN 'yuan_shu'
           WHEN s.id = 'ad190' AND c.province = '交州' THEN 'shi_xie'
           WHEN s.id = 'ad200' AND c.id IN ('pingyuan','xiapi') THEN 'liu_bei'
           WHEN s.id = 'ad200' AND c.province IN ('冀州','青州') THEN 'yuan_shao'
           WHEN s.id = 'ad200' AND c.province IN ('兖州','豫州','司隶','徐州') THEN 'cao_cao'
           WHEN s.id = 'ad200' AND c.province = '荆州' THEN 'liu_biao'
           WHEN s.id = 'ad200' AND c.province = '益州' THEN 'liu_zhang'
           WHEN s.id = 'ad200' AND c.province = '凉州' THEN 'ma_teng'
           WHEN s.id = 'ad200' AND c.province = '扬州' THEN 'sun_quan'
           WHEN s.id = 'ad200' AND c.province = '交州' THEN 'shi_xie'
           WHEN s.id = 'ad208' AND c.province IN ('司隶','冀州','兖州','豫州','徐州','青州','幽州','并州') THEN 'cao_cao'
           WHEN s.id = 'ad208' AND c.id IN ('jiangxia') THEN 'liu_bei'
           WHEN s.id = 'ad208' AND c.province = '荆州' THEN 'liu_biao'
           WHEN s.id = 'ad208' AND c.province = '益州' THEN 'liu_zhang'
           WHEN s.id = 'ad208' AND c.province = '凉州' THEN 'ma_teng'
           WHEN s.id = 'ad208' AND c.province IN ('扬州','交州') THEN 'sun_quan'
           WHEN s.id = 'ad220' AND c.province IN ('司隶','冀州','兖州','豫州','徐州','青州','幽州','并州','凉州') THEN 'cao_cao'
           WHEN s.id = 'ad220' AND c.province = '益州' THEN 'liu_bei'
           WHEN s.id = 'ad220' AND c.province IN ('扬州','交州') THEN 'sun_quan'
           WHEN s.id = 'ad220' AND c.province = '荆州' AND c.id IN ('xiangyang','nanyang') THEN 'cao_cao'
           WHEN s.id = 'ad220' AND c.province = '荆州' THEN 'sun_quan'
           ELSE 'han_court'
       END,
       CAST((c.population_min + c.population_max) / 2 AS INTEGER),
       260 + c.commerce_base * 2,
       700 + c.agriculture_base * 4,
       1200 + c.strategic_rank * 650,
       35 + c.strategic_rank * 4,
       c.agriculture_base,
       c.commerce_base,
       c.defense_base,
       70,
       CASE c.id
           WHEN 'xuchang' THEN 'xun_yu'
           WHEN 'luoyang' THEN 'xiahou_dun'
           WHEN 'xiapi' THEN 'zhang_fei'
           WHEN 'pingyuan' THEN 'guan_yu'
           WHEN 'ye' THEN 'ju_shou'
           WHEN 'beihai' THEN 'kong_rong'
           WHEN 'jianye' THEN 'zhou_yu'
           WHEN 'jiangxia' THEN 'lu_su'
           WHEN 'chengdu' THEN 'liu_zhang'
           WHEN 'hanzhong' THEN 'zhang_lu'
           WHEN 'wuwei' THEN 'ma_teng'
           ELSE NULL
       END
FROM scenarios s CROSS JOIN cities c;

INSERT INTO scenario_diplomacy (scenario_id, faction_a, faction_b, score, truce_until_turn) VALUES
('ad190','cao_cao','dong_zhuo',-80,NULL),('ad190','yuan_shao','dong_zhuo',-80,NULL),('ad190','liu_bei','dong_zhuo',-70,NULL),('ad190','sun_quan','yuan_shu',20,NULL),
('ad200','liu_bei','cao_cao',-45,NULL),('ad200','liu_bei','sun_quan',10,NULL),('ad200','liu_bei','yuan_shao',-10,NULL),('ad200','cao_cao','sun_quan',-35,NULL),('ad200','cao_cao','yuan_shao',-55,NULL),('ad200','sun_quan','yuan_shao',-20,NULL),
('ad208','liu_bei','sun_quan',45,NULL),('ad208','cao_cao','sun_quan',-75,NULL),('ad208','cao_cao','liu_bei',-80,NULL),('ad208','liu_biao','cao_cao',-40,NULL),
('ad220','cao_cao','liu_bei',-90,NULL),('ad220','cao_cao','sun_quan',-65,NULL),('ad220','liu_bei','sun_quan',-25,NULL);

INSERT INTO officer_life_events (id, officer_id, event_year, event_month, event_kind, faction_id, city_id, notes)
SELECT 'start_' || id, id, COALESCE(birth_year + 18, 190), 1, 'Appear',
       CASE
           WHEN id IN ('cao_cao','cao_pi','cao_ren','cao_hong','xiahou_dun','xiahou_yuan','xun_yu','xun_you','guo_jia','jia_xu','cheng_yu','man_chong','yu_jin','yue_jin','zhang_he','xu_huang','li_dian','xu_chu','dian_wei','sima_yi') THEN 'cao_cao'
           WHEN id IN ('liu_bei','guan_yu','zhang_fei','zhao_yun','zhuge_liang','pang_tong','fa_zheng','jian_yong','mi_zhu','sun_qian','chen_dao','ma_chao','huang_zhong','wei_yan','jiang_wei','liu_shan') THEN 'liu_bei'
           WHEN id IN ('sun_jian','sun_ce','sun_quan','zhou_yu','lu_su','lu_xun','zhang_zhao','zhang_hong','huang_gai','cheng_pu','gan_ning','taishi_ci','zhou_tai') THEN 'sun_quan'
           WHEN id IN ('yuan_shao','yuan_tan','yuan_shang','ju_shou','tian_feng','shen_pei','yan_liang','wen_chou','gao_lan') THEN 'yuan_shao'
           WHEN id IN ('yuan_shu','ji_ling') THEN 'yuan_shu'
           WHEN id IN ('liu_biao','kuai_liang','kuai_yue','huang_zu') THEN 'liu_biao'
           WHEN id IN ('liu_zhang','zhang_ren','yan_yan') THEN 'liu_zhang'
           WHEN id IN ('ma_teng','han_sui','ma_dai') THEN 'ma_teng'
           WHEN id IN ('zhang_lu','yang_song') THEN 'zhang_lu'
           WHEN id IN ('gongsun_zan','zhao_yun_early') THEN 'gongsun_zan'
           WHEN id IN ('lu_bu','gao_shun','chen_gong') THEN 'lu_bu'
           WHEN id IN ('dong_zhuo','li_ru') THEN 'dong_zhuo'
           WHEN id IN ('shi_xie') THEN 'shi_xie'
           ELSE 'han_court'
       END,
       CASE
           WHEN id IN ('cao_cao','cao_pi','cao_ren','cao_hong','xiahou_dun','xiahou_yuan','xun_yu','xun_you','guo_jia','jia_xu','cheng_yu','man_chong','yu_jin','yue_jin','zhang_he','xu_huang','li_dian','xu_chu','dian_wei','sima_yi') THEN 'xuchang'
           WHEN id IN ('liu_bei','guan_yu','zhao_yun','zhuge_liang','pang_tong','fa_zheng','jian_yong','mi_zhu','sun_qian','chen_dao') THEN 'pingyuan'
           WHEN id IN ('zhang_fei') THEN 'xiapi'
           WHEN id IN ('ma_chao','huang_zhong','wei_yan','jiang_wei','liu_shan') THEN 'chengdu'
           WHEN id IN ('sun_jian','sun_ce','sun_quan','zhou_yu','lu_su','lu_xun','zhang_zhao','zhang_hong','huang_gai','cheng_pu','gan_ning','taishi_ci','zhou_tai') THEN 'jianye'
           WHEN id IN ('yuan_shao','yuan_tan','yuan_shang','ju_shou','tian_feng','shen_pei','yan_liang','wen_chou','gao_lan') THEN 'ye'
           WHEN id IN ('yuan_shu','ji_ling') THEN 'shouchun'
           WHEN id IN ('liu_biao','kuai_liang','kuai_yue','huang_zu') THEN 'xiangyang'
           WHEN id IN ('liu_zhang','zhang_ren','yan_yan') THEN 'chengdu'
           WHEN id IN ('ma_teng','han_sui','ma_dai') THEN 'wuwei'
           WHEN id IN ('zhang_lu','yang_song') THEN 'hanzhong'
           WHEN id IN ('gongsun_zan','zhao_yun_early') THEN 'ji'
           WHEN id IN ('lu_bu','gao_shun','chen_gong') THEN 'puyang'
           WHEN id IN ('dong_zhuo','li_ru') THEN 'changan'
           WHEN id IN ('shi_xie') THEN 'jiaozhi'
           ELSE 'luoyang'
       END,
       '根据第一版历史目录生成的初始登场事件'
FROM officers;

INSERT INTO officer_life_events (id, officer_id, event_year, event_month, event_kind, faction_id, city_id, notes)
SELECT 'death_' || id, id, death_year, 12, 'Die', NULL, NULL, '根据人物死亡年生成的离场事件'
FROM officers
WHERE death_year IS NOT NULL;
