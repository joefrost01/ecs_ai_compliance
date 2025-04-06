use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use minijinja::Environment;
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tracing::{info, error};

use crate::constants;
use crate::metrics::ComplianceMetrics;

/// Data structures for templates and API
#[derive(Clone, Serialize)]
struct DashboardData {
    total_events: usize,
    eu_act_violations: usize,
    gdpr_violations: usize,
    internal_violations: usize,
    compliance_percentage: f64,
    processing_rate: f64,
    risk_distribution: RiskDistribution,
    service_usage: Vec<UsageData>,
    department_usage: Vec<UsageData>,
    processing_rate_history: Vec<f64>,
    // Enhanced dashboard data
    service_compliance: Vec<ServiceComplianceData>,
    department_analysis: Vec<DepartmentAnalysis>,
}

#[derive(Clone, Serialize)]
struct RiskDistribution {
    low: f64,
    medium: f64,
    high: f64,
}

#[derive(Clone, Serialize)]
struct UsageData {
    name: String,
    percentage: f64,
}

/// Data structure for service compliance data.
#[derive(Clone, Serialize)]
struct ServiceComplianceData {
    name: String,
    usage_percentage: f64,
    // As we aren’t tracking per-service violations in our metrics,
    // we use the global compliance percentage for each service.
    eu_act_compliance: f64,
    gdpr_compliance: f64,
    internal_compliance: f64,
    total_usage: usize,
}

/// Data structure for department analysis.
#[derive(Clone, Serialize)]
struct DepartmentAnalysis {
    name: String,
    usage_percentage: f64,
    // Similarly, we use the global compliance percentage as a placeholder.
    compliance_rate: f64,
    // Determine risk level based on the overall compliance rate.
    risk_level: String,
    // No random violations—this field is left blank; real data can be sourced if available.
    common_violations: String,
}

/// Application state
struct AppState {
    metrics: Arc<RwLock<ComplianceMetrics>>,
    jinja_env: Environment<'static>,
}

/// Custom filter to format numbers with commas
fn format_number(value: i64) -> String {
    let mut result = String::new();
    let value_str = value.to_string();
    let len = value_str.len();

    for (i, c) in value_str.chars().enumerate() {
        result.push(c);
        if (len - i - 1) % 3 == 0 && i < len - 1 {
            result.push(',');
        }
    }

    result
}

/// Generate service compliance data directly from the actual metrics.
fn generate_service_compliance_data(metrics: &ComplianceMetrics) -> Vec<ServiceComplianceData> {
    let mut service_compliance = Vec::new();
    // Use the global compliance percentage as a placeholder for service-level compliance.
    let global_compliance = metrics.compliance_percentage();
    for i in 0..constants::SERVICE_NAMES.len() {
        let total_service_usage = metrics.service_counts[i];
        service_compliance.push(ServiceComplianceData {
            name: constants::SERVICE_NAMES[i].to_string(),
            usage_percentage: if metrics.total_events > 0 {
                (total_service_usage as f64 / metrics.total_events as f64) * 100.0
            } else { 0.0 },
            eu_act_compliance: global_compliance,
            gdpr_compliance: global_compliance,
            internal_compliance: global_compliance,
            total_usage: total_service_usage,
        });
    }
    service_compliance
}

/// Generate department analysis data directly from the actual metrics.
fn generate_department_analysis(metrics: &ComplianceMetrics) -> Vec<DepartmentAnalysis> {
    let mut department_analysis = Vec::new();
    // Use the global compliance percentage as a placeholder for department-level compliance.
    let global_compliance = metrics.compliance_percentage();
    let risk_level = if global_compliance >= 90.0 {
        "Low"
    } else if global_compliance >= 75.0 {
        "Medium"
    } else {
        "High"
    }
        .to_string();

    for i in 0..constants::DEPARTMENT_NAMES.len() {
        let total_dept_usage = metrics.department_counts[i];
        department_analysis.push(DepartmentAnalysis {
            name: constants::DEPARTMENT_NAMES[i].to_string(),
            usage_percentage: if metrics.total_events > 0 {
                (total_dept_usage as f64 / metrics.total_events as f64) * 100.0
            } else { 0.0 },
            compliance_rate: global_compliance,
            risk_level: risk_level.clone(),
            common_violations: String::new(),
        });
    }
    department_analysis
}

/// Setup minijinja environment.
fn setup_jinja() -> Environment<'static> {
    let mut env = Environment::new();

    // Add templates directly.
    env.add_template("index.html", include_str!("../templates/index.html"))
        .expect("Failed to add index template");
    env.add_template("partials/stats.html", include_str!("../templates/partials/stats.html"))
        .expect("Failed to add stats template");

    // Add custom filters.
    env.add_filter("format_number", format_number);

    env
}

/// Main index handler.
async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let metrics = state.metrics.read().await;

    // Convert metrics to template context.
    let risk_dist = metrics.risk_distribution();
    let service_usage: Vec<UsageData> = (0..constants::SERVICE_NAMES.len())
        .map(|i| {
            UsageData {
                name: constants::SERVICE_NAMES[i].to_string(),
                percentage: if metrics.total_events > 0 {
                    (metrics.service_counts[i] as f64 / metrics.total_events as f64) * 100.0
                } else {
                    0.0
                },
            }
        })
        .collect();

    let department_usage: Vec<UsageData> = (0..constants::DEPARTMENT_NAMES.len())
        .map(|i| {
            UsageData {
                name: constants::DEPARTMENT_NAMES[i].to_string(),
                percentage: if metrics.total_events > 0 {
                    (metrics.department_counts[i] as f64 / metrics.total_events as f64) * 100.0
                } else {
                    0.0
                },
            }
        })
        .collect();

    // Generate enhanced data for the dashboard using actual metrics.
    let service_compliance = generate_service_compliance_data(&metrics);
    let department_analysis = generate_department_analysis(&metrics);

    // Render the template.
    match state.jinja_env.get_template("index.html") {
        Ok(template) => {
            #[derive(Serialize)]
            struct TemplateContext<'a> {
                total_events: usize,
                eu_act_violations: usize,
                gdpr_violations: usize,
                internal_violations: usize,
                compliance_percentage: f64,
                processing_rate: f64,
                processing_rate_history: &'a Vec<f64>,
                risk_distribution: RiskDistribution,
                service_usage: Vec<UsageData>,
                department_usage: Vec<UsageData>,
                service_compliance: Vec<ServiceComplianceData>,
                department_analysis: Vec<DepartmentAnalysis>,
            }

            let context = TemplateContext {
                total_events: metrics.total_events,
                eu_act_violations: metrics.eu_act_violations,
                gdpr_violations: metrics.gdpr_violations,
                internal_violations: metrics.internal_violations,
                compliance_percentage: metrics.compliance_percentage(),
                processing_rate: metrics.processing_rate,
                processing_rate_history: &metrics.historical_rates,
                risk_distribution: RiskDistribution {
                    low: risk_dist[2],
                    medium: risk_dist[1],
                    high: risk_dist[0],
                },
                service_usage,
                department_usage,
                service_compliance,
                department_analysis,
            };

            match template.render(context) {
                Ok(html) => Html(html).into_response(),
                Err(e) => {
                    error!("Template rendering error: {}", e);
                    Html(format!("<h1>Template Error</h1><pre>{}</pre>", e)).into_response()
                }
            }
        }
        Err(e) => {
            error!("Template loading error: {}", e);
            Html(format!("<h1>Template Not Found</h1><pre>{}</pre>", e)).into_response()
        }
    }
}

/// API endpoint for dashboard data.
async fn dashboard_data(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let metrics = state.metrics.read().await;

    // Generate enhanced data for the API endpoint using actual metrics.
    let service_compliance = generate_service_compliance_data(&metrics);
    let department_analysis = generate_department_analysis(&metrics);

    let risk_dist = metrics.risk_distribution();
    let dashboard_data = DashboardData {
        total_events: metrics.total_events,
        eu_act_violations: metrics.eu_act_violations,
        gdpr_violations: metrics.gdpr_violations,
        internal_violations: metrics.internal_violations,
        compliance_percentage: metrics.compliance_percentage(),
        processing_rate: metrics.processing_rate / 1_000_000.0, // Converted to millions.
        risk_distribution: RiskDistribution {
            low: risk_dist[2],
            medium: risk_dist[1],
            high: risk_dist[0],
        },
        service_usage: (0..constants::SERVICE_NAMES.len())
            .map(|i| {
                UsageData {
                    name: constants::SERVICE_NAMES[i].to_string(),
                    percentage: if metrics.total_events > 0 {
                        (metrics.service_counts[i] as f64 / metrics.total_events as f64) * 100.0
                    } else {
                        0.0
                    },
                }
            })
            .collect(),
        department_usage: (0..constants::DEPARTMENT_NAMES.len())
            .map(|i| {
                UsageData {
                    name: constants::DEPARTMENT_NAMES[i].to_string(),
                    percentage: if metrics.total_events > 0 {
                        (metrics.department_counts[i] as f64 / metrics.total_events as f64) * 100.0
                    } else {
                        0.0
                    },
                }
            })
            .collect(),
        processing_rate_history: metrics
            .historical_rates
            .iter()
            .map(|&rate| rate / 1_000_000.0)
            .collect(),
        service_compliance,
        department_analysis,
    };

    Json(dashboard_data)
}

/// HTMX endpoint for stats updates.
async fn stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let metrics = state.metrics.read().await;

    match state.jinja_env.get_template("partials/stats.html") {
        Ok(template) => {
            #[derive(Serialize)]
            struct StatsContext {
                total_events: usize,
                eu_act_violations: usize,
                gdpr_violations: usize,
                internal_violations: usize,
                processing_rate: f64,
            }
            let context = StatsContext {
                total_events: metrics.total_events,
                eu_act_violations: metrics.eu_act_violations,
                gdpr_violations: metrics.gdpr_violations,
                internal_violations: metrics.internal_violations,
                processing_rate: metrics.processing_rate,
            };

            match template.render(context) {
                Ok(html) => Html(html).into_response(),
                Err(e) => {
                    error!("Stats template rendering error: {}", e);
                    Html(format!("<div>Template Error: {}</div>", e)).into_response()
                }
            }
        }
        Err(e) => {
            error!("Stats template loading error: {}", e);
            Html(format!("<div>Template Not Found: {}</div>", e)).into_response()
        }
    }
}

/// Main function to start the web server.
pub async fn start_server(
    metrics: Arc<RwLock<ComplianceMetrics>>,
    shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
) {
    let jinja_env = setup_jinja();

    let state = Arc::new(AppState {
        metrics,
        jinja_env,
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/api/dashboard", get(dashboard_data))
        .route("/api/stats", get(stats))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("Web dashboard available at http://localhost:8080");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");
    info!("Listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .expect("Server error");

    info!("Server shutdown complete");
}
