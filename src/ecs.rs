use crate::components::*;
use crate::constants::*;
use crate::metrics::ComplianceMetrics;
use crossbeam_channel::Sender;
use hecs::World;
use rand::{rng, Rng};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Event generation function optimized for performance
pub fn generate_ai_events(count: usize) -> Vec<(AIService, Usage)> {
    let mut events = Vec::with_capacity(count);
    let mut rng = rng();

    for _ in 0..count {
        let ai_service = AIService {
            name_idx: rng.random_range(0..5) as u8,
            vendor_idx: rng.random_range(0..5) as u8,
        };

        let usage = Usage {
            department_idx: rng.random_range(0..5) as u8,
            data_sensitivity: rng.random_range(0..100),
        };

        events.push((ai_service, usage));
    }

    events
}

// Compliance rules systems - optimized for performance
pub fn eu_ai_act_system(world: &mut World) {
    // Pre-calculate high-risk vendors (OpenAI is index 0)
    let high_risk_vendor_idx = 0u8;

    for (_id, (service, usage, status)) in world.query_mut::<(&AIService, &Usage, &mut ComplianceStatus)>() {
        // Example rule: High-risk AI services with sensitive data need special handling
        let is_high_risk = service.vendor_idx == high_risk_vendor_idx;

        if is_high_risk && usage.data_sensitivity > 70 {
            status.flags &= !EU_ACT_COMPLIANT; // Clear the compliant bit
        } else {
            status.flags |= EU_ACT_COMPLIANT;  // Set the compliant bit
        }
    }
}

pub fn gdpr_system(world: &mut World) {
    for (_id, (usage, status)) in world.query_mut::<(&Usage, &mut ComplianceStatus)>() {
        // Example rule: Any usage with high data sensitivity requires GDPR compliance
        if usage.data_sensitivity < 50 {
            status.flags |= GDPR_COMPLIANT;
        } else {
            status.flags &= !GDPR_COMPLIANT;
        }
    }
}

pub fn internal_policy_system(world: &mut World) {
    // Finance department index is 2
    let finance_idx = 2u8;

    // Approved services for finance: Claude (1) and Copilot (3)
    let approved_services: [u8; 2] = [1, 3];

    for (_id, (service, usage, status)) in world.query_mut::<(&AIService, &Usage, &mut ComplianceStatus)>() {
        if usage.department_idx == finance_idx {
            if approved_services.contains(&service.name_idx) {
                status.flags |= INTERNAL_POLICY_COMPLIANT;
            } else {
                status.flags &= !INTERNAL_POLICY_COMPLIANT;
            }
        } else {
            status.flags |= INTERNAL_POLICY_COMPLIANT;
        }
    }
}

pub fn risk_assessment_system(world: &mut World) {
    // OpenAI vendor index
    let openai_idx = 0u8;
    // Temporary storage for risk insertions
    let mut insertions = Vec::new();

    for (id, (service, usage, status)) in world.query_mut::<(&AIService, &Usage, &ComplianceStatus)>() {
        let mut factor_flags = 0u16;
        let mut score = 0u8;

        // Check compliance status using bit flags
        if status.flags & EU_ACT_COMPLIANT == 0 {
            factor_flags |= RISK_EU_ACT;
            score += 40;
        }

        if status.flags & GDPR_COMPLIANT == 0 {
            factor_flags |= RISK_GDPR;
            score += 30;
        }

        if status.flags & INTERNAL_POLICY_COMPLIANT == 0 {
            factor_flags |= RISK_INTERNAL;
            score += 20;
        }

        // Add risk based on data sensitivity
        if usage.data_sensitivity > 80 {
            factor_flags |= RISK_SENSITIVE_DATA;
            score += 10;
        }

        // Add vendor-specific risks
        if service.vendor_idx == openai_idx {
            factor_flags |= RISK_PUBLIC_MODEL;
            score += 5;
        }

        // Cap at 100
        score = score.min(100);

        // Prepare risk assessment component
        let risk = RiskAssessment {
            score,
            factor_flags,
        };

        // Save the insertion to apply later
        insertions.push((id, risk));
    }

    // Now perform insertions after releasing the mutable borrow from the query
    for (id, risk) in insertions {
        // Entity might have been removed in another system, ignore error if so.
        let _ = world.insert_one(id, risk);
    }
}

pub fn collect_metrics(world: &World) -> ComplianceMetrics {
    let mut metrics = ComplianceMetrics::default();

    for (_id, (service, usage, status, risk_opt)) in &mut world.query::<(&AIService, &Usage, &ComplianceStatus, Option<&RiskAssessment>)>() {
        metrics.total_events += 1;

        // Track service usage
        metrics.service_counts[service.name_idx as usize] += 1;

        // Track vendor usage
        metrics.vendor_counts[service.vendor_idx as usize] += 1;

        // Track department usage
        metrics.department_counts[usage.department_idx as usize] += 1;

        // Track data sensitivity
        metrics.total_data_sensitivity += usage.data_sensitivity as u64;
        metrics.data_sensitivity_samples += 1;

        // Track compliance violations
        if status.flags & EU_ACT_COMPLIANT == 0 {
            metrics.eu_act_violations += 1;
        }

        if status.flags & GDPR_COMPLIANT == 0 {
            metrics.gdpr_violations += 1;
        }

        if status.flags & INTERNAL_POLICY_COMPLIANT == 0 {
            metrics.internal_violations += 1;
        }

        // Track risk assessment
        if let Some(risk) = risk_opt {
            // Count risk factors
            if risk.factor_flags & RISK_EU_ACT != 0 { metrics.risk_factor_counts[0] += 1; }
            if risk.factor_flags & RISK_GDPR != 0 { metrics.risk_factor_counts[1] += 1; }
            if risk.factor_flags & RISK_INTERNAL != 0 { metrics.risk_factor_counts[2] += 1; }
            if risk.factor_flags & RISK_SENSITIVE_DATA != 0 { metrics.risk_factor_counts[3] += 1; }
            if risk.factor_flags & RISK_PUBLIC_MODEL != 0 { metrics.risk_factor_counts[4] += 1; }

            // Track risk levels
            if risk.score > 70 {
                metrics.high_risk_count += 1;
            } else if risk.score > 30 {
                metrics.medium_risk_count += 1;
            } else {
                metrics.low_risk_count += 1;
            }
        }
    }

    // Calculate average data sensitivity
    if metrics.data_sensitivity_samples > 0 {
        metrics.avg_data_sensitivity = metrics.total_data_sensitivity as f64 / metrics.data_sensitivity_samples as f64;
    }

    metrics
}

// Worker function for multithreaded processing
pub fn worker_thread(
    events_per_batch: usize,
    stop_signal: Arc<AtomicBool>,
    metrics_sender: Sender<ComplianceMetrics>
) {
    let mut world = World::new();
    let mut thread_metrics = ComplianceMetrics::default();
    let mut batch_count = 0;

    while !stop_signal.load(Ordering::Relaxed) {
        // Generate a batch of events
        let events = generate_ai_events(events_per_batch);

        // Process events
        for (ai_service, usage) in events {
            let compliance = ComplianceStatus {
                flags: EU_ACT_COMPLIANT | GDPR_COMPLIANT | INTERNAL_POLICY_COMPLIANT,
            };

            world.spawn((ai_service, usage, compliance));
        }

        // Run compliance systems
        eu_ai_act_system(&mut world);
        gdpr_system(&mut world);
        internal_policy_system(&mut world);
        risk_assessment_system(&mut world);

        // Collect metrics for this batch
        let batch_metrics = collect_metrics(&world);
        thread_metrics.merge(&batch_metrics);

        // Report metrics periodically
        batch_count += 1;
        if batch_count % 10 == 0 {
            metrics_sender.send(thread_metrics.clone()).unwrap_or_default();
            thread_metrics = ComplianceMetrics::default();
        }

        // Clear world for next batch
        world.clear();
    }

    // Send any remaining metrics
    if thread_metrics.total_events > 0 {
        metrics_sender.send(thread_metrics).unwrap_or_default();
    }
}