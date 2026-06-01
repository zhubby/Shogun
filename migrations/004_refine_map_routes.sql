UPDATE cities
SET x = -145
WHERE id = 'pingyuan';

DELETE FROM roads
WHERE (from_city_id = 'ganling' AND to_city_id = 'pingyuan')
   OR (from_city_id = 'pingyuan' AND to_city_id = 'ganling')
   OR (from_city_id = 'ganling' AND to_city_id = 'zhongshan')
   OR (from_city_id = 'zhongshan' AND to_city_id = 'ganling')
   OR (from_city_id = 'jianye' AND to_city_id = 'yuzhang')
   OR (from_city_id = 'yuzhang' AND to_city_id = 'jianye');

INSERT OR IGNORE INTO roads (from_city_id, to_city_id) VALUES
('ye', 'zhongshan'),
('danyang', 'yuzhang'),
('yuzhang', 'changsha'),
('lingling', 'cangwu'),
('guiyang', 'cangwu');
