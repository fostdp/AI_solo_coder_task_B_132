use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, Context};
use parking_lot::Mutex;

use crate::clickhouse_store::ClickHouseStore;
use crate::config_loader::AppConfig;
use crate::hydraulic::HydraulicModel;
use crate::models::{
    ClepsydraConfig, CrossEraComparison, DynastyClepsydraConfig, DynastyComparison,
    ErrorTransferAnalysis, ErrorTransferNode, FlowComparisonPoint, ModernTimepiece,
    TimelineAccuracy, VirtualOperationRequest, VirtualOperationResult,
};

fn default_dynasties() -> Vec<DynastyClepsydraConfig> {
    vec![
        DynastyClepsydraConfig {
            dynasty_id: "HAN_CHENJIAN".to_string(),
            dynasty_name: "汉代".to_string(),
            era: "西汉".to_string(),
            clepsydra_type: "沉箭漏（单级浮箭）".to_string(),
            stage_count: 1,
            description: "汉代沉箭漏为早期单级漏壶，箭尺随水位下沉指示时间，结构简单但精度较低。".to_string(),
            historical_daily_error_seconds: 900.0,
            typical_water_temp_c: 15.0,
            material: "青铜".to_string(),
            configs: vec![
                ClepsydraConfig {
                    clepsydra_id: "HAN01".to_string(),
                    name: "沉箭壶".to_string(),
                    max_level: 80.0, min_level: 5.0, standard_flow: 1.8,
                    cross_section_area: 113.1, orifice_diameter: 0.25, flow_coefficient: 0.58,
                }
            ],
            reference_year: -100,
        },
        DynastyClepsydraConfig {
            dynasty_id: "HAN_FUJIAN".to_string(),
            dynasty_name: "汉代".to_string(),
            era: "东汉".to_string(),
            clepsydra_type: "浮箭漏（二级补偿）".to_string(),
            stage_count: 2,
            description: "东汉张衡改进的二级浮箭漏，增加补偿壶以稳定水位，精度较单级大幅提升。".to_string(),
            historical_daily_error_seconds: 300.0,
            typical_water_temp_c: 15.0,
            material: "青铜".to_string(),
            configs: vec![
                ClepsydraConfig {
                    clepsydra_id: "HF01".to_string(), name: "上壶".to_string(),
                    max_level: 90.0, min_level: 10.0, standard_flow: 2.0,
                    cross_section_area: 95.0, orifice_diameter: 0.28, flow_coefficient: 0.60,
                },
                ClepsydraConfig {
                    clepsydra_id: "HF02".to_string(), name: "下壶".to_string(),
                    max_level: 70.0, min_level: 5.0, standard_flow: 2.0,
                    cross_section_area: 78.5, orifice_diameter: 0.28, flow_coefficient: 0.60,
                },
            ],
            reference_year: 125,
        },
        DynastyClepsydraConfig {
            dynasty_id: "TANG_JINGLU".to_string(),
            dynasty_name: "唐代".to_string(),
            era: "盛唐".to_string(),
            clepsydra_type: "四级浮箭漏（吕才）".to_string(),
            stage_count: 4,
            description: "唐代吕才设计的四级漏壶，从单级发展到多级补偿，是宋代水运仪象台的前驱。".to_string(),
            historical_daily_error_seconds: 120.0,
            typical_water_temp_c: 18.0,
            material: "铜鎏金".to_string(),
            configs: vec![
                ClepsydraConfig {
                    clepsydra_id: "TJ01".to_string(), name: "夜天池".to_string(),
                    max_level: 110.0, min_level: 15.0, standard_flow: 2.3,
                    cross_section_area: 85.0, orifice_diameter: 0.29, flow_coefficient: 0.61,
                },
                ClepsydraConfig {
                    clepsydra_id: "TJ02".to_string(), name: "日天池".to_string(),
                    max_level: 95.0, min_level: 12.0, standard_flow: 2.3,
                    cross_section_area: 85.0, orifice_diameter: 0.29, flow_coefficient: 0.61,
                },
                ClepsydraConfig {
                    clepsydra_id: "TJ03".to_string(), name: "平壶".to_string(),
                    max_level: 75.0, min_level: 10.0, standard_flow: 2.3,
                    cross_section_area: 85.0, orifice_diameter: 0.29, flow_coefficient: 0.61,
                },
                ClepsydraConfig {
                    clepsydra_id: "TJ04".to_string(), name: "万分水".to_string(),
                    max_level: 55.0, min_level: 5.0, standard_flow: 2.3,
                    cross_section_area: 70.0, orifice_diameter: 0.29, flow_coefficient: 0.61,
                },
            ],
            reference_year: 650,
        },
        DynastyClepsydraConfig {
            dynasty_id: "SONG_LIANHUA".to_string(),
            dynasty_name: "宋代".to_string(),
            era: "北宋".to_string(),
            clepsydra_type: "莲花漏（燕肃）".to_string(),
            stage_count: 3,
            description: "北宋燕肃发明的莲花漏，采用漫流系统恒定水位，刻花莲花装饰，精度极高，是宋代漏壶之冠。".to_string(),
            historical_daily_error_seconds: 45.0,
            typical_water_temp_c: 20.0,
            material: "精铜".to_string(),
            configs: vec![
                ClepsydraConfig {
                    clepsydra_id: "SL01".to_string(), name: "上匮".to_string(),
                    max_level: 100.0, min_level: 20.0, standard_flow: 2.45,
                    cross_section_area: 80.0, orifice_diameter: 0.3, flow_coefficient: 0.62,
                },
                ClepsydraConfig {
                    clepsydra_id: "SL02".to_string(), name: "次匮".to_string(),
                    max_level: 85.0, min_level: 15.0, standard_flow: 2.45,
                    cross_section_area: 80.0, orifice_diameter: 0.3, flow_coefficient: 0.62,
                },
                ClepsydraConfig {
                    clepsydra_id: "SL03".to_string(), name: "下匮".to_string(),
                    max_level: 65.0, min_level: 10.0, standard_flow: 2.45,
                    cross_section_area: 70.0, orifice_diameter: 0.3, flow_coefficient: 0.62,
                },
            ],
            reference_year: 1030,
        },
        DynastyClepsydraConfig {
            dynasty_id: "SONG_YITIAN".to_string(),
            dynasty_name: "宋代".to_string(),
            era: "北宋".to_string(),
            clepsydra_type: "水运仪象台（苏颂）".to_string(),
            stage_count: 4,
            description: "苏颂、韩公廉于元祐三年建造的水运仪象台四级漏壶，天上壶、夜漏壶、平水壶、万分水串联，驱动浑仪浑象，精度日误差<1分钟。".to_string(),
            historical_daily_error_seconds: 50.0,
            typical_water_temp_c: 20.0,
            material: "精铜".to_string(),
            configs: vec![
                ClepsydraConfig {
                    clepsydra_id: "KD1".to_string(), name: "天上壶".to_string(),
                    max_level: 120.0, min_level: 20.0, standard_flow: 2.5,
                    cross_section_area: 78.54, orifice_diameter: 0.3, flow_coefficient: 0.62,
                },
                ClepsydraConfig {
                    clepsydra_id: "KD2".to_string(), name: "夜漏壶".to_string(),
                    max_level: 100.0, min_level: 15.0, standard_flow: 2.5,
                    cross_section_area: 78.54, orifice_diameter: 0.3, flow_coefficient: 0.62,
                },
                ClepsydraConfig {
                    clepsydra_id: "KD3".to_string(), name: "平水壶".to_string(),
                    max_level: 80.0, min_level: 10.0, standard_flow: 2.5,
                    cross_section_area: 78.54, orifice_diameter: 0.3, flow_coefficient: 0.62,
                },
                ClepsydraConfig {
                    clepsydra_id: "KD4".to_string(), name: "万分水".to_string(),
                    max_level: 60.0, min_level: 5.0, standard_flow: 2.5,
                    cross_section_area: 78.54, orifice_diameter: 0.3, flow_coefficient: 0.62,
                },
            ],
            reference_year: 1088,
        },
        DynastyClepsydraConfig {
            dynasty_id: "YONG_LE".to_string(),
            dynasty_name: "明代".to_string(),
            era: "明初".to_string(),
            clepsydra_type: "永乐漏刻".to_string(),
            stage_count: 4,
            description: "明代永乐年间造漏刻，继承宋代技术，在皇宫和钦天监使用，结构稳定。".to_string(),
            historical_daily_error_seconds: 65.0,
            typical_water_temp_c: 18.0,
            material: "黄铜".to_string(),
            configs: vec![
                ClepsydraConfig {
                    clepsydra_id: "YL01".to_string(), name: "子壶".to_string(),
                    max_level: 115.0, min_level: 18.0, standard_flow: 2.48,
                    cross_section_area: 82.0, orifice_diameter: 0.3, flow_coefficient: 0.62,
                },
                ClepsydraConfig {
                    clepsydra_id: "YL02".to_string(), name: "丑壶".to_string(),
                    max_level: 95.0, min_level: 14.0, standard_flow: 2.48,
                    cross_section_area: 82.0, orifice_diameter: 0.3, flow_coefficient: 0.62,
                },
                ClepsydraConfig {
                    clepsydra_id: "YL03".to_string(), name: "寅壶".to_string(),
                    max_level: 75.0, min_level: 9.0, standard_flow: 2.48,
                    cross_section_area: 78.0, orifice_diameter: 0.3, flow_coefficient: 0.62,
                },
                ClepsydraConfig {
                    clepsydra_id: "YL04".to_string(), name: "卯壶".to_string(),
                    max_level: 55.0, min_level: 4.5, standard_flow: 2.48,
                    cross_section_area: 72.0, orifice_diameter: 0.3, flow_coefficient: 0.62,
                },
            ],
            reference_year: 1420,
        },
    ]
}

fn default_modern_timepieces() -> Vec<ModernTimepiece> {
    vec![
        ModernTimepiece {
            piece_id: "MECH_WATCH".to_string(), name: "机械手表".to_string(),
            category: "机械".to_string(), daily_error_seconds: 10.0, yearly_error_seconds: 3650.0,
            technology: "摆轮游丝".to_string(), invention_year: 1675,
            description: "传统机械手表，日误差±10秒属天文台级别。".to_string(),
            accuracy_class: "中等".to_string(),
        },
        ModernTimepiece {
            piece_id: "QUARTZ_WATCH".to_string(), name: "石英手表".to_string(),
            category: "电子".to_string(), daily_error_seconds: 0.5, yearly_error_seconds: 182.5,
            technology: "石英晶体振荡器".to_string(), invention_year: 1969,
            description: "普通石英手表，日误差0.5秒，年误差约3分钟。".to_string(),
            accuracy_class: "良好".to_string(),
        },
        ModernTimepiece {
            piece_id: "HI_ACC_QUARTZ".to_string(), name: "高精度石英表".to_string(),
            category: "电子".to_string(), daily_error_seconds: 0.05, yearly_error_seconds: 18.25,
            technology: "恒温石英晶体".to_string(), invention_year: 1960,
            description: "高精度石英表（如Grand Seiko 9F），年误差10-20秒。".to_string(),
            accuracy_class: "优秀".to_string(),
        },
        ModernTimepiece {
            piece_id: "ATOMIC_CS".to_string(), name: "铯原子钟".to_string(),
            category: "原子".to_string(), daily_error_seconds: 1e-6, yearly_error_seconds: 3.65e-4,
            technology: "铯原子超精细跃迁".to_string(), invention_year: 1955,
            description: "NIST-F1铯原子钟，3000万年误差1秒，定义秒的基准。".to_string(),
            accuracy_class: "顶级".to_string(),
        },
        ModernTimepiece {
            piece_id: "ATOMIC_RB".to_string(), name: "铷原子钟".to_string(),
            category: "原子".to_string(), daily_error_seconds: 5e-5, yearly_error_seconds: 0.01825,
            technology: "铷原子跃迁".to_string(), invention_year: 1958,
            description: "商业铷原子钟，体积小，常用于通信基站。".to_string(),
            accuracy_class: "极高".to_string(),
        },
        ModernTimepiece {
            piece_id: "GPS_CLOCK".to_string(), name: "GPS授时".to_string(),
            category: "卫星".to_string(), daily_error_seconds: 1e-5, yearly_error_seconds: 0.00365,
            technology: "原子钟群+相对论修正".to_string(), invention_year: 1978,
            description: "GPS卫星系统授时，误差纳秒级，含广义相对论修正。".to_string(),
            accuracy_class: "顶级".to_string(),
        },
        ModernTimepiece {
            piece_id: "PENDULUM".to_string(), name: "精密摆钟".to_string(),
            category: "机械".to_string(), daily_error_seconds: 0.2, yearly_error_seconds: 73.0,
            technology: "重力摆".to_string(), invention_year: 1656,
            description: "惠更斯发明的精密摆钟，天文台级摆钟可达日误差0.2秒。".to_string(),
            accuracy_class: "良好".to_string(),
        },
        ModernTimepiece {
            piece_id: "MECH_CHRONO".to_string(), name: "机械天文台表".to_string(),
            category: "机械".to_string(), daily_error_seconds: 2.0, yearly_error_seconds: 730.0,
            technology: "陀飞轮/补偿摆轮".to_string(), invention_year: 1920,
            description: "通过COSC认证的天文台机械表，日误差-4~+6秒。".to_string(),
            accuracy_class: "良好".to_string(),
        },
    ]
}

fn dynasty_color(dynasty: &str) -> &'static str {
    match dynasty {
        "HAN_CHENJIAN" => "#8B4513",
        "HAN_FUJIAN" => "#A0522D",
        "TANG_JINGLU" => "#DAA520",
        "SONG_LIANHUA" => "#FF6B6B",
        "SONG_YITIAN" => "#4ECDC4",
        "YONG_LE" => "#9B59B6",
        _ => "#666666",
    }
}

fn modern_color(category: &str) -> &'static str {
    match category {
        "机械" => "#7F8C8D",
        "电子" => "#3498DB",
        "原子" => "#E74C3C",
        "卫星" => "#2ECC71",
        _ => "#95A5A6",
    }
}

pub struct AnalysisService {
    pub config: Arc<AppConfig>,
    pub store: Arc<ClickHouseStore>,
    pub hydraulic: Arc<HydraulicModel>,
    pub dynasties: Mutex<HashMap<String, DynastyClepsydraConfig>>,
    pub modern_pieces: Mutex<HashMap<String, ModernTimepiece>>,
}

impl AnalysisService {
    pub fn new(
        config: Arc<AppConfig>,
        store: Arc<ClickHouseStore>,
        hydraulic: Arc<HydraulicModel>,
    ) -> Self {
        let mut dyn_map = HashMap::new();
        for d in default_dynasties() {
            dyn_map.insert(d.dynasty_id.clone(), d);
        }
        let mut mod_map = HashMap::new();
        for m in default_modern_timepieces() {
            mod_map.insert(m.piece_id.clone(), m);
        }
        Self {
            config, store, hydraulic,
            dynasties: Mutex::new(dyn_map),
            modern_pieces: Mutex::new(mod_map),
        }
    }

    pub fn get_all_dynasties(&self) -> Vec<DynastyClepsydraConfig> {
        let map = self.dynasties.lock();
        map.values().cloned().collect()
    }

    pub fn get_dynasty(&self, id: &str) -> Option<DynastyClepsydraConfig> {
        let map = self.dynasties.lock();
        map.get(id).cloned()
    }

    pub fn get_all_modern(&self) -> Vec<ModernTimepiece> {
        let map = self.modern_pieces.lock();
        map.values().cloned().collect()
    }

    pub fn get_modern(&self, id: &str) -> Option<ModernTimepiece> {
        let map = self.modern_pieces.lock();
        map.get(id).cloned()
    }

    fn calc_dynasty_daily_error(&self, dynasty: &DynastyClepsydraConfig) -> f64 {
        let temp = dynasty.typical_water_temp_c;
        let humidity = 60.0;
        let quality = 1.0;
        let pressure = self.hydraulic.altitude_to_pressure(self.config.hydraulic.altitude_m);

        let mut cumulative_error = 0.0;
        let _dt = 1.0;
        let seconds_per_day = 86400.0;

        for cfg in &dynasty.configs {
            let level_range = cfg.max_level - cfg.min_level;
            let avg_level = cfg.min_level + level_range * 0.7;

            let theoretical = self.hydraulic.calculate_theoretical_flow(
                avg_level, cfg, temp,
            );

            let level_drop_ratio = if dynasty.stage_count <= 1 {
                0.25
            } else if dynasty.stage_count == 2 {
                0.10
            } else if dynasty.stage_count == 3 {
                0.035
            } else {
                0.018
            };

            let actual_flow_deviation = theoretical * level_drop_ratio;
            let evap = self.hydraulic.calculate_evaporation_rate(
                temp, humidity, cfg.cross_section_area, quality, pressure,
            );

            let effective_error_rate = (actual_flow_deviation + evap) / theoretical * 100.0;
            let stage_error = effective_error_rate / 100.0 * seconds_per_day;
            cumulative_error += stage_error / (dynasty.stage_count as f64).sqrt();
        }

        let base_historical = dynasty.historical_daily_error_seconds;
        (base_historical + cumulative_error) / 2.0
    }

    pub fn compare_dynasties(&self, left_id: &str, right_id: &str) -> Result<DynastyComparison> {
        let left = self.get_dynasty(left_id)
            .with_context(|| format!("未找到朝代漏壶: {}", left_id))?;
        let right = self.get_dynasty(right_id)
            .with_context(|| format!("未找到朝代漏壶: {}", right_id))?;

        let left_error = self.calc_dynasty_daily_error(&left);
        let right_error = self.calc_dynasty_daily_error(&right);

        let error_ratio = if right_error.abs() > 1e-9 {
            left_error / right_error
        } else {
            1.0
        };

        let winner = if left_error < right_error {
            left.dynasty_name.clone()
        } else {
            right.dynasty_name.clone()
        };

        let mut key_diff = Vec::new();
        if left.stage_count != right.stage_count {
            key_diff.push(format!(
                "级数差异：{}{}级 vs {}{}级",
                left.dynasty_name, left.stage_count,
                right.dynasty_name, right.stage_count
            ));
        }
        key_diff.push(format!(
            "材质差异：{} vs {}",
            left.material, right.material
        ));
        key_diff.push(format!(
            "典型水温：{:.0}°C vs {:.0}°C",
            left.typical_water_temp_c, right.typical_water_temp_c
        ));
        if left_error < right_error {
            key_diff.push(format!(
                "精度提升：{}比{}精确{:.1}倍",
                right.dynasty_name, left.dynasty_name, error_ratio
            ));
        } else {
            key_diff.push(format!(
                "精度提升：{}比{}精确{:.1}倍",
                left.dynasty_name, right.dynasty_name, 1.0 / error_ratio
            ));
        }

        let flow_comparison: Vec<FlowComparisonPoint> = (0..left.configs.len().max(right.configs.len()))
            .map(|i| {
                let left_cfg = left.configs.get(i);
                let right_cfg = right.configs.get(i);
                FlowComparisonPoint {
                    stage: format!("第{}级", i + 1),
                    left_flow_mlps: left_cfg.map(|c| c.standard_flow).unwrap_or(0.0),
                    right_flow_mlps: right_cfg.map(|c| c.standard_flow).unwrap_or(0.0),
                    left_level_cm: left_cfg.map(|c| (c.max_level + c.min_level) / 2.0).unwrap_or(0.0),
                    right_level_cm: right_cfg.map(|c| (c.max_level + c.min_level) / 2.0).unwrap_or(0.0),
                }
            })
            .collect();

        Ok(DynastyComparison {
            left_dynasty: left,
            right_dynasty: right,
            left_daily_error_seconds: left_error,
            right_daily_error_seconds: right_error,
            error_ratio,
            winner,
            key_differences: key_diff,
            flow_comparison,
        })
    }

    pub fn analyze_error_transfer(&self, dynasty_id: &str) -> Result<ErrorTransferAnalysis> {
        let dynasty = self.get_dynasty(dynasty_id)
            .with_context(|| format!("未找到朝代漏壶: {}", dynasty_id))?;

        let temp = dynasty.typical_water_temp_c;
        let pressure = self.hydraulic.altitude_to_pressure(self.config.hydraulic.altitude_m);
        let humidity = 60.0;
        let quality = 1.0;

        let mut nodes = Vec::new();
        let mut cumulative_input_error = 0.0;
        let mut total_self_error = 0.0;

        for (i, cfg) in dynasty.configs.iter().enumerate() {
            let idx = i as u32;
            let level_range = cfg.max_level - cfg.min_level;
            let avg_level = cfg.min_level + level_range * 0.7;

            let theoretical = self.hydraulic.calculate_theoretical_flow(avg_level, cfg, temp);
            let evap = self.hydraulic.calculate_evaporation_rate(
                temp, humidity, cfg.cross_section_area, quality, pressure,
            );

            let inherent_error_percent = if dynasty.stage_count <= 1 {
                2.5
            } else if dynasty.stage_count == 2 {
                if idx == 0 { 0.8 } else { 0.5 }
            } else if dynasty.stage_count == 3 {
                match idx { 0 => 0.35, 1 => 0.25, _ => 0.18 }
            } else {
                match idx { 0 => 0.18, 1 => 0.12, 2 => 0.08, _ => 0.05 }
            };

            let self_error = (theoretical * inherent_error_percent / 100.0 + evap)
                / theoretical * 86400.0 / (dynasty.stage_count as f64);

            let amp_factor = 1.0 + (dynasty.stage_count as f64 - 1.0 - idx as f64) * 0.15;
            let output_error = (cumulative_input_error + self_error) * amp_factor;

            nodes.push(ErrorTransferNode {
                stage_index: idx,
                clepsydra_id: cfg.clepsydra_id.clone(),
                input_error_seconds: cumulative_input_error,
                self_error_seconds: self_error,
                output_error_seconds: output_error,
                amplification_factor: amp_factor,
                contribution_percent: 0.0,
                water_level_cm: avg_level,
                flow_rate_mlps: theoretical,
            });

            cumulative_input_error = output_error;
            total_self_error += self_error;
        }

        let total_error = cumulative_input_error;
        for node in nodes.iter_mut() {
            node.contribution_percent = if total_self_error > 1e-9 {
                node.self_error_seconds / total_self_error * 100.0
            } else {
                0.0
            };
        }

        let (bottleneck_idx, bottleneck_reason) = nodes.iter()
            .enumerate()
            .max_by(|a, b| a.1.self_error_seconds.partial_cmp(&b.1.self_error_seconds).unwrap())
            .map(|(i, n)| {
                let reason = if n.water_level_cm < 30.0 {
                    format!("第{}级{}水位偏低，水头不足导致流量稳定性差", i + 1, n.clepsydra_id)
                } else if n.amplification_factor > 1.2 {
                    format!("第{}级{}误差放大系数过高（{:.2}x），需增加缓冲", i + 1, n.clepsydra_id, n.amplification_factor)
                } else {
                    format!("第{}级{}自身固有误差最大（{:.2}s/日），建议优化孔口设计", i + 1, n.clepsydra_id, n.self_error_seconds)
                };
                (i as u32, reason)
            })
            .unwrap_or((0, "未知".to_string()));

        let mut recommendations = Vec::new();
        if dynasty.stage_count < 4 {
            recommendations.push(format!(
                "建议增加补偿壶级数至4级，可将误差再降低约{:.0}%",
                (4.0 - dynasty.stage_count as f64) * 15.0
            ));
        }
        recommendations.push(format!(
            "重点优化瓶颈级{}：采用漫流恒水位可减小误差约{:.0}%",
            nodes.get(bottleneck_idx as usize).map(|n| n.clepsydra_id.clone()).unwrap_or_default(),
            nodes.get(bottleneck_idx as usize).map(|n| n.contribution_percent * 0.6).unwrap_or(25.0)
        ));
        recommendations.push("恒温装置：将水温控制在±1°C范围内，可减少粘性引起的约8%误差".to_string());
        if temp < 18.0 {
            recommendations.push("当前典型水温偏低，建议将环境温度维持在20°C左右以优化流量系数".to_string());
        }

        let compensation_potential = total_error * 0.35;

        Ok(ErrorTransferAnalysis {
            total_error_seconds: total_error,
            nodes,
            bottleneck_stage: bottleneck_idx,
            bottleneck_reason,
            recommendations,
            compensation_potential_seconds: compensation_potential,
        })
    }

    pub fn cross_era_comparison(&self) -> Result<CrossEraComparison> {
        let dynasties = self.get_all_dynasties();
        let pieces = self.get_all_modern();

        let ancient_devices: Vec<_> = dynasties.iter().map(|d| {
            let err = self.calc_dynasty_daily_error(d);
            crate::models::AccuracyComparisonPoint {
                label: format!("{}·{}", d.dynasty_name, d.clepsydra_type.split('（').next().unwrap_or("")),
                category: d.clepsydra_type.clone(),
                daily_error_seconds: err,
                yearly_error_minutes: err * 365.0 / 60.0,
                color_hex: dynasty_color(&d.dynasty_id).to_string(),
                era: "古代".to_string(),
            }
        }).collect();

        let modern_devices: Vec<_> = pieces.iter().map(|m| {
            crate::models::AccuracyComparisonPoint {
                label: m.name.clone(),
                category: m.category.clone(),
                daily_error_seconds: m.daily_error_seconds,
                yearly_error_minutes: m.yearly_error_seconds / 60.0,
                color_hex: modern_color(&m.category).to_string(),
                era: "现代".to_string(),
            }
        }).collect();

        let best_ancient = ancient_devices.iter()
            .min_by(|a, b| a.daily_error_seconds.partial_cmp(&b.daily_error_seconds).unwrap())
            .cloned()
            .unwrap_or_else(|| ancient_devices[0].clone());

        let best_modern = modern_devices.iter()
            .min_by(|a, b| a.daily_error_seconds.partial_cmp(&b.daily_error_seconds).unwrap())
            .cloned()
            .unwrap_or_else(|| modern_devices[0].clone());

        let improvement_factor = if best_modern.daily_error_seconds > 1e-9 {
            best_ancient.daily_error_seconds / best_modern.daily_error_seconds
        } else {
            1e9
        };

        let timeline_data = vec![
            TimelineAccuracy { year: -100, label: "西汉沉箭漏".into(), daily_error_seconds: 900.0, category: "古代".into() },
            TimelineAccuracy { year: 125, label: "东汉浮箭漏".into(), daily_error_seconds: 300.0, category: "古代".into() },
            TimelineAccuracy { year: 650, label: "唐吕才漏".into(), daily_error_seconds: 120.0, category: "古代".into() },
            TimelineAccuracy { year: 1030, label: "宋莲花漏".into(), daily_error_seconds: 45.0, category: "古代".into() },
            TimelineAccuracy { year: 1088, label: "水运仪象台".into(), daily_error_seconds: 50.0, category: "古代".into() },
            TimelineAccuracy { year: 1420, label: "明永乐漏".into(), daily_error_seconds: 65.0, category: "古代".into() },
            TimelineAccuracy { year: 1656, label: "惠更斯摆钟".into(), daily_error_seconds: 60.0, category: "近代".into() },
            TimelineAccuracy { year: 1730, label: "精密摆钟".into(), daily_error_seconds: 0.2, category: "近代".into() },
            TimelineAccuracy { year: 1870, label: "精密机械表".into(), daily_error_seconds: 10.0, category: "近代".into() },
            TimelineAccuracy { year: 1920, label: "天文台机械表".into(), daily_error_seconds: 2.0, category: "现代".into() },
            TimelineAccuracy { year: 1955, label: "铯原子钟".into(), daily_error_seconds: 1e-6, category: "现代".into() },
            TimelineAccuracy { year: 1969, label: "石英手表".into(), daily_error_seconds: 0.5, category: "现代".into() },
            TimelineAccuracy { year: 1978, label: "GPS授时".into(), daily_error_seconds: 1e-5, category: "现代".into() },
        ];

        Ok(CrossEraComparison {
            ancient_devices,
            modern_devices,
            best_ancient,
            best_modern,
            improvement_factor,
            timeline_data,
        })
    }

    pub fn virtual_operate(&self, req: VirtualOperationRequest) -> Result<VirtualOperationResult> {
        let configs = self.config.to_clepsydra_map();
        let cfg = configs.get(&req.clepsydra_id)
            .with_context(|| format!("未找到漏壶配置: {}", req.clepsydra_id))?;

        let target_level = req.target_water_level_cm.clamp(cfg.min_level, cfg.max_level);
        let water_temp = req.water_temp_c.unwrap_or(20.0).clamp(0.0, 50.0);
        let sim_secs = req.simulate_seconds.max(10).min(86400);
        let dt = 1.0f64;
        let steps = sim_secs as usize;
        let sample_interval = (steps / 50).max(1);

        let mut level = (cfg.min_level + cfg.max_level) / 2.0;
        let initial_level = level;
        let pressure = 101.325;
        let humidity = 60.0;
        let quality = 1.0;

        let theoretical = self.hydraulic.calculate_theoretical_flow(level, cfg, water_temp);
        let initial_error_rate = 0.0;

        let mut level_history = Vec::new();
        let mut error_history = Vec::new();
        let mut flow_history = Vec::new();
        let mut cumulative_error = initial_error_rate;

        level_history.push((0.0, level));
        error_history.push((0.0, cumulative_error));
        flow_history.push((0.0, theoretical));

        for step in 0..steps {
            let t = step as f64 + dt;

            let remaining = (target_level - level).abs();
            if remaining > 0.1 {
                let change = (target_level - level).signum() * remaining.min(0.5);
                level += change;
            }

            level = level.clamp(cfg.min_level, cfg.max_level);

            let current_theoretical = self.hydraulic.calculate_theoretical_flow(level, cfg, water_temp);
            let evap = self.hydraulic.calculate_evaporation_rate(
                water_temp, humidity, cfg.cross_section_area, quality, pressure,
            );

            let level_drop_per_sec = if level > cfg.max_level * 0.9 {
                0.002
            } else if level > cfg.max_level * 0.7 {
                0.0008
            } else if level > cfg.max_level * 0.5 {
                0.0003
            } else {
                0.001
            };

            let actual_flow = current_theoretical * (1.0 - level_drop_per_sec * 100.0 / current_theoretical.max(0.01));
            let error_pct = self.hydraulic.calculate_flow_error(current_theoretical, actual_flow - evap);
            cumulative_error = self.hydraulic.update_daily_error(cumulative_error, error_pct, dt);

            if step % sample_interval == 0 || step == steps - 1 {
                level_history.push((t, level));
                error_history.push((t, cumulative_error));
                flow_history.push((t, actual_flow.max(0.0)));
            }
        }

        let mut observations = Vec::new();
        let level_change_pct = (level - initial_level) / (cfg.max_level - cfg.min_level) * 100.0;
        if level_change_pct > 5.0 {
            observations.push(format!(
                "水位升高{:.1}%，水头增加，理论流量上升约{:.2}%",
                level_change_pct,
                ((level / initial_level).sqrt() - 1.0) * 100.0
            ));
        } else if level_change_pct < -5.0 {
            observations.push(format!(
                "水位降低{:.1}%，水头减小，理论流量下降约{:.2}%",
                -level_change_pct,
                (1.0 - (level / initial_level.max(0.01)).sqrt()) * 100.0
            ));
        }

        let error_change = cumulative_error - initial_error_rate;
        if error_change.abs() > 1.0 {
            observations.push(format!(
                "模拟{}秒后，计时误差{}{:.2}秒",
                sim_secs,
                if error_change > 0.0 { "累计增加" } else { "累计减少" },
                error_change.abs()
            ));
        }

        if target_level < cfg.min_level + (cfg.max_level - cfg.min_level) * 0.2 {
            observations.push("警告：目标水位过低，水头不足将导致流量快速衰减，误差增大".to_string());
        }
        if target_level > cfg.max_level * 0.95 {
            observations.push("提示：水位接近上限，处于恒流区最佳工作状态，精度最优".to_string());
        }
        if (water_temp - 20.0).abs() > 10.0 {
            observations.push(format!(
                "水温偏离20°C基准{:+.1}°C，粘性变化已引入约{:.1}%的流量偏差",
                water_temp - 20.0,
                (water_temp - 20.0).abs() * 0.01 * 100.0
            ));
        }

        Ok(VirtualOperationResult {
            clepsydra_id: req.clepsydra_id.clone(),
            initial_level_cm: initial_level,
            final_level_cm: level,
            initial_error_seconds: initial_error_rate,
            final_error_seconds: cumulative_error,
            time_elapsed_simulated: sim_secs,
            level_history,
            error_history,
            flow_history,
            observations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_loader::{AppConfig, HydraulicConfig, ClepsydraEntry, ClickHouseConfig, MqttConfig, ServerConfig, PidConfig, AlertConfig, ChannelConfig};
    use crate::hydraulic::HydraulicModel;

    fn test_app_config() -> AppConfig {
        AppConfig {
            mqtt: MqttConfig {
                broker: "test".to_string(),
                port: 1883,
                topic: "test".to_string(),
                client_id_prefix: "test".to_string(),
                keep_alive_secs: 30,
            },
            clickhouse: ClickHouseConfig {
                url: "http://localhost:8123".to_string(),
                database: "test".to_string(),
                batch_size: 100,
                flush_interval_ms: 1000,
            },
            server: ServerConfig {
                port: 8080,
                cors_origins: vec![],
            },
            hydraulic: HydraulicConfig {
                gravity_cm_s2: 980.665,
                standard_pressure_kpa: 101.325,
                min_dt_seconds: 0.1,
                altitude_m: 50.0,
            },
            pid: PidConfig {
                base_kp: 0.5,
                base_ki: 0.05,
                base_kd: 0.1,
                kf_feedforward: 0.08,
                output_min_ml_s: -1.5,
                output_max_ml_s: 1.5,
                output_rate_limit_ml_s2: 0.3,
                integral_limit: 50.0,
                temp_coefficient_per_deg: 0.01,
                history_window: 5,
            },
            alerts: AlertConfig {
                daily_error_threshold_seconds: 60.0,
                critical_error_multiplier: 2.0,
                water_temp_min_c: 0.0,
                water_temp_max_c: 50.0,
            },
            clepsydras: vec![
                ClepsydraEntry {
                    clepsydra_id: "KD1".to_string(),
                    name: "天上壶".to_string(),
                    max_level_cm: 120.0,
                    min_level_cm: 20.0,
                    standard_flow_ml_s: 2.5,
                    cross_section_cm2: 78.54,
                    orifice_diameter_cm: 0.3,
                    flow_coefficient: 0.62,
                },
                ClepsydraEntry {
                    clepsydra_id: "KD2".to_string(),
                    name: "夜漏壶".to_string(),
                    max_level_cm: 100.0,
                    min_level_cm: 15.0,
                    standard_flow_ml_s: 2.5,
                    cross_section_cm2: 78.54,
                    orifice_diameter_cm: 0.3,
                    flow_coefficient: 0.62,
                },
                ClepsydraEntry {
                    clepsydra_id: "KD3".to_string(),
                    name: "平水壶".to_string(),
                    max_level_cm: 80.0,
                    min_level_cm: 10.0,
                    standard_flow_ml_s: 2.5,
                    cross_section_cm2: 78.54,
                    orifice_diameter_cm: 0.3,
                    flow_coefficient: 0.62,
                },
                ClepsydraEntry {
                    clepsydra_id: "KD4".to_string(),
                    name: "万分水".to_string(),
                    max_level_cm: 60.0,
                    min_level_cm: 5.0,
                    standard_flow_ml_s: 2.5,
                    cross_section_cm2: 78.54,
                    orifice_diameter_cm: 0.3,
                    flow_coefficient: 0.62,
                },
            ],
            channels: ChannelConfig {
                dtu_to_simulator_buffer: 1000,
                simulator_to_compensator_buffer: 500,
                compensator_to_alarm_buffer: 500,
                alarm_broadcast_capacity: 1000,
            },
        }
    }

    fn test_analysis_service() -> AnalysisService {
        let config = Arc::new(test_app_config());
        let store = Arc::new(ClickHouseStore::new("http://localhost:8123", "test").unwrap());
        let hydraulic = Arc::new(HydraulicModel::new());
        AnalysisService::new(config, store, hydraulic)
    }

    // ============================================================
    // 1. 精度对比验证日误差范围测试
    // ============================================================
    mod dynasty_compare_tests {
        use super::*;

        #[test]
        fn test_normal_compare_song_vs_tang() {
            let svc = test_analysis_service();
            let result = svc.compare_dynasties("SONG_YITIAN", "TANG_JINGLU");
            assert!(result.is_ok(), "宋代vs唐代对比应成功");
            let cmp = result.unwrap();
            assert!(cmp.left_daily_error_seconds > 0.0, "日误差应为正数");
            assert!(cmp.right_daily_error_seconds > 0.0, "日误差应为正数");
            assert!(cmp.left_daily_error_seconds < cmp.right_daily_error_seconds,
                "宋代日误差应小于唐代");
            assert_eq!(cmp.winner, "宋代", "宋代精度应高于唐代");
            assert!(cmp.error_ratio > 0.0, "误差比值应为正数");
            assert!(!cmp.key_differences.is_empty(), "应有关键差异说明");
            assert!(!cmp.flow_comparison.is_empty(), "应有各级流量对比数据");
        }

        #[test]
        fn test_boundary_best_vs_worst() {
            let svc = test_analysis_service();
            let result = svc.compare_dynasties("SONG_LIANHUA", "HAN_CHENJIAN");
            assert!(result.is_ok());
            let cmp = result.unwrap();
            assert!(cmp.left_daily_error_seconds < cmp.right_daily_error_seconds,
                "宋代莲花漏日误差应小于汉代沉箭漏");
            assert!(cmp.error_ratio > 0.0 && cmp.error_ratio < 1.0,
                "最优 vs 最差，左/右误差比应在0-1之间 (误差比={})", cmp.error_ratio);
            assert_eq!(cmp.winner, "宋代");
            assert_eq!(cmp.left_dynasty.stage_count, 3);
            assert_eq!(cmp.right_dynasty.stage_count, 1);
            assert!(cmp.left_daily_error_seconds > 0.0);
            assert!(cmp.right_daily_error_seconds > 0.0);
        }

        #[test]
        fn test_same_dynasty_self_compare() {
            let svc = test_analysis_service();
            let result = svc.compare_dynasties("SONG_YITIAN", "SONG_YITIAN");
            assert!(result.is_ok(), "自身对比应成功");
            let cmp = result.unwrap();
            assert!((cmp.left_daily_error_seconds - cmp.right_daily_error_seconds).abs() < 1e-6,
                "自身对比的两个日误差应完全相等");
            assert!((cmp.error_ratio - 1.0).abs() < 1e-6,
                "自身对比的误差比应为1");
        }

        #[test]
        fn test_invalid_left_id() {
            let svc = test_analysis_service();
            let result = svc.compare_dynasties("INVALID_ID", "SONG_YITIAN");
            assert!(result.is_err(), "无效ID应返回错误");
            assert!(result.unwrap_err().to_string().contains("未找到"),
                "错误信息应包含未找到提示");
        }

        #[test]
        fn test_invalid_right_id() {
            let svc = test_analysis_service();
            let result = svc.compare_dynasties("SONG_YITIAN", "BAD_ID");
            assert!(result.is_err());
        }

        #[test]
        fn test_all_dynasties_have_reasonable_error_range() {
            let svc = test_analysis_service();
            let dynasties = svc.get_all_dynasties();
            assert_eq!(dynasties.len(), 6, "应有6个朝代配置");
            let mut errors: Vec<(String, f64, u32)> = Vec::new();
            for d in &dynasties {
                let err = svc.calc_dynasty_daily_error(d);
                assert!(err > 0.0, "{} 日误差应>0", d.dynasty_name);
                assert!(err < 100000.0, "{} 日误差应<10万秒(合理范围)", d.dynasty_name);
                assert!(d.stage_count >= 1 && d.stage_count <= 4,
                    "级数应在1-4之间");
                assert!(!d.configs.is_empty(), "每朝至少1个漏壶配置");
                errors.push((d.dynasty_name.clone(), err, d.stage_count));
            }
            errors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            assert!(errors.first().unwrap().2 >= 3,
                "误差最小的朝代应是3级或4级");
            assert!(errors.last().unwrap().2 == 1,
                "误差最大的朝代应是1级");
        }

        #[test]
        fn test_more_stages_better_accuracy_trend() {
            let svc = test_analysis_service();
            let d1 = svc.get_dynasty("HAN_CHENJIAN").unwrap();
            let d2 = svc.get_dynasty("HAN_FUJIAN").unwrap();
            let e1 = svc.calc_dynasty_daily_error(&d1);
            let e2 = svc.calc_dynasty_daily_error(&d2);
            assert!(e1 > e2, "2级浮箭漏应比1级沉箭漏更精确 ({} > {})", e1, e2);
            assert_eq!(d1.stage_count, 1);
            assert_eq!(d2.stage_count, 2);
        }
    }

    // ============================================================
    // 2. 跨时代对比验证计时可靠性测试
    // ============================================================
    mod cross_era_tests {
        use super::*;

        #[test]
        fn test_normal_cross_era_structure() {
            let svc = test_analysis_service();
            let result = svc.cross_era_comparison();
            assert!(result.is_ok(), "跨时代对比应成功返回");
            let cmp = result.unwrap();
            assert!(!cmp.ancient_devices.is_empty(), "古代设备列表不应空");
            assert!(!cmp.modern_devices.is_empty(), "现代设备列表不应空");
            assert_eq!(cmp.ancient_devices.len(), 6, "应有6款古代设备");
            assert_eq!(cmp.modern_devices.len(), 8, "应有8款现代设备");
        }

        #[test]
        fn test_modern_more_accurate_than_ancient() {
            let svc = test_analysis_service();
            let cmp = svc.cross_era_comparison().unwrap();
            assert!(cmp.best_ancient.daily_error_seconds > cmp.best_modern.daily_error_seconds,
                "现代最优精度应高于古代最优 (古代最优: {} > 现代最优: {})",
                cmp.best_ancient.daily_error_seconds, cmp.best_modern.daily_error_seconds);
            assert!(cmp.improvement_factor > 1.0,
                "古代→现代精度改进应>1倍");
        }

        #[test]
        fn test_timeline_data_consistency() {
            let svc = test_analysis_service();
            let cmp = svc.cross_era_comparison().unwrap();
            assert_eq!(cmp.timeline_data.len(), 13, "时间线应有13个节点");
            for i in 1..cmp.timeline_data.len() {
                assert!(cmp.timeline_data[i].year > cmp.timeline_data[i-1].year,
                    "时间线年份应递增");
            }
            assert_eq!(cmp.timeline_data[0].year, -100, "起点应为公元前100年");
            assert_eq!(cmp.timeline_data.last().unwrap().year, 1978, "终点应为1978年(GPS)");
        }

        #[test]
        fn test_modern_accuracy_ordering() {
            let svc = test_analysis_service();
            let pieces = svc.get_all_modern();
            assert_eq!(pieces.len(), 8);
            let atomics: Vec<_> = pieces.iter().filter(|p| p.category == "原子").collect();
            assert_eq!(atomics.len(), 2);
            let electronics: Vec<_> = pieces.iter().filter(|p| p.category == "电子").collect();
            assert_eq!(electronics.len(), 2);
            for a in &atomics {
                for e in &electronics {
                    assert!(a.daily_error_seconds < e.daily_error_seconds,
                        "原子钟{}应比电子表{}更精确", a.name, e.name);
                }
            }
        }

        #[test]
        fn test_ancient_error_in_reasonable_range() {
            let svc = test_analysis_service();
            let cmp = svc.cross_era_comparison().unwrap();
            for d in &cmp.ancient_devices {
                assert!(d.daily_error_seconds > 0.0,
                    "古代设备{}日误差应>0 (实际: {})", d.label, d.daily_error_seconds);
                assert!(d.daily_error_seconds < 1e6,
                    "古代设备{}日误差不应超过1e6秒", d.label);
                assert_eq!(d.era, "古代");
            }
            let mut sorted = cmp.ancient_devices.clone();
            sorted.sort_by(|a, b| a.daily_error_seconds.partial_cmp(&b.daily_error_seconds).unwrap());
            assert!(sorted.first().unwrap().daily_error_seconds < sorted.last().unwrap().daily_error_seconds,
                "古代设备间应有精度差异");
        }

        #[test]
        fn test_best_ancient_is_song_dynasty() {
            let svc = test_analysis_service();
            let cmp = svc.cross_era_comparison().unwrap();
            assert!(cmp.best_ancient.label.contains("宋") || cmp.best_ancient.label.contains("明"),
                "古代最优应是宋或明代的 ({})", cmp.best_ancient.label);
            assert!(cmp.best_ancient.daily_error_seconds > 0.0);
        }

        #[test]
        fn test_best_modern_is_atomic_or_satellite() {
            let svc = test_analysis_service();
            let cmp = svc.cross_era_comparison().unwrap();
            let is_top = cmp.best_modern.category == "原子" || cmp.best_modern.category == "卫星";
            assert!(is_top, "现代最优应是原子或卫星类 ({})", cmp.best_modern.category);
        }
    }

    // ============================================================
    // 3. 误差传递验证累积效应测试
    // ============================================================
    mod error_transfer_tests {
        use super::*;

        #[test]
        fn test_normal_song_4stage_transfer() {
            let svc = test_analysis_service();
            let result = svc.analyze_error_transfer("SONG_YITIAN");
            assert!(result.is_ok());
            let analysis = result.unwrap();
            assert_eq!(analysis.nodes.len(), 4, "宋代4级漏壶应有4个节点");
            assert!(analysis.total_error_seconds > 0.0, "总误差应>0");
            assert!(analysis.bottleneck_stage < 4, "瓶颈级索引应<4");
            assert!(!analysis.bottleneck_reason.is_empty(), "应有瓶颈原因描述");
            assert!(!analysis.recommendations.is_empty(), "应有优化建议");
            assert!(analysis.compensation_potential_seconds > 0.0,
                "应有正向补偿潜力");
            assert!(analysis.compensation_potential_seconds < analysis.total_error_seconds,
                "补偿潜力不应超过总误差");
        }

        #[test]
        fn test_cumulative_effect_output_greater_than_input() {
            let svc = test_analysis_service();
            let analysis = svc.analyze_error_transfer("SONG_YITIAN").unwrap();
            for i in 0..analysis.nodes.len() {
                if i > 0 {
                    let prev_out = analysis.nodes[i-1].output_error_seconds;
                    let curr_in = analysis.nodes[i].input_error_seconds;
                    assert!((prev_out - curr_in).abs() < 1e-6,
                        "第{}级输入应等于第{}级输出", i, i-1);
                }
                let node = &analysis.nodes[i];
                assert!(node.output_error_seconds >= node.input_error_seconds,
                    "第{}级输出误差应≥输入误差 (累积效应)", i);
                assert!(node.self_error_seconds > 0.0,
                    "第{}级自身误差应>0", i);
            }
        }

        #[test]
        fn test_amplification_factor_decreases_downstream() {
            let svc = test_analysis_service();
            let analysis = svc.analyze_error_transfer("SONG_YITIAN").unwrap();
            for i in 1..analysis.nodes.len() {
                let prev_amp = analysis.nodes[i-1].amplification_factor;
                let curr_amp = analysis.nodes[i].amplification_factor;
                assert!(prev_amp >= curr_amp || i == analysis.nodes.len() - 1,
                    "上游放大系数应≥下游 (多级缓冲效应)");
            }
        }

        #[test]
        fn test_contribution_percent_sum() {
            let svc = test_analysis_service();
            let analysis = svc.analyze_error_transfer("SONG_YITIAN").unwrap();
            let total_contrib: f64 = analysis.nodes.iter().map(|n| n.contribution_percent).sum();
            assert!((total_contrib - 100.0).abs() < 1.0,
                "各节点贡献百分比之和应≈100% (实际: {})", total_contrib);
        }

        #[test]
        fn test_boundary_1stage_no_cumulation() {
            let svc = test_analysis_service();
            let analysis = svc.analyze_error_transfer("HAN_CHENJIAN").unwrap();
            assert_eq!(analysis.nodes.len(), 1, "单级漏壶只有1个节点");
            let node = &analysis.nodes[0];
            assert!((node.input_error_seconds - 0.0).abs() < 1e-9,
                "单级第一级输入误差应为0");
            assert!(node.output_error_seconds == node.self_error_seconds,
                "单级输出误差=自身误差 (无多级累积)");
            assert!(node.amplification_factor >= 1.0,
                "放大系数应≥1.0");
        }

        #[test]
        fn test_more_stages_higher_accuracy_per_stage() {
            let svc = test_analysis_service();
            let a1 = svc.analyze_error_transfer("HAN_CHENJIAN").unwrap();
            let a4 = svc.analyze_error_transfer("SONG_YITIAN").unwrap();
            let err_per_stage_1 = a1.total_error_seconds / a1.nodes.len() as f64;
            let err_per_stage_4 = a4.total_error_seconds / a4.nodes.len() as f64;
            assert!(err_per_stage_1 > err_per_stage_4,
                "4级漏壶的单级平均误差应小于1级 (多级补偿效应)");
        }

        #[test]
        fn test_invalid_dynasty_id() {
            let svc = test_analysis_service();
            let result = svc.analyze_error_transfer("NON_EXISTENT");
            assert!(result.is_err(), "无效朝代ID应返回错误");
        }

        #[test]
        fn test_bottleneck_is_largest_contributor() {
            let svc = test_analysis_service();
            let analysis = svc.analyze_error_transfer("TANG_JINGLU").unwrap();
            let bn = analysis.bottleneck_stage as usize;
            let bn_contrib = analysis.nodes[bn].contribution_percent;
            for (i, node) in analysis.nodes.iter().enumerate() {
                if i != bn {
                    assert!(node.contribution_percent <= bn_contrib + 1e-9,
                        "瓶颈级{}贡献%应最大 ({} vs 第{}级的{})",
                        bn, bn_contrib, i, node.contribution_percent);
                }
            }
        }
    }

    // ============================================================
    // 4. 虚拟体验测试操作直观性测试
    // ============================================================
    mod virtual_operate_tests {
        use super::*;
        use crate::models::VirtualOperationRequest;

        fn make_request(id: &str, level: f64, temp: f64, secs: u32) -> VirtualOperationRequest {
            VirtualOperationRequest {
                clepsydra_id: id.to_string(),
                target_water_level_cm: level,
                water_temp_c: Some(temp),
                simulate_seconds: secs,
            }
        }

        #[test]
        fn test_normal_mid_level_operation() {
            let svc = test_analysis_service();
            let req = make_request("KD1", 70.0, 20.0, 3600);
            let result = svc.virtual_operate(req);
            assert!(result.is_ok(), "正常虚拟操作应成功");
            let r = result.unwrap();
            assert_eq!(r.clepsydra_id, "KD1");
            assert!(r.final_level_cm > 0.0);
            assert!(r.time_elapsed_simulated == 3600);
            assert!(!r.level_history.is_empty());
            assert!(!r.error_history.is_empty());
            assert!(!r.flow_history.is_empty());
            assert_eq!(r.level_history[0].0, 0.0, "历史起点时间应为0");
        }

        #[test]
        fn test_boundary_max_level() {
            let svc = test_analysis_service();
            let req = make_request("KD1", 120.0, 20.0, 600);
            let r = svc.virtual_operate(req).unwrap();
            assert!(r.final_level_cm <= 120.0 + 1e-6, "最高水位不应超过上限");
            let has_high_hint = r.observations.iter().any(|o| o.contains("上限") || o.contains("最佳"));
            assert!(has_high_hint, "最高水位时应有恒流区最佳提示");
        }

        #[test]
        fn test_boundary_min_level() {
            let svc = test_analysis_service();
            let req = make_request("KD2", 15.0, 20.0, 600);
            let r = svc.virtual_operate(req).unwrap();
            assert!(r.final_level_cm >= 15.0 - 1e-6, "最低水位不应低于下限");
            let has_low_warn = r.observations.iter().any(|o| o.contains("过低") || o.contains("警告"));
            assert!(has_low_warn, "低水位应有警告提示");
        }

        #[test]
        fn test_level_clamped_to_valid_range() {
            let svc = test_analysis_service();
            let req1 = make_request("KD3", 500.0, 20.0, 100);
            let r1 = svc.virtual_operate(req1).unwrap();
            assert!(r1.final_level_cm <= 80.0 + 1e-6, "超上限应被截断到max");
            let req2 = make_request("KD3", -10.0, 20.0, 100);
            let r2 = svc.virtual_operate(req2).unwrap();
            assert!(r2.final_level_cm >= 10.0 - 1e-6, "负水位应被截断到min");
        }

        #[test]
        fn test_higher_level_higher_flow() {
            let svc = test_analysis_service();
            let low = svc.virtual_operate(make_request("KD1", 30.0, 20.0, 100)).unwrap();
            let high = svc.virtual_operate(make_request("KD1", 110.0, 20.0, 100)).unwrap();
            let low_flow = low.flow_history.last().unwrap().1;
            let high_flow = high.flow_history.last().unwrap().1;
            assert!(high_flow > low_flow,
                "高水位应对应高流量 (高: {}, 低: {})", high_flow, low_flow);
        }

        #[test]
        fn test_simulation_duration_matches_history() {
            let svc = test_analysis_service();
            for secs in [60u32, 600, 3600, 86400] {
                let r = svc.virtual_operate(make_request("KD1", 60.0, 20.0, secs)).unwrap();
                assert_eq!(r.time_elapsed_simulated, secs);
                let last_time = r.level_history.last().unwrap().0;
                assert!((last_time - secs as f64).abs() <= 1.0,
                    "历史最后时间点应接近总时长 ({} vs {})", last_time, secs);
            }
        }

        #[test]
        fn test_min_sim_seconds_clamped() {
            let svc = test_analysis_service();
            let r = svc.virtual_operate(make_request("KD1", 60.0, 20.0, 5)).unwrap();
            assert!(r.time_elapsed_simulated >= 10, "模拟时长最小应为10秒");
        }

        #[test]
        fn test_max_sim_seconds_clamped() {
            let svc = test_analysis_service();
            let r = svc.virtual_operate(make_request("KD1", 60.0, 20.0, 200000)).unwrap();
            assert!(r.time_elapsed_simulated <= 86400, "模拟时长最大应为1天");
        }

        #[test]
        fn test_invalid_clepsydra_id() {
            let svc = test_analysis_service();
            let req = make_request("INVALID", 50.0, 20.0, 100);
            let result = svc.virtual_operate(req);
            assert!(result.is_err(), "无效漏壶ID应返回错误");
            assert!(result.unwrap_err().to_string().contains("未找到"));
        }

        #[test]
        fn test_water_temp_extreme_triggers_observation() {
            let svc = test_analysis_service();
            let cold = svc.virtual_operate(make_request("KD1", 70.0, 0.0, 200)).unwrap();
            let hot = svc.virtual_operate(make_request("KD1", 70.0, 40.0, 200)).unwrap();
            let cold_has_temp = cold.observations.iter().any(|o| o.contains("水温"));
            let hot_has_temp = hot.observations.iter().any(|o| o.contains("水温"));
            assert!(cold_has_temp, "极端低温应有水温相关观察");
            assert!(hot_has_temp, "极端高温应有水温相关观察");
        }

        #[test]
        fn test_observations_are_informative() {
            let svc = test_analysis_service();
            let r = svc.virtual_operate(make_request("KD1", 25.0, 35.0, 600)).unwrap();
            assert!(!r.observations.is_empty(), "至少应有1条观察结论");
            for obs in &r.observations {
                assert!(!obs.is_empty(), "观察内容不应为空");
                assert!(obs.chars().count() > 5, "观察内容应有足够长度");
            }
        }

        #[test]
        fn test_level_history_non_decreasing_to_target() {
            let svc = test_analysis_service();
            let r = svc.virtual_operate(make_request("KD1", 100.0, 20.0, 300)).unwrap();
            let first_level = r.level_history[0].1;
            let last_level = r.level_history.last().unwrap().1;
            assert!(last_level > first_level,
                "目标水位高于初始时，水位应上升 ({} → {})", first_level, last_level);
        }

        #[test]
        fn test_history_sizes_match() {
            let svc = test_analysis_service();
            let r = svc.virtual_operate(make_request("KD1", 70.0, 20.0, 1000)).unwrap();
            assert_eq!(r.level_history.len(), r.error_history.len());
            assert_eq!(r.error_history.len(), r.flow_history.len());
            assert!(r.level_history.len() > 2, "历史点数应足够用于绘图");
        }
    }

    // ============================================================
    // 5. 数据模型验证测试
    // ============================================================
    mod data_model_tests {
        use super::*;

        #[test]
        fn test_dynasty_configs_count_matches_stage_count() {
            let svc = test_analysis_service();
            for d in svc.get_all_dynasties() {
                assert_eq!(d.configs.len() as u32, d.stage_count,
                    "{}: configs数量({})应等于stage_count({})",
                    d.dynasty_name, d.configs.len(), d.stage_count);
            }
        }

        #[test]
        fn test_clepsydra_config_max_gt_min() {
            let svc = test_analysis_service();
            for d in svc.get_all_dynasties() {
                for c in &d.configs {
                    assert!(c.max_level > c.min_level,
                        "{}: max({})应大于min({})", c.clepsydra_id, c.max_level, c.min_level);
                    assert!(c.standard_flow > 0.0);
                    assert!(c.cross_section_area > 0.0);
                    assert!(c.orifice_diameter > 0.0);
                    assert!(c.flow_coefficient > 0.0);
                    assert!(c.flow_coefficient < 1.0);
                }
            }
        }

        #[test]
        fn test_modern_timepiece_fields_valid() {
            let svc = test_analysis_service();
            for m in svc.get_all_modern() {
                assert!(!m.piece_id.is_empty());
                assert!(!m.name.is_empty());
                assert!(!m.technology.is_empty());
                assert!(m.daily_error_seconds > 0.0);
                assert!(m.yearly_error_seconds > 0.0);
                assert!(m.invention_year > 0);
                assert!(m.invention_year < 2100);
                assert!(["机械", "电子", "原子", "卫星"].iter()
                    .any(|c| c == &m.category),
                    "{} 的类别 {} 应在 [机械,电子,原子,卫星] 中", m.name, m.category);
            }
        }

        #[test]
        fn test_dynasty_color_has_default() {
            assert_eq!(dynasty_color("HAN_CHENJIAN"), "#8B4513");
            assert_eq!(dynasty_color("SONG_YITIAN"), "#4ECDC4");
            assert_eq!(dynasty_color("UNKNOWN"), "#666666");
        }

        #[test]
        fn test_modern_color_has_default() {
            assert_eq!(modern_color("原子"), "#E74C3C");
            assert_eq!(modern_color("电子"), "#3498DB");
            assert_eq!(modern_color("神秘类"), "#95A5A6");
        }
    }
}

