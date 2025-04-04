use clap::Parser;

/// Command line arguments for the application.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Number of AI events to process per second.
    #[arg(short, long, default_value_t = 100000)]
    pub rate: u32,

    /// Reporting interval in seconds.
    #[arg(short, long, default_value_t = 5)]
    pub interval: u64,

    /// Number of worker threads (defaults to number of logical cores).
    #[arg(short, long)]
    pub threads: Option<usize>,
}

/// Component representing an AI service event.
/// Uses indices into static arrays for name and vendor to reduce memory footprint.
#[derive(Clone, Copy)]
pub struct AIService {
    pub name_idx: u8,
    pub vendor_idx: u8,
}

/// Component representing the usage details of an AI event.
#[derive(Clone, Copy)]
pub struct Usage {
    pub department_idx: u8,
    pub data_sensitivity: u8, // Scale from 0 to 100.
}

/// Component representing compliance status using bit flags.
#[derive(Clone, Copy)]
pub struct ComplianceStatus {
    pub flags: u8,
}

/// Component representing a risk assessment for an AI event.
#[derive(Clone, Copy)]
pub struct RiskAssessment {
    pub score: u8,      // Risk score on a 0-100 scale.
    pub factor_flags: u16, // Bit flags indicating which risk factors apply.
}
