use crate::components::*;
use crate::constants::*;
use crate::metrics::ComplianceMetrics;
use crossbeam_channel::Sender;
use hecs::World;
use rand::{rng, Rng};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Generates AI events as a vector of (AIService, Usage) tuples.
///
/// # Arguments
///
/// * `count` - The number of events to generate.
///
/// # Returns
///
/// A vector containing AI events.
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

/// Applies the EU AI Act compliance rule to all relevant entities.
///
/// High-risk services with sensitive data have their compliant bit cleared.
///
/// # Arguments
///
/// * `world` - A mutable reference to the ECS world.
pub fn eu_ai_act_system(world: &mut World) {
    let high_risk_vendor_idx = 0u8; // Assume vendor at index 0 is high risk.
    for (_id, (service, usage, status)) in world.query_mut::<(&AIService, &Usage, &mut ComplianceStatus)>() {
        let is_high_risk = service.vendor_idx == high_risk_vendor_idx;
        if is_high_risk && usage.data_sensitivity > 70 {
            status.flags &= !EU_ACT_COMPLIANT;
        } else {
            status.flags |= EU_ACT_COMPLIANT;
        }
    }
}

/// Applies GDPR compliance rules to each entity.
///
/// Usage with data sensitivity below 50 is marked as GDPR compliant.
///
/// # Arguments
///
/// * `world` - A mutable reference to the ECS world.
pub fn gdpr_system(world: &mut World) {
    for (_id, (usage, status)) in world.query_mut::<(&Usage, &mut ComplianceStatus)>() {
        if usage.data_sensitivity < 50 {
            status.flags |= GDPR_COMPLIANT;
        } else {
            status.flags &= !GDPR_COMPLIANT;
        }
    }
}

/// Applies internal policy compliance rules, especially for finance.
///
/// For finance, only specific services are approved.
///
/// # Arguments
///
/// * `world` - A mutable reference to the ECS world.
pub fn internal_policy_system(world: &mut World) {
    let finance_idx = 2u8;
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

/// Assesses risk based on compliance and usage data, then attaches a RiskAssessment component.
///
/// # Arguments
///
/// * `world` - A mutable reference to the ECS world.
pub fn risk_assessment_system(world: &mut World) {
    let openai_idx = 0u8;
    let mut insertions = Vec::new();
    for (id, (service, usage, status)) in world.query_mut::<(&AIService, &Usage, &ComplianceStatus)>() {
        let mut factor_flags = 0u16;
        let mut score = 0u8;
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
        if usage.data_sensitivity > 80 {
            factor_flags |= RISK_SENSITIVE_DATA;
            score += 10;
        }
        if service.vendor_idx == openai_idx {
            factor_flags |= RISK_PUBLIC_MODEL;
            score += 5;
        }
        score = score.min(100);
        let risk = RiskAssessment {
            score,
            factor_flags,
        };
        insertions.push((id, risk));
    }
    for (id, risk) in insertions {
        let _ = world.insert_one(id, risk);
    }
}

/// Aggregates compliance metrics from all entities in the world.
///
/// # Arguments
///
/// * `world` - A reference to the ECS world.
///
/// # Returns
///
/// A `ComplianceMetrics` structure with aggregated values.
pub fn collect_metrics(world: &World) -> ComplianceMetrics {
    let mut metrics = ComplianceMetrics::default();
    for (_id, (service, usage, status, risk_opt)) in &mut world.query::<(&AIService, &Usage, &ComplianceStatus, Option<&RiskAssessment>)>() {
        metrics.total_events += 1;
        metrics.service_counts[service.name_idx as usize] += 1;
        metrics.vendor_counts[service.vendor_idx as usize] += 1;
        metrics.department_counts[usage.department_idx as usize] += 1;
        metrics.total_data_sensitivity += usage.data_sensitivity as u64;
        metrics.data_sensitivity_samples += 1;
        if status.flags & EU_ACT_COMPLIANT == 0 {
            metrics.eu_act_violations += 1;
        }
        if status.flags & GDPR_COMPLIANT == 0 {
            metrics.gdpr_violations += 1;
        }
        if status.flags & INTERNAL_POLICY_COMPLIANT == 0 {
            metrics.internal_violations += 1;
        }
        if let Some(risk) = risk_opt {
            if risk.factor_flags & RISK_EU_ACT != 0 { metrics.risk_factor_counts[0] += 1; }
            if risk.factor_flags & RISK_GDPR != 0 { metrics.risk_factor_counts[1] += 1; }
            if risk.factor_flags & RISK_INTERNAL != 0 { metrics.risk_factor_counts[2] += 1; }
            if risk.factor_flags & RISK_SENSITIVE_DATA != 0 { metrics.risk_factor_counts[3] += 1; }
            if risk.factor_flags & RISK_PUBLIC_MODEL != 0 { metrics.risk_factor_counts[4] += 1; }
            if risk.score > 70 {
                metrics.high_risk_count += 1;
            } else if risk.score > 30 {
                metrics.medium_risk_count += 1;
            } else {
                metrics.low_risk_count += 1;
            }
        }
    }
    if metrics.data_sensitivity_samples > 0 {
        metrics.avg_data_sensitivity = metrics.total_data_sensitivity as f64 / metrics.data_sensitivity_samples as f64;
    }
    metrics
}

/// Worker function that generates events, processes them, and sends metrics through a channel.
///
/// Runs continuously until a stop signal is set.
///
/// # Arguments
///
/// * `events_per_batch` - Number of events to process in each batch.
/// * `stop_signal` - Atomic flag indicating when to stop processing.
/// * `metrics_sender` - Channel sender for reporting metrics.
pub fn worker_thread(
    events_per_batch: usize,
    stop_signal: Arc<AtomicBool>,
    metrics_sender: Sender<ComplianceMetrics>,
) {
    let mut world = World::new();
    let mut thread_metrics = ComplianceMetrics::default();
    let mut batch_count = 0;
    while !stop_signal.load(Ordering::Relaxed) {
        let events = generate_ai_events(events_per_batch);
        for (ai_service, usage) in events {
            let compliance = ComplianceStatus {
                flags: EU_ACT_COMPLIANT | GDPR_COMPLIANT | INTERNAL_POLICY_COMPLIANT,
            };
            world.spawn((ai_service, usage, compliance));
        }
        eu_ai_act_system(&mut world);
        gdpr_system(&mut world);
        internal_policy_system(&mut world);
        risk_assessment_system(&mut world);
        let batch_metrics = collect_metrics(&world);
        thread_metrics.merge(&batch_metrics);
        batch_count += 1;
        if batch_count % 10 == 0 {
            if let Err(e) = metrics_sender.send(thread_metrics.clone()) {
                eprintln!("Error sending metrics: {:?}", e);
            }
            thread_metrics = ComplianceMetrics::default();
        }
        world.clear();
    }
    if thread_metrics.total_events > 0 {
        let _ = metrics_sender.send(thread_metrics);
    }
}
