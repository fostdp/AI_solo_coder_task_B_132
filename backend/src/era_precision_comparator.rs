use std::sync::Arc;
use anyhow::{Context, Result};

use crate::config_loader::AppConfig;
use crate::hydraulic::HydraulicModel;
use crate::models::{
    ClepsydraConfig, DynastyClepsydraConfig, DynastyComparison, FlowComparisonPoint,
};

pub struct EraPrecisionComparator {
    pub config: Arc<AppConfig>,
    pub hydraulic: Arc<HydraulicModel>,
}

impl EraPrecisionComparator {
    pub fn new(config: Arc<AppConfig>, hydraulic: Arc<HydraulicModel>) -> Self {
        Self { config, hydraulic }
    }

    pub fn calc_dynasty_daily_error(&self, dynasty: &DynastyClepsydraConfig) -> f64 {
        let temp = dynasty.typical_water_temp_c;
        let humidity = 60.0;
        let quality = 1.0;
        let pressure = self.hydraulic.altitude_to_pressure(self.config.hydraulic.altitude_m);

        let mut cumulative_error = 0.0;
        let seconds_per_day = 86400.0;

        for cfg in &dynasty.configs {
            let level_range = cfg.max_level - cfg.min_level;
            let avg_level = cfg.min_level + level_range * 0.7;

            let theoretical = self.hydraulic.calculate_theoretical_flow(avg_level, cfg, temp);

            let level_drop_ratio = match dynasty.stage_count {
                0 | 1 => 0.25,
                2 => 0.10,
                3 => 0.035,
                _ => 0.018,
            };

            let actual_flow_deviation = theoretical * level_drop_ratio;
            let evap = self.hydraulic.calculate_evaporation_rate(
                temp, humidity, cfg.cross_section_area, quality, pressure,
            );

            let effective_error_rate = (actual_flow_deviation + evap) / theoretical * 100.0;
            let stage_error = effective_error_rate / 100.0 * seconds_per_day;
            cumulative_error += stage_error / (dynasty.stage_count as f64).sqrt().max(1.0);
        }

        let base_historical = dynasty.historical_daily_error_seconds;
        (base_historical + cumulative_error) / 2.0
    }

    pub fn compare_dynasties(
        &self,
        left: &DynastyClepsydraConfig,
        right: &DynastyClepsydraConfig,
    ) -> Result<DynastyComparison> {
        let left_error = self.calc_dynasty_daily_error(left);
        let right_error = self.calc_dynasty_daily_error(right);

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

        let flow_comparison: Vec<FlowComparisonPoint> =
            (0..left.configs.len().max(right.configs.len()))
                .map(|i| {
                    let left_cfg = left.configs.get(i);
                    let right_cfg = right.configs.get(i);
                    FlowComparisonPoint {
                        stage: format!("第{}级", i + 1),
                        left_flow_mlps: left_cfg.map(|c| c.standard_flow).unwrap_or(0.0),
                        right_flow_mlps: right_cfg.map(|c| c.standard_flow).unwrap_or(0.0),
                        left_level_cm: left_cfg
                            .map(|c| (c.max_level + c.min_level) / 2.0)
                            .unwrap_or(0.0),
                        right_level_cm: right_cfg
                            .map(|c| (c.max_level + c.min_level) / 2.0)
                            .unwrap_or(0.0),
                    }
                })
                .collect();

        Ok(DynastyComparison {
            left_dynasty: left.clone(),
            right_dynasty: right.clone(),
            left_daily_error_seconds: left_error,
            right_daily_error_seconds: right_error,
            error_ratio,
            winner,
            key_differences: key_diff,
            flow_comparison,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_loader::HydraulicConfig;

    fn make_test_config() -> Arc<AppConfig> {
        Arc::new(AppConfig {
            hydraulic: HydraulicConfig::default(),
            ..Default::default()
        })
    }

    #[test]
    fn test_calc_error_returns_positive() {
        let cfg = make_test_config();
        let hyd = Arc::new(HydraulicModel::new());
        let comp = EraPrecisionComparator::new(cfg, hyd);
        let mut dyn_cfg = make_simple_dynasty("A", 2);
        let err = comp.calc_dynasty_daily_error(&dyn_cfg);
        assert!(err > 0.0, "daily error should be positive, got {}", err);
    }

    #[test]
    fn test_more_stages_less_error() {
        let cfg = make_test_config();
        let hyd = Arc::new(HydraulicModel::new());
        let comp = EraPrecisionComparator::new(cfg, hyd);
        let d1 = make_simple_dynasty("S1", 1);
        let d4 = make_simple_dynasty("S4", 4);
        let e1 = comp.calc_dynasty_daily_error(&d1);
        let e4 = comp.calc_dynasty_daily_error(&d4);
        assert!(e4 < e1, "4-stage ({}) should be more accurate than 1-stage ({})", e4, e1);
    }

    #[test]
    fn test_compare_returns_valid_comparison() {
        let cfg = make_test_config();
        let hyd = Arc::new(HydraulicModel::new());
        let comp = EraPrecisionComparator::new(cfg, hyd);
        let left = make_simple_dynasty("L", 2);
        let right = make_simple_dynasty("R", 4);
        let cmp = comp.compare_dynasties(&left, &right).unwrap();
        assert!(cmp.left_daily_error_seconds > 0.0);
        assert!(cmp.right_daily_error_seconds > 0.0);
        assert!(!cmp.winner.is_empty());
        assert!(!cmp.flow_comparison.is_empty());
        assert!(!cmp.key_differences.is_empty());
    }

    fn make_simple_dynasty(id: &str, stages: u32) -> DynastyClepsydraConfig {
        use crate::models::ClepsydraConfig;
        let configs: Vec<ClepsydraConfig> = (0..stages)
            .map(|i| ClepsydraConfig {
                clepsydra_id: format!("{}-K{}", id, i),
                name: format!("{}壶{}", id, i),
                max_level: 100.0,
                min_level: 20.0,
                standard_flow: 1.0,
                cross_section_area: 314.0,
                orifice_diameter: 0.5,
                flow_coefficient: 0.6,
            })
            .collect();
        DynastyClepsydraConfig {
            dynasty_id: id.to_string(),
            dynasty_name: format!("{}朝", id),
            era: "test".to_string(),
            clepsydra_type: "测试".to_string(),
            stage_count: stages,
            description: "test".to_string(),
            historical_daily_error_seconds: 300.0,
            typical_water_temp_c: 20.0,
            material: "铜".to_string(),
            configs,
            reference_year: 1000,
            historical_references: vec!["test".to_string()],
            data_source: "unit test".to_string(),
            uncertainty_percent: 10.0,
        }
    }
}
