use crate::constants::TAB_NAMES;
use crate::metrics::ComplianceMetrics;
use crate::ui::widgets::*;
use crossterm::event::{KeyCode, KeyEvent};
use std::io;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal,
};

/// Commands that can be sent to update the dashboard state.
pub enum DashboardCommand {
    /// Update the displayed metrics.
    UpdateMetrics(ComplianceMetrics),
}

/// Enumeration of dashboard tabs.
#[derive(Debug)]
pub enum DashboardTab {
    Overview,
    Services,
    Compliance,
    Risk,
}

impl DashboardTab {
    /// Returns the index of the current tab.
    pub fn index(&self) -> usize {
        match self {
            DashboardTab::Overview => 0,
            DashboardTab::Services => 1,
            DashboardTab::Compliance => 2,
            DashboardTab::Risk => 3,
        }
    }
}

/// The main dashboard structure holding metrics and UI state.
pub struct Dashboard {
    pub metrics: ComplianceMetrics,
    pub active_tab: DashboardTab,
    pub should_quit: bool,
}

impl Dashboard {
    /// Creates a new instance of the Dashboard.
    pub fn new() -> Self {
        Dashboard {
            metrics: ComplianceMetrics::default(),
            active_tab: DashboardTab::Overview,
            should_quit: false,
        }
    }

    /// Handles an incoming command to update the dashboard.
    pub fn handle_command(&mut self, cmd: DashboardCommand) {
        match cmd {
            DashboardCommand::UpdateMetrics(metrics) => self.metrics = metrics,
        }
    }

    /// Processes a key event to update the UI (tab switching, quitting, etc.).
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('1') => self.active_tab = DashboardTab::Overview,
            KeyCode::Char('2') => self.active_tab = DashboardTab::Services,
            KeyCode::Char('3') => self.active_tab = DashboardTab::Compliance,
            KeyCode::Char('4') => self.active_tab = DashboardTab::Risk,
            KeyCode::Tab => {
                // Cycle through tabs in order.
                self.active_tab = match self.active_tab {
                    DashboardTab::Overview => DashboardTab::Services,
                    DashboardTab::Services => DashboardTab::Compliance,
                    DashboardTab::Compliance => DashboardTab::Risk,
                    DashboardTab::Risk => DashboardTab::Overview,
                };
            }
            _ => {}
        }
    }

    /// Renders the dashboard UI.
    pub fn render<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        terminal.draw(|f| {
            let size = f.size();
            // Layout: first row for tabs, remaining for content.
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            // Render the tab bar.
            render_tabs(f, chunks[0], &TAB_NAMES, self.active_tab.index());

            // Render content based on the active tab.
            match self.active_tab {
                DashboardTab::Overview => self.render_overview_tab(f, chunks[1]),
                DashboardTab::Services => self.render_services_tab(f, chunks[1]),
                DashboardTab::Compliance => self.render_compliance_tab(f, chunks[1]),
                DashboardTab::Risk => self.render_risk_tab(f, chunks[1]),
            }
        })?;
        Ok(())
    }

    /// Renders the overview tab: gauge, stats, and charts.
    fn render_overview_tab<B: Backend>(&self, f: &mut tui::Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(40),
                    Constraint::Percentage(40),
                ]
                .as_ref(),
            )
            .split(area);

        // Top: overall compliance gauge.
        render_compliance_gauge(f, chunks[0], &self.metrics);

        // Middle: stats and service chart.
        let middle_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[1]);

        render_stats(f, middle_chunks[0], &self.metrics);
        render_service_chart(f, middle_chunks[1], &self.metrics);

        // Bottom: processing rate history.
        render_rate_chart(f, chunks[2], &self.metrics);
    }

    /// Renders the services tab with charts for service and department usage.
    fn render_services_tab<B: Backend>(&self, f: &mut tui::Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        render_service_chart(f, chunks[0], &self.metrics);
        render_department_chart(f, chunks[1], &self.metrics);
    }

    /// Renders the compliance tab with gauge and violations chart.
    fn render_compliance_tab<B: Backend>(&self, f: &mut tui::Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .split(area);

        render_compliance_gauge(f, chunks[0], &self.metrics);
        render_violation_chart(f, chunks[1], &self.metrics);
    }

    /// Renders the risk tab with stats and risk charts.
    fn render_risk_tab<B: Backend>(&self, f: &mut tui::Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ]
                .as_ref(),
            )
            .split(area);

        render_stats(f, chunks[0], &self.metrics);
        render_risk_factors(f, chunks[1], &self.metrics);
        render_risk_distribution(f, chunks[2], &self.metrics);
    }
}
