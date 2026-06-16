-- ============================================================
-- 古代水运仪象台漏壶水力精度仿真系统 - ClickHouse 初始化脚本
-- 分层存储：秒级原始(7天) → 分钟聚合(30天) → 小时聚合(1年) → 日聚合(3年)
-- ============================================================

CREATE DATABASE IF NOT EXISTS clepsydra
ENGINE = Atomic;

USE clepsydra;

-- ============================================================
-- 1. 原始数据表（秒级，保留 7 天）
-- ============================================================

CREATE TABLE IF NOT EXISTS sensor_data (
    timestamp DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3),
    clepsydra_id String COMMENT '漏壶编号: KD1-天上壶, KD2-夜漏壶, KD3-平水壶, KD4-万分水',
    water_level Float64 COMMENT '水位高度 (cm)',
    flow_rate Float64 COMMENT '流量 (mL/s)',
    water_temp Float64 COMMENT '水温 (°C)',
    humidity Float64 COMMENT '环境湿度 (%)',
    quality Float64 COMMENT '水质系数 (0.8-1.2)',
    pressure Float64 DEFAULT 101.325 COMMENT '大气压 (kPa)',
    received_at DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3)
)
ENGINE = MergeTree()
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (clepsydra_id, timestamp)
TTL toDateTime(timestamp) + INTERVAL 7 DAY
COMMENT '漏壶传感器秒级原始数据（保留7天）';

CREATE TABLE IF NOT EXISTS hydraulic_metrics (
    timestamp DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3),
    clepsydra_id String,
    theoretical_flow Float64 COMMENT '理论流量 (mL/s)',
    actual_flow Float64 COMMENT '实际流量 (mL/s)',
    flow_error Float64 COMMENT '流量误差率 (%)',
    evaporation_rate Float64 COMMENT '蒸发速率 (mL/s)',
    daily_error_seconds Float64 COMMENT '日累计计时误差 (秒)',
    compensation_flow Float64 COMMENT 'PID补偿流量 (mL/s)',
    pid_kp Float64 COMMENT 'PID比例系数',
    pid_ki Float64 COMMENT 'PID积分系数',
    pid_kd Float64 COMMENT 'PID微分系数'
)
ENGINE = MergeTree()
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (clepsydra_id, timestamp)
TTL toDateTime(timestamp) + INTERVAL 7 DAY
COMMENT '水力精度仿真与PID补偿计算结果（保留7天）';

-- ============================================================
-- 2. 分钟级聚合表（保留 30 天）
-- ============================================================

CREATE TABLE IF NOT EXISTS sensor_data_1min (
    timestamp DateTime('Asia/Shanghai'),
    clepsydra_id String,
    avg_water_level Float64,
    min_water_level Float64,
    max_water_level Float64,
    avg_flow_rate Float64,
    min_flow_rate Float64,
    max_flow_rate Float64,
    avg_water_temp Float64,
    avg_humidity Float64,
    avg_quality Float64,
    avg_pressure Float64,
    sample_count UInt64
)
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (clepsydra_id, timestamp)
TTL timestamp + INTERVAL 30 DAY
COMMENT '传感器数据分钟级聚合（保留30天）';

CREATE MATERIALIZED VIEW IF NOT EXISTS sensor_data_1min_mv
TO sensor_data_1min
AS
SELECT
    toStartOfMinute(timestamp) AS timestamp,
    clepsydra_id,
    avg(water_level) AS avg_water_level,
    min(water_level) AS min_water_level,
    max(water_level) AS max_water_level,
    avg(flow_rate) AS avg_flow_rate,
    min(flow_rate) AS min_flow_rate,
    max(flow_rate) AS max_flow_rate,
    avg(water_temp) AS avg_water_temp,
    avg(humidity) AS avg_humidity,
    avg(quality) AS avg_quality,
    avg(pressure) AS avg_pressure,
    count() AS sample_count
FROM sensor_data
GROUP BY timestamp, clepsydra_id;

CREATE TABLE IF NOT EXISTS hydraulic_metrics_1min (
    timestamp DateTime('Asia/Shanghai'),
    clepsydra_id String,
    avg_theoretical_flow Float64,
    avg_actual_flow Float64,
    avg_flow_error Float64,
    max_flow_error Float64,
    avg_evaporation_rate Float64,
    avg_daily_error Float64,
    max_daily_error Float64,
    avg_compensation_flow Float64,
    sample_count UInt64
)
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (clepsydra_id, timestamp)
TTL timestamp + INTERVAL 30 DAY
COMMENT '水力指标分钟级聚合（保留30天）';

CREATE MATERIALIZED VIEW IF NOT EXISTS hydraulic_metrics_1min_mv
TO hydraulic_metrics_1min
AS
SELECT
    toStartOfMinute(timestamp) AS timestamp,
    clepsydra_id,
    avg(theoretical_flow) AS avg_theoretical_flow,
    avg(actual_flow) AS avg_actual_flow,
    avg(flow_error) AS avg_flow_error,
    max(flow_error) AS max_flow_error,
    avg(evaporation_rate) AS avg_evaporation_rate,
    avg(daily_error_seconds) AS avg_daily_error,
    max(daily_error_seconds) AS max_daily_error,
    avg(compensation_flow) AS avg_compensation_flow,
    count() AS sample_count
FROM hydraulic_metrics
GROUP BY timestamp, clepsydra_id;

-- ============================================================
-- 3. 小时级聚合表（保留 1 年）
-- ============================================================

CREATE TABLE IF NOT EXISTS sensor_data_1hour (
    timestamp DateTime('Asia/Shanghai'),
    clepsydra_id String,
    avg_water_level Float64,
    min_water_level Float64,
    max_water_level Float64,
    avg_flow_rate Float64,
    min_flow_rate Float64,
    max_flow_rate Float64,
    avg_water_temp Float64,
    min_water_temp Float64,
    max_water_temp Float64,
    avg_humidity Float64,
    avg_quality Float64,
    avg_pressure Float64,
    sample_count UInt64
)
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (clepsydra_id, timestamp)
TTL timestamp + INTERVAL 1 YEAR
COMMENT '传感器数据小时级聚合（保留1年）';

CREATE MATERIALIZED VIEW IF NOT EXISTS sensor_data_1hour_mv
TO sensor_data_1hour
AS
SELECT
    toStartOfHour(timestamp) AS timestamp,
    clepsydra_id,
    avg(avg_water_level) AS avg_water_level,
    min(min_water_level) AS min_water_level,
    max(max_water_level) AS max_water_level,
    avg(avg_flow_rate) AS avg_flow_rate,
    min(min_flow_rate) AS min_flow_rate,
    max(max_flow_rate) AS max_flow_rate,
    avg(avg_water_temp) AS avg_water_temp,
    min(avg_water_temp) AS min_water_temp,
    max(avg_water_temp) AS max_water_temp,
    avg(avg_humidity) AS avg_humidity,
    avg(avg_quality) AS avg_quality,
    avg(avg_pressure) AS avg_pressure,
    sum(sample_count) AS sample_count
FROM sensor_data_1min
GROUP BY timestamp, clepsydra_id;

CREATE TABLE IF NOT EXISTS hydraulic_metrics_1hour (
    timestamp DateTime('Asia/Shanghai'),
    clepsydra_id String,
    avg_theoretical_flow Float64,
    avg_actual_flow Float64,
    avg_flow_error Float64,
    max_flow_error Float64,
    avg_evaporation_rate Float64,
    avg_daily_error Float64,
    max_daily_error Float64,
    avg_compensation_flow Float64,
    sample_count UInt64
)
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (clepsydra_id, timestamp)
TTL timestamp + INTERVAL 1 YEAR
COMMENT '水力指标小时级聚合（保留1年）';

CREATE MATERIALIZED VIEW IF NOT EXISTS hydraulic_metrics_1hour_mv
TO hydraulic_metrics_1hour
AS
SELECT
    toStartOfHour(timestamp) AS timestamp,
    clepsydra_id,
    avg(avg_theoretical_flow) AS avg_theoretical_flow,
    avg(avg_actual_flow) AS avg_actual_flow,
    avg(avg_flow_error) AS avg_flow_error,
    max(max_flow_error) AS max_flow_error,
    avg(avg_evaporation_rate) AS avg_evaporation_rate,
    avg(avg_daily_error) AS avg_daily_error,
    max(max_daily_error) AS max_daily_error,
    avg(avg_compensation_flow) AS avg_compensation_flow,
    sum(sample_count) AS sample_count
FROM hydraulic_metrics_1min
GROUP BY timestamp, clepsydra_id;

-- ============================================================
-- 4. 日级聚合表（保留 3 年）
-- ============================================================

CREATE TABLE IF NOT EXISTS daily_error_summary (
    date Date,
    clepsydra_id String,
    max_daily_error Float64,
    avg_daily_error Float64,
    min_daily_error Float64,
    avg_theoretical_flow Float64,
    avg_actual_flow Float64,
    avg_evaporation_rate Float64,
    avg_compensation_flow Float64,
    max_water_level Float64,
    min_water_level Float64,
    avg_water_temp Float64,
    avg_pressure Float64,
    data_points UInt64
)
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, clepsydra_id)
TTL date + INTERVAL 3 YEAR
COMMENT '日级误差与精度统计汇总（保留3年）';

CREATE MATERIALIZED VIEW IF NOT EXISTS daily_error_summary_mv
TO daily_error_summary
AS
SELECT
    toDate(timestamp) AS date,
    clepsydra_id,
    max(max_daily_error) AS max_daily_error,
    avg(avg_daily_error) AS avg_daily_error,
    min(avg_daily_error) AS min_daily_error,
    avg(avg_theoretical_flow) AS avg_theoretical_flow,
    avg(avg_actual_flow) AS avg_actual_flow,
    avg(avg_evaporation_rate) AS avg_evaporation_rate,
    avg(avg_compensation_flow) AS avg_compensation_flow,
    max(max_water_level) AS max_water_level,
    min(min_water_level) AS min_water_level,
    avg(avg_water_temp) AS avg_water_temp,
    avg(avg_pressure) AS avg_pressure,
    sum(sample_count) AS data_points
FROM hydraulic_metrics_1hour
    INNER JOIN sensor_data_1hour USING (timestamp, clepsydra_id)
GROUP BY date, clepsydra_id;

-- ============================================================
-- 5. 告警事件表（保留 1 年）
-- ============================================================

CREATE TABLE IF NOT EXISTS alerts (
    id UUID DEFAULT generateUUIDv4(),
    timestamp DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3),
    clepsydra_id String,
    alert_type String COMMENT '告警类型: WATER_LEVEL_HIGH, WATER_LEVEL_LOW, DAILY_ERROR_EXCEED, TEMP_ABNORMAL',
    alert_level String COMMENT '告警级别: INFO, WARNING, CRITICAL',
    message String,
    value Float64 COMMENT '触发告警的数值',
    threshold Float64 COMMENT '告警阈值',
    resolved UInt8 DEFAULT 0 COMMENT '是否已解决',
    resolved_at DateTime64(3, 'Asia/Shanghai')
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (alert_type, timestamp)
TTL toDateTime(timestamp) + INTERVAL 1 YEAR
COMMENT '告警事件记录表（保留1年）';

-- ============================================================
-- 6. 漏壶配置参数表
-- ============================================================

CREATE TABLE IF NOT EXISTS clepsydra_config (
    clepsydra_id String,
    name String COMMENT '漏壶名称',
    max_level Float64 COMMENT '最高水位 (cm)',
    min_level Float64 COMMENT '最低水位 (cm)',
    standard_flow Float64 COMMENT '标准流量 (mL/s)',
    cross_section_area Float64 COMMENT '横截面积 (cm²)',
    orifice_diameter Float64 COMMENT '出水孔直径 (cm)',
    flow_coefficient Float64 COMMENT '流量系数',
    updated_at DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3)
)
ENGINE = ReplacingMergeTree(updated_at)
ORDER BY clepsydra_id
COMMENT '漏壶配置参数';

-- 插入初始漏壶配置（宋代水运仪象台四级漏壶）
INSERT INTO clepsydra_config (clepsydra_id, name, max_level, min_level, standard_flow, cross_section_area, orifice_diameter, flow_coefficient) VALUES
('KD1', '天上壶', 120.0, 20.0, 2.5, 78.54, 0.3, 0.62),
('KD2', '夜漏壶', 100.0, 15.0, 2.5, 78.54, 0.3, 0.62),
('KD3', '平水壶', 80.0, 10.0, 2.5, 78.54, 0.3, 0.62),
('KD4', '万分水', 60.0, 5.0, 2.5, 78.54, 0.3, 0.62);

-- ============================================================
-- 7. 水位异常检测结果表（保留 30 天）
-- ============================================================

CREATE TABLE IF NOT EXISTS water_level_alerts (
    timestamp DateTime64(3, 'Asia/Shanghai'),
    clepsydra_id String,
    water_level Float64,
    max_level Float64,
    min_level Float64,
    is_high UInt8,
    is_low UInt8
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (clepsydra_id, timestamp)
TTL toDateTime(timestamp) + INTERVAL 30 DAY
COMMENT '水位异常检测结果（保留30天）';

-- ============================================================
-- 8. 系统元数据表
-- ============================================================

CREATE TABLE IF NOT EXISTS system_info (
    key String,
    value String,
    updated_at DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3)
)
ENGINE = ReplacingMergeTree(updated_at)
ORDER BY key
COMMENT '系统元信息';

INSERT INTO system_info (key, value) VALUES
('schema_version', '3.0'),
('initialized_at', toString(now64(3))),
('description', '古代水运仪象台漏壶水力精度仿真系统数据库');

-- ============================================================
-- 9. 朝代漏壶配置表
-- ============================================================

CREATE TABLE IF NOT EXISTS dynasty_clepsydra_config (
    dynasty_id String,
    dynasty_name String,
    era String,
    clepsydra_type String,
    stage_count UInt8,
    description String,
    historical_daily_error_seconds Float64,
    typical_water_temp_c Float64,
    material String,
    reference_year Int32,
    stage_configs String COMMENT 'JSON格式的各漏壶配置数组',
    created_at DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3)
)
ENGINE = ReplacingMergeTree(created_at)
ORDER BY dynasty_id
COMMENT '各朝代漏壶系统配置参数';

INSERT INTO dynasty_clepsydra_config (dynasty_id, dynasty_name, era, clepsydra_type, stage_count, description, historical_daily_error_seconds, typical_water_temp_c, material, reference_year, stage_configs) VALUES
('HAN_CHENJIAN', '汉代', '西汉', '沉箭漏（单级浮箭）', 1, '汉代沉箭漏为早期单级漏壶，箭尺随水位下沉指示时间，结构简单但精度较低。', 900.0, 15.0, '青铜', -100, '[{"clepsydra_id":"HAN01","name":"沉箭壶","max_level":80.0,"min_level":5.0,"standard_flow":1.8,"cross_section_area":113.1,"orifice_diameter":0.25,"flow_coefficient":0.58}]'),
('HAN_FUJIAN', '汉代', '东汉', '浮箭漏（二级补偿）', 2, '东汉张衡改进的二级浮箭漏，增加补偿壶以稳定水位，精度较单级大幅提升。', 300.0, 15.0, '青铜', 125, '[{"clepsydra_id":"HF01","name":"上壶","max_level":90.0,"min_level":10.0,"standard_flow":2.0,"cross_section_area":95.0,"orifice_diameter":0.28,"flow_coefficient":0.60},{"clepsydra_id":"HF02","name":"下壶","max_level":70.0,"min_level":5.0,"standard_flow":2.0,"cross_section_area":78.5,"orifice_diameter":0.28,"flow_coefficient":0.60}]'),
('TANG_JINGLU', '唐代', '盛唐', '四级浮箭漏（吕才）', 4, '唐代吕才设计的四级漏壶，从单级发展到多级补偿，是宋代水运仪象台的前驱。', 120.0, 18.0, '铜鎏金', 650, '[{"clepsydra_id":"TJ01","name":"夜天池","max_level":110.0,"min_level":15.0,"standard_flow":2.3,"cross_section_area":85.0,"orifice_diameter":0.29,"flow_coefficient":0.61},{"clepsydra_id":"TJ02","name":"日天池","max_level":95.0,"min_level":12.0,"standard_flow":2.3,"cross_section_area":85.0,"orifice_diameter":0.29,"flow_coefficient":0.61},{"clepsydra_id":"TJ03","name":"平壶","max_level":75.0,"min_level":10.0,"standard_flow":2.3,"cross_section_area":85.0,"orifice_diameter":0.29,"flow_coefficient":0.61},{"clepsydra_id":"TJ04","name":"万分水","max_level":55.0,"min_level":5.0,"standard_flow":2.3,"cross_section_area":70.0,"orifice_diameter":0.29,"flow_coefficient":0.61}]'),
('SONG_LIANHUA', '宋代', '北宋', '莲花漏（燕肃）', 3, '北宋燕肃发明的莲花漏，采用漫流系统恒定水位，刻花莲花装饰，精度极高，是宋代漏壶之冠。', 45.0, 20.0, '精铜', 1030, '[{"clepsydra_id":"SL01","name":"上匮","max_level":100.0,"min_level":20.0,"standard_flow":2.45,"cross_section_area":80.0,"orifice_diameter":0.3,"flow_coefficient":0.62},{"clepsydra_id":"SL02","name":"次匮","max_level":85.0,"min_level":15.0,"standard_flow":2.45,"cross_section_area":80.0,"orifice_diameter":0.3,"flow_coefficient":0.62},{"clepsydra_id":"SL03","name":"下匮","max_level":65.0,"min_level":10.0,"standard_flow":2.45,"cross_section_area":70.0,"orifice_diameter":0.3,"flow_coefficient":0.62}]'),
('SONG_YITIAN', '宋代', '北宋', '水运仪象台（苏颂）', 4, '苏颂、韩公廉于元祐三年建造的水运仪象台四级漏壶，天上壶、夜漏壶、平水壶、万分水串联，驱动浑仪浑象，精度日误差<1分钟。', 50.0, 20.0, '精铜', 1088, '[{"clepsydra_id":"KD1","name":"天上壶","max_level":120.0,"min_level":20.0,"standard_flow":2.5,"cross_section_area":78.54,"orifice_diameter":0.3,"flow_coefficient":0.62},{"clepsydra_id":"KD2","name":"夜漏壶","max_level":100.0,"min_level":15.0,"standard_flow":2.5,"cross_section_area":78.54,"orifice_diameter":0.3,"flow_coefficient":0.62},{"clepsydra_id":"KD3","name":"平水壶","max_level":80.0,"min_level":10.0,"standard_flow":2.5,"cross_section_area":78.54,"orifice_diameter":0.3,"flow_coefficient":0.62},{"clepsydra_id":"KD4","name":"万分水","max_level":60.0,"min_level":5.0,"standard_flow":2.5,"cross_section_area":78.54,"orifice_diameter":0.3,"flow_coefficient":0.62}]'),
('YONG_LE', '明代', '明初', '永乐漏刻', 4, '明代永乐年间造漏刻，继承宋代技术，在皇宫和钦天监使用，结构稳定。', 65.0, 18.0, '黄铜', 1420, '[{"clepsydra_id":"YL01","name":"子壶","max_level":115.0,"min_level":18.0,"standard_flow":2.48,"cross_section_area":82.0,"orifice_diameter":0.3,"flow_coefficient":0.62},{"clepsydra_id":"YL02","name":"丑壶","max_level":95.0,"min_level":14.0,"standard_flow":2.48,"cross_section_area":82.0,"orifice_diameter":0.3,"flow_coefficient":0.62},{"clepsydra_id":"YL03","name":"寅壶","max_level":75.0,"min_level":9.0,"standard_flow":2.48,"cross_section_area":78.0,"orifice_diameter":0.3,"flow_coefficient":0.62},{"clepsydra_id":"YL04","name":"卯壶","max_level":55.0,"min_level":4.5,"standard_flow":2.48,"cross_section_area":72.0,"orifice_diameter":0.3,"flow_coefficient":0.62}]');

-- ============================================================
-- 10. 现代计时器配置表
-- ============================================================

CREATE TABLE IF NOT EXISTS modern_timepiece_config (
    piece_id String,
    name String,
    category String,
    daily_error_seconds Float64,
    yearly_error_seconds Float64,
    technology String,
    invention_year UInt32,
    description String,
    accuracy_class String,
    created_at DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3)
)
ENGINE = ReplacingMergeTree(created_at)
ORDER BY piece_id
COMMENT '现代计时器精度参数（用于跨时代对比）';

INSERT INTO modern_timepiece_config (piece_id, name, category, daily_error_seconds, yearly_error_seconds, technology, invention_year, description, accuracy_class) VALUES
('MECH_WATCH', '机械手表', '机械', 10.0, 3650.0, '摆轮游丝', 1675, '传统机械手表，日误差±10秒属天文台级别。', '中等'),
('QUARTZ_WATCH', '石英手表', '电子', 0.5, 182.5, '石英晶体振荡器', 1969, '普通石英手表，日误差0.5秒，年误差约3分钟。', '良好'),
('HI_ACC_QUARTZ', '高精度石英表', '电子', 0.05, 18.25, '恒温石英晶体', 1960, '高精度石英表（如Grand Seiko 9F），年误差10-20秒。', '优秀'),
('ATOMIC_CS', '铯原子钟', '原子', 1.0e-6, 3.65e-4, '铯原子超精细跃迁', 1955, 'NIST-F1铯原子钟，3000万年误差1秒，定义秒的基准。', '顶级'),
('ATOMIC_RB', '铷原子钟', '原子', 5.0e-5, 0.01825, '铷原子跃迁', 1958, '商业铷原子钟，体积小，常用于通信基站。', '极高'),
('GPS_CLOCK', 'GPS授时', '卫星', 1.0e-5, 0.00365, '原子钟群+相对论修正', 1978, 'GPS卫星系统授时，误差纳秒级，含广义相对论修正。', '顶级'),
('PENDULUM', '精密摆钟', '机械', 0.2, 73.0, '重力摆', 1656, '惠更斯发明的精密摆钟，天文台级摆钟可达日误差0.2秒。', '良好'),
('MECH_CHRONO', '机械天文台表', '机械', 2.0, 730.0, '陀飞轮/补偿摆轮', 1920, '通过COSC认证的天文台机械表，日误差-4~+6秒。', '良好');

-- ============================================================
-- 11. 精度对比结果表
-- ============================================================

CREATE TABLE IF NOT EXISTS accuracy_comparison_results (
    id UUID DEFAULT generateUUIDv4(),
    timestamp DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3),
    comparison_type String COMMENT 'dynasty / cross_era / custom',
    left_entity_id String,
    right_entity_id String,
    left_daily_error_seconds Float64,
    right_daily_error_seconds Float64,
    error_ratio Float64,
    user_session_id String DEFAULT '',
    parameters String COMMENT 'JSON格式的环境参数'
)
ENGINE = MergeTree()
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (comparison_type, timestamp)
TTL toDateTime(timestamp) + INTERVAL 90 DAY
COMMENT '精度对比结果历史记录（保留90天）';

-- ============================================================
-- 12. 误差传递分析结果表
-- ============================================================

CREATE TABLE IF NOT EXISTS error_transfer_analysis (
    id UUID DEFAULT generateUUIDv4(),
    timestamp DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3),
    dynasty_id String,
    total_error_seconds Float64,
    bottleneck_stage UInt8,
    bottleneck_reason String,
    compensation_potential_seconds Float64,
    node_data String COMMENT 'JSON格式的各节点误差数据',
    recommendations String COMMENT 'JSON格式的建议数组'
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (dynasty_id, timestamp)
TTL toDateTime(timestamp) + INTERVAL 1 YEAR
COMMENT '多级漏壶误差传递分析结果（保留1年）';

-- ============================================================
-- 13. 用户虚拟操作记录表
-- ============================================================

CREATE TABLE IF NOT EXISTS virtual_operation_logs (
    id UUID DEFAULT generateUUIDv4(),
    timestamp DateTime64(3, 'Asia/Shanghai') DEFAULT now64(3),
    user_session_id String DEFAULT '',
    clepsydra_id String,
    initial_level_cm Float64,
    target_level_cm Float64,
    final_level_cm Float64,
    initial_error_seconds Float64,
    final_error_seconds Float64,
    simulated_seconds UInt32,
    water_temp_c Float64,
    observations String COMMENT 'JSON格式的观察结论数组'
)
ENGINE = MergeTree()
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (clepsydra_id, timestamp)
TTL toDateTime(timestamp) + INTERVAL 30 DAY
COMMENT '用户虚拟操作漏壶体验记录（保留30天）';
