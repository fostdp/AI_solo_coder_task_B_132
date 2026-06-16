use anyhow::Result;

use crate::models::{
    AccuracyComparisonPoint, CrossEraComparison, DynastyClepsydraConfig, ModernTimepiece,
    TimelineAccuracy,
};

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

#[derive(Clone)]
pub struct ChronometryComparator;

impl ChronometryComparator {
    pub fn new() -> Self {
        Self
    }

    pub fn cross_era_comparison<F>(
        &self,
        dynasties: &[DynastyClepsydraConfig],
        modern_pieces: &[ModernTimepiece],
        calc_dynasty_error: F,
    ) -> Result<CrossEraComparison>
    where
        F: Fn(&DynastyClepsydraConfig) -> f64,
    {
        let ancient_devices: Vec<AccuracyComparisonPoint> = dynasties
            .iter()
            .map(|d| {
                let err = calc_dynasty_error(d);
                AccuracyComparisonPoint {
                    label: format!(
                        "{}·{}",
                        d.dynasty_name,
                        d.clepsydra_type.split('（').next().unwrap_or("")
                    ),
                    category: d.clepsydra_type.clone(),
                    daily_error_seconds: err,
                    yearly_error_minutes: err * 365.0 / 60.0,
                    color_hex: dynasty_color(&d.dynasty_id).to_string(),
                    era: "古代".to_string(),
                }
            })
            .collect();

        let modern_devices: Vec<AccuracyComparisonPoint> = modern_pieces
            .iter()
            .map(|m| AccuracyComparisonPoint {
                label: m.name.clone(),
                category: m.category.clone(),
                daily_error_seconds: m.daily_error_seconds,
                yearly_error_minutes: m.yearly_error_seconds / 60.0,
                color_hex: modern_color(&m.category).to_string(),
                era: "现代".to_string(),
            })
            .collect();

        let best_ancient = ancient_devices
            .iter()
            .min_by(|a, b| a.daily_error_seconds.partial_cmp(&b.daily_error_seconds).unwrap())
            .cloned()
            .unwrap_or_else(|| ancient_devices[0].clone());

        let best_modern = modern_devices
            .iter()
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dynasty(id: &str, name: &str, err: f64) -> DynastyClepsydraConfig {
        use crate::models::ClepsydraConfig;
        DynastyClepsydraConfig {
            dynasty_id: id.to_string(),
            dynasty_name: name.to_string(),
            era: "古代".to_string(),
            clepsydra_type: "漏壶（测试）".to_string(),
            stage_count: 2,
            description: "t".to_string(),
            historical_daily_error_seconds: err,
            typical_water_temp_c: 20.0,
            material: "铜".to_string(),
            configs: vec![ClepsydraConfig {
                clepsydra_id: "K1".into(), name: "壶".into(),
                max_level: 100.0, min_level: 20.0, standard_flow: 1.0,
                cross_section_area: 314.0, orifice_diameter: 0.5, flow_coefficient: 0.6,
            }],
            reference_year: 1000,
            historical_references: vec!["t".into()],
            data_source: "t".into(),
            uncertainty_percent: 10.0,
        }
    }

    fn make_modern(id: &str, name: &str, err: f64, cat: &str) -> ModernTimepiece {
        ModernTimepiece {
            piece_id: id.into(), name: name.into(), category: cat.into(),
            daily_error_seconds: err, yearly_error_seconds: err * 365.0,
            technology: "t".into(), invention_year: 2000,
            description: "t".into(), accuracy_class: "t".into(),
            standard_reference: "ISO t".into(), iso_class: Some("t".into()),
        }
    }

    #[test]
    fn test_cross_era_returns_both_eras() {
        let comp = ChronometryComparator::new();
        let dynasties = vec![
            make_dynasty("SONG_LIANHUA", "宋莲花漏", 50.0),
            make_dynasty("TANG_JINGLU", "唐吕才漏", 120.0),
        ];
        let modern = vec![
            make_modern("Q", "石英表", 0.5, "电子"),
            make_modern("A", "原子钟", 1e-6, "原子"),
        ];
        let result = comp.cross_era_comparison(&dynasties, &modern, |d| d.historical_daily_error_seconds).unwrap();
        assert_eq!(result.ancient_devices.len(), 2);
        assert_eq!(result.modern_devices.len(), 2);
        assert!(result.improvement_factor > 1.0);
        assert!(result.timeline_data.len() >= 10);
        assert_eq!(result.best_modern.label, "原子钟");
    }

    #[test]
    fn test_colors_are_assigned() {
        let comp = ChronometryComparator::new();
        let dynasties = vec![make_dynasty("SONG_LIANHUA", "宋", 50.0)];
        let modern = vec![make_modern("A", "铯钟", 1e-6, "原子")];
        let r = comp.cross_era_comparison(&dynasties, &modern, |d| d.historical_daily_error_seconds).unwrap();
        assert_eq!(r.ancient_devices[0].color_hex, "#FF6B6B");
        assert_eq!(r.modern_devices[0].color_hex, "#E74C3C");
    }
}
