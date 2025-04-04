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

// Dashboard commands
pub enum DashboardCommand {
    UpdateMetrics(ComplianceMetrics),
}

// Dashboard tabs
pub enum DashboardTab {
    Overview,
    Services,
    Compliance,
    Risk,
}

impl DashboardTab {
    fn index(&self) -> usize {
        match self {
            DashboardTab::Overview => 0,
            DashboardTab::Services => 1,
            DashboardTab::Compliance => 2,
            DashboardTab::Risk => 3,
        }
    }
}

pub struct Dashboard {
    pub metrics: ComplianceMetrics,
    pub active_tab: DashboardTab,
    pub should_quit: bool,
}

impl Dashboard {
    pub fn new() -> Self {
        Dashboard {
            metrics: ComplianceMetrics::default(),
            active_tab: DashboardTab::Overview,
            should_quit: false,
        }
    }

    pub fn handle_command(&mut self, cmd: DashboardCommand) {
        match cmd {
            DashboardCommand::UpdateMetrics(metrics) => self.metrics = metrics,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('1') => self.active_tab = DashboardTab::Overview,
            KeyCode::Char('2') => self.active_tab = DashboardTab::Services,
            KeyCode::Char('3') => self.active_tab = DashboardTab::Compliance,
            KeyCode::Char('4') => self.active_tab = DashboardTab::Risk,
            KeyCode::Tab => {
                // Cycle through tabs
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

    pub fn render<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        terminal.draw(|f| {
            let size = f.size();

            // Create main layout with tabs and content area
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            // Render tab bar - fix: borrow TAB_NAMES with &
            render_tabs(f, chunks[0], &TAB_NAMES, self.active_tab.index());

            // Render content based on active tab
            match self.active_tab {
                DashboardTab::Overview => self.render_overview_tab(f, chunks[1]),
                DashboardTab::Services => self.render_services_tab(f, chunks[1]),
                DashboardTab::Compliance => self.render_compliance_tab(f, chunks[1]),
                DashboardTab::Risk => self.render_risk_tab(f, chunks[1]),
            }
        })?;

        Ok(())
    }

    fn render_overview_tab<B: Backend>(&self, f: &mut tui::Frame<B>, area: Rect) {
        // Split the area into sections
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

        // Top section - compliance gauge
        render_compliance_gauge(f, chunks[0], &self.metrics);

        // Middle section - stats and service chart
        let middle_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[1]);

        render_stats(f, middle_chunks[0], &self.metrics);
        render_service_chart(f, middle_chunks[1], &self.metrics);

        // Bottom section - processing rate chart
        render_rate_chart(f, chunks[2], &self.metrics);
    }

    fn render_services_tab<B: Backend>(&self, f: &mut tui::Frame<B>, area: Rect) {
        // Split the area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ]
                    .as_ref(),
            )
            .split(area);

        // Top section - service chart
        render_service_chart(f, chunks[0], &self.metrics);

        // Bottom section - department chart
        render_department_chart(f, chunks[1], &self.metrics);
    }

    fn render_compliance_tab<B: Backend>(&self, f: &mut tui::Frame<B>, area: Rect) {
        // Split the area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(70),
                ]
                    .as_ref(),
            )
            .split(area);

        // Top section - compliance gauge
        render_compliance_gauge(f, chunks[0], &self.metrics);

        // Bottom section - violation bar chart instead of line chart
        render_violation_chart(f, chunks[1], &self.metrics);
    }

    fn render_risk_tab<B: Backend>(&self, f: &mut tui::Frame<B>, area: Rect) {
        // Split the area into sections
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

        // Top section - risk statistics
        render_stats(f, chunks[0], &self.metrics);

        // Middle section - risk factors bar chart
        render_risk_factors(f, chunks[1], &self.metrics);

        // Bottom section - risk distribution (using the previously unused method)
        render_risk_distribution(f, chunks[2], &self.metrics);
    }
}