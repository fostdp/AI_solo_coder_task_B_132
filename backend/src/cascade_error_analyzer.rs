use anyhow::Result;

use crate::models::{
    ClepsydraConfig, DynastyClepsydraConfig, ErrorTransferAnalysis, ErrorTransferNode,
};

const INHERENT_ERROR_TABLE: [[f64; 4]; 4] = [
    // idx=0   1      2      3
    [2.50,  0.00,  0.00,  0.00], // N=1
    [0.80,  0.50,  0.00,  0.00], // N=2
    [0.35,  0.25,  0.18,  0.00], // N=3
    [0.18,  0.12,  0.08,  0.05], // N=4
];

fn inherent_error_pct(stages: u32, idx: u32) -> f64 {
    let n = stages.clamp(1, 4) as usize - 1;
    let i = idx as usize;
    if i < 4 { INHERENT_ERROR_TABLE[n][i] } else { 0.05 }
}

fn amp_factor(stages: u32, idx: u32) -> f64 {
    if stages <= 1 { return 1.0; }
    let n = stages as f64;
    let i = idx as f64;
    1.0 + ((n - 1.0 - i) / (n - 1.0)) * 0.45
}

fn stage_self_error(
    base_daily_err: f64,
    inherent_pct: f64,
    avg_level: f64,
    stages: u32,
) -> f64 {
    let level_factor = if avg_level < 30.0 { 1.3 }
        else if avg_level > 70.0 { 0.9 }
        else { 1.0 };
    base_daily_err * (inherent_pct / 100.0) * level_factor
        / (stages as f64).sqrt().max(1.0)
}

#[derive(Clone)]
pub struct CascadeErrorAnalyzer;

impl CascadeErrorAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(
        &self,
        dynasty: &DynastyClepsydraConfig,
    ) -> Result<ErrorTransferAnalysis> {
        let base_err = dynasty.historical_daily_error_seconds;
        let stages = dynasty.stage_count;
        let temp = dynasty.typical_water_temp_c;

        let mut nodes = Vec::new();
        let mut cumulative_input_error = 0.0;
        let mut total_self_error = 0.0;

        for (i, cfg) in dynasty.configs.iter().enumerate() {
            let idx = i as u32;
            let avg_level = cfg.min_level + (cfg.max_level - cfg.min_level) * 0.7;

            let self_err = stage_self_error(
                base_err,
                inherent_error_pct(stages, idx),
                avg_level,
                stages,
            );

            let amp = amp_factor(stages, idx);
            let output_error = (cumulative_input_error + self_err) * amp;

            nodes.push(ErrorTransferNode {
                stage_index: idx,
                clepsydra_id: cfg.clepsydra_id.clone(),
                input_error_seconds: cumulative_input_error,
                self_error_seconds: self_err,
                output_error_seconds: output_error,
                amplification_factor: amp,
                contribution_percent: 0.0,
                water_level_cm: avg_level,
                flow_rate_mlps: cfg.standard_flow,
            });

            cumulative_input_error = output_error;
            total_self_error += self_err;
        }

        let total_error = cumulative_input_error;
        for node in nodes.iter_mut() {
            node.contribution_percent = if total_self_error > 1e-9 {
                node.self_error_seconds / total_self_error * 100.0
            } else {
                0.0
            };
        }

        let (bottleneck_idx, bottleneck_reason) = nodes
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.self_error_seconds.partial_cmp(&b.1.self_error_seconds).unwrap())
            .map(|(i, n)| {
                let reason = if n.water_level_cm < 30.0 {
                    format!(
                        "第{}级{}水位偏低，水头不足导致流量稳定性差",
                        i + 1,
                        n.clepsydra_id
                    )
                } else if n.amplification_factor > 1.3 {
                    format!(
                        "第{}级{}误差放大系数过高（{:.2}x），需增加缓冲壶",
                        i + 1,
                        n.clepsydra_id,
                        n.amplification_factor
                    )
                } else {
                    format!(
                        "第{}级{}自身固有误差最大（{:.2}s/日），建议优化孔口设计",
                        i + 1,
                        n.clepsydra_id,
                        n.self_error_seconds
                    )
                };
                (i as u32, reason)
            })
            .unwrap_or((0, "未知".to_string()));

        let mut recommendations = Vec::new();
        if stages < 4 {
            recommendations.push(format!(
                "建议增加补偿壶级数至4级，可将误差再降低约{:.0}%",
                (4.0 - stages as f64) * 15.0
            ));
        }
        recommendations.push(format!(
            "重点优化瓶颈级{}：采用漫流恒水位可减小误差约{:.0}%",
            nodes
                .get(bottleneck_idx as usize)
                .map(|n| n.clepsydra_id.clone())
                .unwrap_or_default(),
            nodes
                .get(bottleneck_idx as usize)
                .map(|n| n.contribution_percent * 0.6)
                .unwrap_or(25.0)
        ));
        recommendations.push(
            "恒温装置：将水温控制在±1°C范围内，可减少粘性引起的约8%误差".to_string(),
        );
        if temp < 18.0 {
            recommendations.push(
                "当前典型水温偏低，建议将环境温度维持在20°C左右以优化流量系数".to_string(),
            );
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ClepsydraConfig;

    fn make_dynasty(stages: u32) -> DynastyClepsydraConfig {
        let configs: Vec<ClepsydraConfig> = (0..stages)
            .map(|i| ClepsydraConfig {
                clepsydra_id: format!("K{}", i + 1),
                name: format!("壶{}", i + 1),
                max_level: 100.0,
                min_level: 40.0,
                standard_flow: 1.0 + i as f64 * 0.1,
                cross_section_area: 314.0,
                orifice_diameter: 0.5,
                flow_coefficient: 0.6,
            })
            .collect();
        DynastyClepsydraConfig {
            dynasty_id: "T".into(),
            dynasty_name: "测试".into(),
            era: "t".into(),
            clepsydra_type: "t".into(),
            stage_count: stages,
            description: "t".into(),
            historical_daily_error_seconds: 100.0,
            typical_water_temp_c: 20.0,
            material: "铜".into(),
            configs,
            reference_year: 1000,
            historical_references: vec!["t".into()],
            data_source: "t".into(),
            uncertainty_percent: 10.0,
        }
    }

    #[test]
    fn test_analyze_returns_nodes() {
        let a = CascadeErrorAnalyzer::new();
        let d = make_dynasty(3);
        let r = a.analyze(&d).unwrap();
        assert_eq!(r.nodes.len(), 3);
        assert!(r.total_error_seconds > 0.0);
    }

    #[test]
    fn test_amplification_decreases_downstream() {
        let a = CascadeErrorAnalyzer::new();
        let d = make_dynasty(4);
        let r = a.analyze(&d).unwrap();
        let amps: Vec<f64> = r.nodes.iter().map(|n| n.amplification_factor).collect();
        for w in amps.windows(2) {
            assert!(w[0] >= w[1], "amp should decrease: {:?}", amps);
        }
    }

    #[test]
    fn test_contribution_sum() {
        let a = CascadeErrorAnalyzer::new();
        let d = make_dynasty(4);
        let r = a.analyze(&d).unwrap();
        let s: f64 = r.nodes.iter().map(|n| n.contribution_percent).sum();
        assert!((s - 100.0).abs() < 1e-6, "sum should be 100, got {}", s);
    }

    #[test]
    fn test_single_stage_no_amp() {
        let a = CascadeErrorAnalyzer::new();
        let d = make_dynasty(1);
        let r = a.analyze(&d).unwrap();
        assert_eq!(r.nodes[0].amplification_factor, 1.0);
        assert_eq!(r.nodes[0].input_error_seconds, 0.0);
    }

    #[test]
    fn test_more_stages_less_per_stage_self_error() {
        let a = CascadeErrorAnalyzer::new();
        let d1 = make_dynasty(1);
        let d4 = make_dynasty(4);
        let r1 = a.analyze(&d1).unwrap();
        let r4 = a.analyze(&d4).unwrap();
        let e1: f64 = r1.nodes.iter().map(|n| n.self_error_seconds).sum();
        let e4: f64 = r4.nodes.iter().map(|n| n.self_error_seconds).sum();
        assert!(e4 < e1, "sum self err 4-stage ({}) < 1-stage ({})", e4, e1);
    }
}
