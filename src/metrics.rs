use std::time::Duration;

// Enhanced metrics for reporting and visualization
#[derive(Default, Clone)]
pub struct ComplianceMetrics {
    pub total_events: usize,
    pub eu_act_violations: usize,
    pub gdpr_violations: usize,
    pub internal_violations: usize,
    pub high_risk_count: usize,
    pub medium_risk_count: usize,
    pub low_risk_count: usize,
    pub service_counts: [usize; 5],
    pub vendor_counts: [usize; 5],
    pub department_counts: [usize; 5],
    pub risk_factor_counts: [usize; 5],
    pub avg_data_sensitivity: f64,
    pub total_data_sensitivity: u64,
    pub data_sensitivity_samples: usize,
    pub processing_rate: f64,
    pub historical_rates: Vec<f64>, // For time-series visualization
    pub historical_violations: Vec<(usize, usize, usize)>, // EU, GDPR, Internal
}

impl ComplianceMetrics {
    pub fn merge(&mut self, other: &ComplianceMetrics) {
        self.total_events += other.total_events;
        self.eu_act_violations += other.eu_act_violations;
        self.gdpr_violations += other.gdpr_violations;
        self.internal_violations += other.internal_violations;
        self.high_risk_count += other.high_risk_count;
        self.medium_risk_count += other.medium_risk_count;
        self.low_risk_count += other.low_risk_count;

        for i in 0..5 {
            self.service_counts[i] += other.service_counts[i];
            self.vendor_counts[i] += other.vendor_counts[i];
            self.department_counts[i] += other.department_counts[i];
            self.risk_factor_counts[i] += other.risk_factor_counts[i];
        }

        self.total_data_sensitivity += other.total_data_sensitivity;
        self.data_sensitivity_samples += other.data_sensitivity_samples;

        if self.data_sensitivity_samples > 0 {
            self.avg_data_sensitivity = self.total_data_sensitivity as f64 / self.data_sensitivity_samples as f64;
        }
    }

    pub fn update_historical_data(&mut self, processed_since_last: usize, elapsed: Duration) {
        // Update processing rate
        self.processing_rate = processed_since_last as f64 / elapsed.as_secs_f64();

        // Add to historical data (limit to last 30 data points)
        self.historical_rates.push(self.processing_rate);
        if self.historical_rates.len() > 30 {
            self.historical_rates.remove(0);
        }

        // Add violation data
        self.historical_violations.push((
            self.eu_act_violations,
            self.gdpr_violations,
            self.internal_violations
        ));
        if self.historical_violations.len() > 30 {
            self.historical_violations.remove(0);
        }
    }

    pub fn compliance_percentage(&self) -> f64 {
        if self.total_events == 0 {
            return 100.0;
        }

        let violation_count = self.eu_act_violations + self.gdpr_violations + self.internal_violations;
        100.0 * (1.0 - (violation_count as f64 / (self.total_events as f64 * 3.0)))
    }

    pub fn risk_distribution(&self) -> [f64; 3] {
        if self.total_events == 0 {
            return [0.0, 0.0, 0.0];
        }

        [
            self.high_risk_count as f64 / self.total_events as f64 * 100.0,
            self.medium_risk_count as f64 / self.total_events as f64 * 100.0,
            self.low_risk_count as f64 / self.total_events as f64 * 100.0,
        ]
    }
}