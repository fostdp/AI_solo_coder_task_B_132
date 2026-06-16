use std::sync::Arc;
use anyhow::Result;

use crate::config_loader::AppConfig;
use crate::hydraulic::HydraulicModel;
use crate::models::{ClepsydraConfig, DynastyClepsydraConfig, VirtualOperationRequest,
    VirtualOperationResult, DynastyComparison, CrossEraComparison, ErrorTransferAnalysis};
use crate::era_precision_comparator::EraPrecisionComparator;
use crate::chronometry_comparator::ChronometryComparator;
use crate::cascade_error_analyzer::CascadeErrorAnalyzer;
use crate::vr_clepsydra::VrClepsydraEngine;

pub struct HydraulicComputePool {
    pub comparator: Arc<EraPrecisionComparator>,
    pub chronometry: ChronometryComparator,
    pub analyzer: CascadeErrorAnalyzer,
    pub vr_engine: Arc<VrClepsydraEngine>,
    pub hydraulic: Arc<HydraulicModel>,
    pub config: Arc<AppConfig>,
}

impl HydraulicComputePool {
    pub fn new(
        config: Arc<AppConfig>,
        hydraulic: Arc<HydraulicModel>,
    ) -> Self {
        let comparator = Arc::new(EraPrecisionComparator::new(config.clone(), hydraulic.clone()));
        let chronometry = ChronometryComparator::new();
        let analyzer = CascadeErrorAnalyzer::new();
        let vr_engine = Arc::new(VrClepsydraEngine::new(config.clone(), hydraulic.clone()));
        Self { comparator, chronometry, analyzer, vr_engine, hydraulic, config }
    }

    pub fn batch_dynasty_errors(
        &self,
        dynasties: &[DynastyClepsydraConfig],
    ) -> Vec<(String, f64)> {
        use rayon::prelude::*;
        let comp = Arc::clone(&self.comparator);
        dynasties
            .par_iter()
            .map(move |d| {
                let err = comp.calc_dynasty_daily_error(d);
                (d.dynasty_id.clone(), err)
            })
            .collect()
    }

    pub fn batch_flow_compute(
        &self,
        batch: &[(f64, ClepsydraConfig, f64)],
    ) -> Vec<f64> {
        use rayon::prelude::*;
        let hyd = Arc::clone(&self.hydraulic);
        batch
            .par_iter()
            .map(move |(level, cfg, temp)| hyd.calculate_theoretical_flow(*level, cfg, *temp))
            .collect()
    }

    pub async fn compare_dynasties_async(
        self: Arc<Self>,
        left: DynastyClepsydraConfig,
        right: DynastyClepsydraConfig,
    ) -> Result<DynastyComparison> {
        let comp = Arc::clone(&self.comparator);
        tokio::task::spawn_blocking(move || {
            comp.compare_dynasties(&left, &right)
        })
        .await
        .map_err(|e| anyhow::anyhow!("join error: {}", e))?
    }

    pub async fn cross_era_async(
        self: Arc<Self>,
        dynasties: Vec<DynastyClepsydraConfig>,
        modern_pieces: Vec<crate::models::ModernTimepiece>,
    ) -> Result<CrossEraComparison> {
        let comp = Arc::clone(&self.comparator);
        let chrono = self.chronometry.clone();
        tokio::task::spawn_blocking(move || {
            chrono.cross_era_comparison(&dynasties, &modern_pieces, |d| {
                comp.calc_dynasty_daily_error(d)
            })
        })
        .await
        .map_err(|e| anyhow::anyhow!("join error: {}", e))?
    }

    pub async fn error_transfer_async(
        self: Arc<Self>,
        dynasty: DynastyClepsydraConfig,
    ) -> Result<ErrorTransferAnalysis> {
        let analyzer = self.analyzer.clone();
        tokio::task::spawn_blocking(move || analyzer.analyze(&dynasty))
            .await
            .map_err(|e| anyhow::anyhow!("join error: {}", e))?
    }

    pub async fn virtual_operate_async(
        self: Arc<Self>,
        req: VirtualOperationRequest,
    ) -> Result<VirtualOperationResult> {
        let eng = Arc::clone(&self.vr_engine);
        tokio::task::spawn_blocking(move || eng.run_virtual_operation(&req))
            .await
            .map_err(|e| anyhow::anyhow!("join error: {}", e))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_loader::{HydraulicConfig, AppConfig};

    #[test]
    fn test_pool_created_ok() {
        let cfg = Arc::new(AppConfig::default());
        let hyd = Arc::new(HydraulicModel::new());
        let pool = HydraulicComputePool::new(cfg, hyd);
        assert!(pool.comparator.config.hydraulic.altitude_m >= 0.0);
    }
}
