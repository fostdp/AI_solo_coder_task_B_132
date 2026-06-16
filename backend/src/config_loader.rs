use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mqtt: MqttConfig,
    pub clickhouse: ClickHouseConfig,
    pub server: ServerConfig,
    pub hydraulic: HydraulicConfig,
    pub pid: PidConfig,
    pub alerts: AlertConfig,
    pub clepsydras: Vec<ClepsydraEntry>,
    pub channels: ChannelConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mqtt: MqttConfig::default(),
            clickhouse: ClickHouseConfig::default(),
            server: ServerConfig::default(),
            hydraulic: HydraulicConfig::default(),
            pid: PidConfig::default(),
            alerts: AlertConfig::default(),
            clepsydras: vec![
                ClepsydraEntry {
                    clepsydra_id: "KD1".into(), name: "天上壶".into(),
                    max_level_cm: 120.0, min_level_cm: 20.0, standard_flow_ml_s: 2.5,
                    cross_section_cm2: 78.54, orifice_diameter_cm: 0.3, flow_coefficient: 0.62,
                },
                ClepsydraEntry {
                    clepsydra_id: "KD2".into(), name: "夜漏壶".into(),
                    max_level_cm: 100.0, min_level_cm: 15.0, standard_flow_ml_s: 2.5,
                    cross_section_cm2: 78.54, orifice_diameter_cm: 0.3, flow_coefficient: 0.62,
                },
                ClepsydraEntry {
                    clepsydra_id: "KD3".into(), name: "平水壶".into(),
                    max_level_cm: 80.0, min_level_cm: 10.0, standard_flow_ml_s: 2.5,
                    cross_section_cm2: 78.54, orifice_diameter_cm: 0.3, flow_coefficient: 0.62,
                },
                ClepsydraEntry {
                    clepsydra_id: "KD4".into(), name: "万分水".into(),
                    max_level_cm: 60.0, min_level_cm: 5.0, standard_flow_ml_s: 2.5,
                    cross_section_cm2: 78.54, orifice_diameter_cm: 0.3, flow_coefficient: 0.62,
                },
            ],
            channels: ChannelConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub broker: String,
    pub port: u16,
    pub topic: String,
    pub client_id_prefix: String,
    pub keep_alive_secs: u64,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker: "localhost".into(),
            port: 1883,
            topic: "clepsydra/sensor/+".into(),
            client_id_prefix: "clepsydra-backend".into(),
            keep_alive_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickHouseConfig {
    pub url: String,
    pub database: String,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
}

impl Default for ClickHouseConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8123".into(),
            database: "clepsydra".into(),
            batch_size: 100,
            flush_interval_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub cors_origins: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 8080, cors_origins: vec!["*".into()] }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydraulicConfig {
    pub gravity_cm_s2: f64,
    pub standard_pressure_kpa: f64,
    pub min_dt_seconds: f64,
    pub altitude_m: f64,
    #[serde(default)]
    pub default_temp_c: f64,
}

impl Default for HydraulicConfig {
    fn default() -> Self {
        Self {
            gravity_cm_s2: 980.665,
            standard_pressure_kpa: 101.325,
            min_dt_seconds: 1.0,
            altitude_m: 50.0,
            default_temp_c: 20.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidConfig {
    pub base_kp: f64,
    pub base_ki: f64,
    pub base_kd: f64,
    pub kf_feedforward: f64,
    pub output_min_ml_s: f64,
    pub output_max_ml_s: f64,
    pub output_rate_limit_ml_s2: f64,
    pub integral_limit: f64,
    pub temp_coefficient_per_deg: f64,
    pub history_window: usize,
}

impl Default for PidConfig {
    fn default() -> Self {
        Self {
            base_kp: 0.5, base_ki: 0.05, base_kd: 0.1,
            kf_feedforward: 0.08,
            output_min_ml_s: -1.5, output_max_ml_s: 1.5,
            output_rate_limit_ml_s2: 0.3,
            integral_limit: 50.0,
            temp_coefficient_per_deg: 0.01,
            history_window: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub daily_error_threshold_seconds: f64,
    pub critical_error_multiplier: f64,
    pub water_temp_min_c: f64,
    pub water_temp_max_c: f64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            daily_error_threshold_seconds: 60.0,
            critical_error_multiplier: 2.0,
            water_temp_min_c: 0.0,
            water_temp_max_c: 50.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClepsydraEntry {
    pub clepsydra_id: String,
    pub name: String,
    pub max_level_cm: f64,
    pub min_level_cm: f64,
    pub standard_flow_ml_s: f64,
    pub cross_section_cm2: f64,
    pub orifice_diameter_cm: f64,
    pub flow_coefficient: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub dtu_to_simulator_buffer: usize,
    pub simulator_to_compensator_buffer: usize,
    pub compensator_to_alarm_buffer: usize,
    pub alarm_broadcast_capacity: usize,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            dtu_to_simulator_buffer: 1000,
            simulator_to_compensator_buffer: 500,
            compensator_to_alarm_buffer: 500,
            alarm_broadcast_capacity: 1000,
        }
    }
}

impl AppConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let cfg: AppConfig = serde_json::from_str(&content)?;
        Ok(cfg)
    }

    pub fn to_clepsydra_map(&self) -> HashMap<String, crate::models::ClepsydraConfig> {
        let mut map = HashMap::new();
        for entry in &self.clepsydras {
            map.insert(entry.clepsydra_id.clone(), crate::models::ClepsydraConfig {
                clepsydra_id: entry.clepsydra_id.clone(),
                name: entry.name.clone(),
                max_level: entry.max_level_cm,
                min_level: entry.min_level_cm,
                standard_flow: entry.standard_flow_ml_s,
                cross_section_area: entry.cross_section_cm2,
                orifice_diameter: entry.orifice_diameter_cm,
                flow_coefficient: entry.flow_coefficient,
            });
        }
        map
    }
}
