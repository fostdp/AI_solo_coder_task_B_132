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
