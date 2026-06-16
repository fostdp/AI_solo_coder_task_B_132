use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Context, Result};

use crate::config_loader::AppConfig;
use crate::hydraulic::HydraulicModel;
use crate::models::{ClepsydraConfig, VirtualOperationRequest, VirtualOperationResult};

pub struct VrClepsydraEngine {
    pub config: Arc<AppConfig>,
    pub hydraulic: Arc<HydraulicModel>,
}

impl VrClepsydraEngine {
    pub fn new(config: Arc<AppConfig>, hydraulic: Arc<HydraulicModel>) -> Self {
        Self { config, hydraulic }
    }

    pub fn run_virtual_operation(
        &self,
        req: &VirtualOperationRequest,
    ) -> Result<VirtualOperationResult> {
        let configs = self.config.to_clepsydra_map();
        let cfg = configs
            .get(&req.clepsydra_id)
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
            if remaining > 0.05 {
                let adaptive_step = 0.2 + 0.08 * remaining;
                let change = (target_level - level).signum() * remaining.min(adaptive_step);
                level += change;
            }

            level = level.clamp(cfg.min_level, cfg.max_level);

            let current_theoretical = self
                .hydraulic
                .calculate_theoretical_flow(level, cfg, water_temp);
            let evap = self.hydraulic.calculate_evaporation_rate(
                water_temp,
                humidity,
                cfg.cross_section_area,
                quality,
                pressure,
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

            let actual_flow = current_theoretical
                * (1.0 - level_drop_per_sec * 100.0 / current_theoretical.max(0.01));
            let error_pct = self
                .hydraulic
                .calculate_flow_error(current_theoretical, actual_flow - evap);
            cumulative_error = self
                .hydraulic
                .update_daily_error(cumulative_error, error_pct, dt);

            if step % sample_interval == 0 || step == steps - 1 {
                level_history.push((t, level));
                error_history.push((t, cumulative_error));
                flow_history.push((t, actual_flow.max(0.0)));
            }
        }

        let mut observations = Vec::new();
        let level_change_pct =
            (level - initial_level) / (cfg.max_level - cfg.min_level) * 100.0;
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
                "模拟期间累计日误差变化{:+.2}秒，当前水平{}秒/日",
                error_change,
                if cumulative_error > 60.0 {
                    "较差".to_string()
                } else if cumulative_error > 10.0 {
                    "一般".to_string()
                } else {
                    "良好".to_string()
                }
            ));
        }

        if water_temp < 5.0 {
            observations.push(format!(
                "水温过低（{:.1}°C），水粘度升高，实际流量比理论值低约8%",
                water_temp
            ));
        } else if water_temp > 40.0 {
            observations.push(format!(
                "水温过高（{:.1}°C），蒸发加剧，计时偏差风险上升",
                water_temp
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

        if observations.is_empty() {
            observations.push(
                "水位变化平稳，流量稳定，处于最优工作区间".to_string(),
            );
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
    use crate::config_loader::HydraulicConfig;

    fn make_cfg() -> Arc<AppConfig> {
        Arc::new(AppConfig {
            hydraulic: HydraulicConfig::default(),
            ..Default::default()
        })
    }

    fn req(id: &str, level: f64, secs: u32) -> VirtualOperationRequest {
        VirtualOperationRequest {
            clepsydra_id: id.to_string(),
            target_water_level_cm: level,
            water_temp_c: Some(20.0),
            simulate_seconds: secs,
        }
    }

    #[test]
    fn test_invalid_id_returns_err() {
        let cfg = make_cfg();
        let hyd = Arc::new(HydraulicModel::new());
        let eng = VrClepsydraEngine::new(cfg, hyd);
        let r = eng.run_virtual_operation(&req("XX", 50.0, 100));
        assert!(r.is_err());
    }

    #[test]
    fn test_normal_operation_returns_history() {
        let cfg = make_cfg();
        let hyd = Arc::new(HydraulicModel::new());
        let eng = VrClepsydraEngine::new(cfg, hyd);
        let r = eng.run_virtual_operation(&req("KD1", 70.0, 60)).unwrap();
        assert!(!r.level_history.is_empty());
        assert!(!r.flow_history.is_empty());
        assert!(!r.error_history.is_empty());
        assert!(!r.observations.is_empty());
    }

    #[test]
    fn test_level_clamped_to_min() {
        let cfg = make_cfg();
        let hyd = Arc::new(HydraulicModel::new());
        let eng = VrClepsydraEngine::new(cfg, hyd);
        let r = eng.run_virtual_operation(&req("KD1", 0.0, 30)).unwrap();
        assert!(r.final_level_cm >= 20.0); // KD1 min_level
    }

    #[test]
    fn test_level_clamped_to_max() {
        let cfg = make_cfg();
        let hyd = Arc::new(HydraulicModel::new());
        let eng = VrClepsydraEngine::new(cfg, hyd);
        let r = eng.run_virtual_operation(&req("KD1", 999.0, 30)).unwrap();
        assert!(r.final_level_cm <= 120.0); // KD1 max_level
    }

    #[test]
    fn test_sim_seconds_clamped() {
        let cfg = make_cfg();
        let hyd = Arc::new(HydraulicModel::new());
        let eng = VrClepsydraEngine::new(cfg, hyd);
        let r1 = eng.run_virtual_operation(&req("KD1", 70.0, 1)).unwrap();
        assert_eq!(r1.time_elapsed_simulated, 10); // min 10
        let r2 = eng.run_virtual_operation(&req("KD1", 70.0, 999999)).unwrap();
        assert_eq!(r2.time_elapsed_simulated, 86400); // max 86400
    }

    #[test]
    fn test_level_trend_matches_target() {
        let cfg = make_cfg();
        let hyd = Arc::new(HydraulicModel::new());
        let eng = VrClepsydraEngine::new(cfg, hyd);
        // target higher than mid (70.0)
        let r = eng.run_virtual_operation(&req("KD1", 110.0, 600)).unwrap();
        assert!(r.final_level_cm > r.initial_level_cm);
    }

    #[test]
    fn test_history_sizes_match() {
        let cfg = make_cfg();
        let hyd = Arc::new(HydraulicModel::new());
        let eng = VrClepsydraEngine::new(cfg, hyd);
        let r = eng.run_virtual_operation(&req("KD1", 80.0, 100)).unwrap();
        assert_eq!(r.level_history.len(), r.error_history.len());
        assert_eq!(r.level_history.len(), r.flow_history.len());
    }

    #[test]
    fn test_extreme_temp_produces_observation() {
        let cfg = make_cfg();
        let hyd = Arc::new(HydraulicModel::new());
        let eng = VrClepsydraEngine::new(cfg, hyd);
        let r = eng
            .run_virtual_operation(&VirtualOperationRequest {
                clepsydra_id: "KD1".into(),
                target_water_level_cm: 70.0,
                water_temp_c: Some(95.0),
                simulate_seconds: 60,
            })
            .unwrap();
        assert!(r
            .observations
            .iter()
            .any(|o| o.contains("水温过高") || o.contains("蒸发")));
    }
}
