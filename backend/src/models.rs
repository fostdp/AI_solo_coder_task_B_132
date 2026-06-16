use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub clepsydra_id: String,
    pub water_level: f64,
    pub flow_rate: f64,
    pub water_temp: f64,
    pub humidity: f64,
    pub quality: f64,
    #[serde(default = "default_pressure")]
    pub pressure: f64,
}

fn default_pressure() -> f64 {
    101.325
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClepsydraConfig {
    pub clepsydra_id: String,
    pub name: String,
    pub max_level: f64,
    pub min_level: f64,
    pub standard_flow: f64,
    pub cross_section_area: f64,
    pub orifice_diameter: f64,
    pub flow_coefficient: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydraulicMetrics {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub clepsydra_id: String,
    pub theoretical_flow: f64,
    pub actual_flow: f64,
    pub flow_error: f64,
    pub evaporation_rate: f64,
    pub daily_error_seconds: f64,
    pub compensation_flow: f64,
    pub pid_kp: f64,
    pub pid_ki: f64,
    pub pid_kd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    WaterLevelHigh,
    WaterLevelLow,
    DailyErrorExceed,
    TempAbnormal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub clepsydra_id: String,
    pub alert_type: AlertType,
    pub alert_level: AlertLevel,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidState {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub kf: f64,
    pub integral: f64,
    pub prev_error: f64,
    pub output_min: f64,
    pub output_max: f64,
    pub prev_temp: f64,
    pub temp_history: [f64; 5],
    pub temp_history_idx: usize,
    pub integral_limit: f64,
    pub last_output: f64,
    pub output_rate_limit: f64,
}

impl PidState {
    pub fn new(kp: f64, ki: f64, kd: f64, output_min: f64, output_max: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            kf: 0.0,
            integral: 0.0,
            prev_error: 0.0,
            output_min,
            output_max,
            prev_temp: 20.0,
            temp_history: [20.0; 5],
            temp_history_idx: 0,
            integral_limit: 50.0,
            last_output: 0.0,
            output_rate_limit: f64::INFINITY,
        }
    }

    pub fn with_feedforward(mut self, kf: f64) -> Self {
        self.kf = kf;
        self
    }

    pub fn with_rate_limit(mut self, rate_limit: f64) -> Self {
        self.output_rate_limit = rate_limit;
        self
    }

    pub fn compute(
        &mut self,
        setpoint: f64,
        actual: f64,
        water_temp: f64,
        dt: f64,
    ) -> f64 {
        let error = setpoint - actual;

        self.temp_history[self.temp_history_idx] = water_temp;
        self.temp_history_idx = (self.temp_history_idx + 1) % self.temp_history.len();

        let temp_avg: f64 = self.temp_history.iter().sum::<f64>() / self.temp_history.len() as f64;
        let temp_rate = (water_temp - temp_avg) / (dt * self.temp_history.len() as f64);
        self.prev_temp = water_temp;

        let feedforward = self.kf * temp_rate;

        self.integral += error * dt;
        self.integral = self.integral.clamp(-self.integral_limit, self.integral_limit);

        let derivative = (error - self.prev_error) / dt;
        self.prev_error = error;

        let pid_output = self.kp * error + self.ki * self.integral + self.kd * derivative;

        let raw_output = pid_output + feedforward;

        let rate_limited_output = if self.output_rate_limit.is_finite() {
            let max_change = self.output_rate_limit * dt;
            (raw_output - self.last_output).clamp(-max_change, max_change) + self.last_output
        } else {
            raw_output
        };

        let clamped_output = rate_limited_output.clamp(self.output_min, self.output_max);

        if clamped_output != raw_output && self.ki.abs() > 1e-9 {
            if clamped_output > 0.0 && error > 0.0 {
                self.integral -= error * dt;
            } else if clamped_output < 0.0 && error < 0.0 {
                self.integral -= error * dt;
            }
        }

        self.last_output = clamped_output;
        clamped_output
    }

    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.prev_error = 0.0;
        self.last_output = 0.0;
        self.prev_temp = 20.0;
        self.temp_history = [20.0; 5];
        self.temp_history_idx = 0;
    }
}

impl AlertType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertType::WaterLevelHigh => "WATER_LEVEL_HIGH",
            AlertType::WaterLevelLow => "WATER_LEVEL_LOW",
            AlertType::DailyErrorExceed => "DAILY_ERROR_EXCEED",
            AlertType::TempAbnormal => "TEMP_ABNORMAL",
        }
    }
}

impl AlertLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertLevel::Info => "INFO",
            AlertLevel::Warning => "WARNING",
            AlertLevel::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynastyClepsydraConfig {
    pub dynasty_id: String,
    pub dynasty_name: String,
    pub era: String,
    pub clepsydra_type: String,
    pub stage_count: u32,
    pub description: String,
    pub historical_daily_error_seconds: f64,
    pub typical_water_temp_c: f64,
    pub material: String,
    pub configs: Vec<ClepsydraConfig>,
    pub reference_year: i32,
    pub historical_references: Vec<String>,
    pub data_source: String,
    pub uncertainty_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModernTimepiece {
    pub piece_id: String,
    pub name: String,
    pub category: String,
    pub daily_error_seconds: f64,
    pub yearly_error_seconds: f64,
    pub technology: String,
    pub invention_year: u32,
    pub description: String,
    pub accuracy_class: String,
    pub standard_reference: String,
    pub iso_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyComparisonPoint {
    pub label: String,
    pub category: String,
    pub daily_error_seconds: f64,
    pub yearly_error_minutes: f64,
    pub color_hex: String,
    pub era: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTransferNode {
    pub stage_index: u32,
    pub clepsydra_id: String,
    pub input_error_seconds: f64,
    pub self_error_seconds: f64,
    pub output_error_seconds: f64,
    pub amplification_factor: f64,
    pub contribution_percent: f64,
    pub water_level_cm: f64,
    pub flow_rate_mlps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTransferAnalysis {
    pub total_error_seconds: f64,
    pub nodes: Vec<ErrorTransferNode>,
    pub bottleneck_stage: u32,
    pub bottleneck_reason: String,
    pub recommendations: Vec<String>,
    pub compensation_potential_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualOperationRequest {
    pub clepsydra_id: String,
    pub target_water_level_cm: f64,
    pub water_temp_c: Option<f64>,
    pub simulate_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualOperationResult {
    pub clepsydra_id: String,
    pub initial_level_cm: f64,
    pub final_level_cm: f64,
    pub initial_error_seconds: f64,
    pub final_error_seconds: f64,
    pub time_elapsed_simulated: u32,
    pub level_history: Vec<(f64, f64)>,
    pub error_history: Vec<(f64, f64)>,
    pub flow_history: Vec<(f64, f64)>,
    pub observations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynastyComparison {
    pub left_dynasty: DynastyClepsydraConfig,
    pub right_dynasty: DynastyClepsydraConfig,
    pub left_daily_error_seconds: f64,
    pub right_daily_error_seconds: f64,
    pub error_ratio: f64,
    pub winner: String,
    pub key_differences: Vec<String>,
    pub flow_comparison: Vec<FlowComparisonPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowComparisonPoint {
    pub stage: String,
    pub left_flow_mlps: f64,
    pub right_flow_mlps: f64,
    pub left_level_cm: f64,
    pub right_level_cm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossEraComparison {
    pub ancient_devices: Vec<AccuracyComparisonPoint>,
    pub modern_devices: Vec<AccuracyComparisonPoint>,
    pub best_ancient: AccuracyComparisonPoint,
    pub best_modern: AccuracyComparisonPoint,
    pub improvement_factor: f64,
    pub timeline_data: Vec<TimelineAccuracy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineAccuracy {
    pub year: i32,
    pub label: String,
    pub daily_error_seconds: f64,
    pub category: String,
}
