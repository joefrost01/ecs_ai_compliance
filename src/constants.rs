/// Service names for AI events.
pub const SERVICE_NAMES: [&str; 5] = ["ChatGPT", "Claude", "Gemini", "Copilot", "Stable Diffusion"];

/// Department names used in usage events.
pub const DEPARTMENT_NAMES: [&str; 5] = ["Engineering", "Marketing", "Finance", "HR", "Legal"];

/// Bit flags for compliance statuses.
pub const EU_ACT_COMPLIANT: u8 = 0b00000001;
pub const GDPR_COMPLIANT: u8 = 0b00000010;
pub const INTERNAL_POLICY_COMPLIANT: u8 = 0b00000100;

/// Bit flags for risk factors.
pub const RISK_EU_ACT: u16 = 0b0000000000000001;
pub const RISK_GDPR: u16 = 0b0000000000000010;
pub const RISK_INTERNAL: u16 = 0b0000000000000100;
pub const RISK_SENSITIVE_DATA: u16 = 0b0000000000001000;
pub const RISK_PUBLIC_MODEL: u16 = 0b0000000000010000;

/// Mapping of risk factor flags to descriptive names.
pub const RISK_FACTOR_NAMES: [(u16, &str); 5] = [
    (RISK_EU_ACT, "EU AI Act non-compliance"),
    (RISK_GDPR, "GDPR non-compliance"),
    (RISK_INTERNAL, "Internal policy violation"),
    (RISK_SENSITIVE_DATA, "High sensitivity data"),
    (RISK_PUBLIC_MODEL, "Public model usage"),
];

/// Tab names for the dashboard UI.
pub const TAB_NAMES: [&str; 4] = ["Overview", "Services", "Compliance", "Risk"];
