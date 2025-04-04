use crate::constants::*;
use crate::metrics::ComplianceMetrics;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{
        Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, GraphType,
        Paragraph, Tabs,
    },
    Frame,
};
use tui::layout::{Constraint, Direction, Layout};

// Helper function to create a block with title
pub fn create_block(title: &str) -> Block {
    Block::default().borders(Borders::ALL).title(Span::styled(
        title,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
}

// Render compliance overview
pub fn render_compliance_gauge<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    metrics: &ComplianceMetrics,
) {
    let compliance_pct = metrics.compliance_percentage();

    let gauge_color = if compliance_pct > 90.0 {
        Color::Green
    } else if compliance_pct > 70.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let gauge = Gauge::default()
        .block(create_block("Overall Compliance"))
        .gauge_style(Style::default().fg(gauge_color).bg(Color::Black))
        .percent(compliance_pct as u16)
        .label(format!("{:.1}%", compliance_pct));

    f.render_widget(gauge, area);
}

// Render processing statistics
pub fn render_stats<B: Backend>(f: &mut Frame<B>, area: Rect, metrics: &ComplianceMetrics) {
    let text = vec![
        Spans::from(Span::raw(format!("Total Events: {}", metrics.total_events))),
        Spans::from(Span::raw(format!(
            "Processing Rate: {:.1} events/s",
            metrics.processing_rate
        ))),
        Spans::from(Span::raw("")),
        Spans::from(Span::raw(format!(
            "EU AI Act Violations: {} ({:.1}%)",
            metrics.eu_act_violations,
            if metrics.total_events > 0 {
                (metrics.eu_act_violations as f64 / metrics.total_events as f64) * 100.0
            } else {
                0.0
            }
        ))),
        Spans::from(Span::raw(format!(
            "GDPR Violations: {} ({:.1}%)",
            metrics.gdpr_violations,
            if metrics.total_events > 0 {
                (metrics.gdpr_violations as f64 / metrics.total_events as f64) * 100.0
            } else {
                0.0
            }
        ))),
        Spans::from(Span::raw(format!(
            "Internal Policy Violations: {} ({:.1}%)",
            metrics.internal_violations,
            if metrics.total_events > 0 {
                (metrics.internal_violations as f64 / metrics.total_events as f64) * 100.0
            } else {
                0.0
            }
        ))),
        Spans::from(Span::raw("")),
        Spans::from(Span::raw(format!(
            "High Risk Events: {} ({:.1}%)",
            metrics.high_risk_count,
            if metrics.total_events > 0 {
                (metrics.high_risk_count as f64 / metrics.total_events as f64) * 100.0
            } else {
                0.0
            }
        ))),
        Spans::from(Span::raw(format!(
            "Medium Risk Events: {} ({:.1}%)",
            metrics.medium_risk_count,
            if metrics.total_events > 0 {
                (metrics.medium_risk_count as f64 / metrics.total_events as f64) * 100.0
            } else {
                0.0
            }
        ))),
        Spans::from(Span::raw(format!(
            "Low Risk Events: {} ({:.1}%)",
            metrics.low_risk_count,
            if metrics.total_events > 0 {
                (metrics.low_risk_count as f64 / metrics.total_events as f64) * 100.0
            } else {
                0.0
            }
        ))),
    ];

    let paragraph = Paragraph::new(text)
        .block(create_block("Processing Statistics"))
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

// Render service usage chart
pub fn render_service_chart<B: Backend>(f: &mut Frame<B>, area: Rect, metrics: &ComplianceMetrics) {
    let mut data = Vec::new();
    let total = metrics.total_events.max(1) as f64;

    for i in 0..5 {
        if metrics.service_counts[i] > 0 {
            let percentage = (metrics.service_counts[i] as f64 / total) * 100.0;
            data.push((SERVICE_NAMES[i], percentage as u64));
        }
    }

    // Sort by usage (descending)
    data.sort_by(|a, b| b.1.cmp(&a.1));

    let barchart = BarChart::default()
        .block(create_block("Service Usage"))
        .data(&data)
        .bar_width(9)
        .bar_style(Style::default().fg(Color::Yellow))
        .value_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(barchart, area);
}

// Render department usage chart
pub fn render_department_chart<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    metrics: &ComplianceMetrics,
) {
    let mut data = Vec::new();
    let total = metrics.total_events.max(1) as f64;

    for i in 0..5 {
        if metrics.department_counts[i] > 0 {
            let percentage = (metrics.department_counts[i] as f64 / total) * 100.0;
            data.push((DEPARTMENT_NAMES[i], percentage as u64));
        }
    }

    // Sort by usage (descending)
    data.sort_by(|a, b| b.1.cmp(&a.1));

    let barchart = BarChart::default()
        .block(create_block("Department Usage"))
        .data(&data)
        .bar_width(9)
        .bar_style(Style::default().fg(Color::Green))
        .value_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(barchart, area);
}

// Render processing rate history chart
pub fn render_rate_chart<B: Backend>(f: &mut Frame<B>, area: Rect, metrics: &ComplianceMetrics) {
    if metrics.historical_rates.is_empty() {
        let message = Paragraph::new("Waiting for data...")
            .block(create_block("Processing Rate History"))
            .style(Style::default().fg(Color::Gray));

        f.render_widget(message, area);
        return;
    }

    // Create data points
    let data: Vec<(f64, f64)> = metrics
        .historical_rates
        .iter()
        .enumerate()
        .map(|(i, &rate)| (i as f64, rate))
        .collect();

    // Find max rate for scaling - fix: specify type for max as f64
    let max_rate = metrics
        .historical_rates
        .iter()
        .fold(0.0, |max: f64, &rate| max.max(rate));

    let datasets = vec![
        Dataset::default()
            .name("Events/second")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .graph_type(GraphType::Line)
            .data(&data),
    ];

    // Create strings for labels outside of chart creation to extend their lifetime
    let mid_label = format!("{:.0}", max_rate / 2.0);
    let max_label = format!("{:.0}", max_rate);

    let chart = Chart::new(datasets)
        .block(create_block("Processing Rate History"))
        .x_axis(
            Axis::default()
                .title(Span::styled("Time", Style::default().fg(Color::White)))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, data.len() as f64])
                .labels(vec![
                    Span::styled("Start", Style::default().fg(Color::White)),
                    Span::styled("Now", Style::default().fg(Color::White)),
                ]),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("Events/s", Style::default().fg(Color::White)))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, max_rate * 1.1]) // Add 10% for better visualization
                .labels(vec![
                    Span::styled("0", Style::default().fg(Color::White)),
                    Span::styled(&mid_label, Style::default().fg(Color::White)),
                    Span::styled(&max_label, Style::default().fg(Color::White)),
                ]),
        );

    f.render_widget(chart, area);
}

// Render risk factor breakdown
pub fn render_risk_factors<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    metrics: &ComplianceMetrics,
) {
    if metrics.total_events == 0 {
        let message = Paragraph::new("Waiting for data...")
            .block(create_block("Risk Factors"))
            .style(Style::default().fg(Color::Gray));

        f.render_widget(message, area);
        return;
    }

    // Prepare data for bar chart
    let mut risk_data = Vec::new();

    for (i, (_, _name)) in RISK_FACTOR_NAMES.iter().enumerate() {
        if metrics.risk_factor_counts[i] > 0 {
            let short_name = match i {
                0 => "EU Act",
                1 => "GDPR",
                2 => "Internal",
                3 => "Sensitive",
                4 => "Public Model",
                _ => "Other",
            };
            risk_data.push((short_name, metrics.risk_factor_counts[i] as u64));
        }
    }

    // Sort by count (descending)
    risk_data.sort_by(|a, b| b.1.cmp(&a.1));

    // If we have data, render a bar chart
    if !risk_data.is_empty() {
        let barchart = BarChart::default()
            .block(create_block("Risk Factors"))
            .data(&risk_data)
            .bar_width(9)
            .bar_style(Style::default().fg(Color::Yellow))
            .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            .bar_gap(2);

        f.render_widget(barchart, area);
    } else {
        // Fallback if no risk factors detected
        let message = Paragraph::new("No risk factors detected")
            .block(create_block("Risk Factors"))
            .style(Style::default().fg(Color::Gray));

        f.render_widget(message, area);
    }
}

// Render violation history chart
pub fn render_violation_chart<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    metrics: &ComplianceMetrics,
) {
    if metrics.total_events == 0 {
        let message = Paragraph::new("Waiting for data...")
            .block(create_block("Compliance Violations"))
            .style(Style::default().fg(Color::Gray));

        f.render_widget(message, area);
        return;
    }

    // Create data for the bar chart
    let violations = vec![
        ("EU AI Act", metrics.eu_act_violations as u64),
        ("GDPR", metrics.gdpr_violations as u64),
        ("Internal", metrics.internal_violations as u64),
    ];

    let barchart = BarChart::default()
        .block(create_block("Compliance Violations"))
        .data(&violations)
        .bar_width(10)
        .bar_style(Style::default().fg(Color::Red))
        .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .bar_gap(3);

    f.render_widget(barchart, area);
}

// Render tab selector
pub fn render_tabs<B: Backend>(f: &mut Frame<B>, area: Rect, titles: &[&str], active_tab: usize) {
    let tabs = Tabs::new(titles.iter().map(|t| Spans::from(*t)).collect())
        .block(Block::default().borders(Borders::BOTTOM))
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .select(active_tab);

    f.render_widget(tabs, area);
}

pub fn render_risk_distribution<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    metrics: &ComplianceMetrics,
) {
    if metrics.total_events == 0 {
        let message = Paragraph::new("Waiting for data...")
            .block(create_block("Risk Distribution"))
            .style(Style::default().fg(Color::Gray));

        f.render_widget(message, area);
        return;
    }

    // Get risk distribution data
    let risk_dist = metrics.risk_distribution();

    // Calculate counts
    let high_count = (risk_dist[0] * metrics.total_events as f64 / 100.0) as u64;
    let medium_count = (risk_dist[1] * metrics.total_events as f64 / 100.0) as u64;
    let low_count = (risk_dist[2] * metrics.total_events as f64 / 100.0) as u64;

    // We'll use one chart per bar with its own style and uniform scale
    let block = create_block("Risk Distribution");
    f.render_widget(block.clone(), area);

    // Get inner area for charts
    let inner = block.inner(area);

    // Split into 3 columns
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ].as_ref())
        .split(inner);

    // High risk bar (red)
    let high_data = vec![("High", high_count)];
    let high_chart = BarChart::default()
        .data(&high_data)
        .bar_width(15)
        .bar_style(Style::default().fg(Color::Red))
        .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .max(high_count.max(medium_count).max(low_count)) // Set same max for all charts
        .bar_gap(0);

    f.render_widget(high_chart, chunks[0]);

    // Medium risk bar (yellow)
    let medium_data = vec![("Medium", medium_count)];
    let medium_chart = BarChart::default()
        .data(&medium_data)
        .bar_width(15)
        .bar_style(Style::default().fg(Color::Yellow))
        .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .max(high_count.max(medium_count).max(low_count)) // Set same max for all charts
        .bar_gap(0);

    f.render_widget(medium_chart, chunks[1]);

    // Low risk bar (green)
    let low_data = vec![("Low", low_count)];
    let low_chart = BarChart::default()
        .data(&low_data)
        .bar_width(15)
        .bar_style(Style::default().fg(Color::Green))
        .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .max(high_count.max(medium_count).max(low_count)) // Set same max for all charts
        .bar_gap(0);

    f.render_widget(low_chart, chunks[2]);
}