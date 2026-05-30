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
