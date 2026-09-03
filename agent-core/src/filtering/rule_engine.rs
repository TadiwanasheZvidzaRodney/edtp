use crate::models::TelemetryEvent;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Rule {
    pub name: String,
    pub description: String,
    pub target_image: String,
    pub susp_cmd: String,
}

pub struct RuleEngine {
    rules: Vec<Rule>,
}

impl RuleEngine {
    pub fn new() -> anyhow::Result<Self> {
        // Mocking a loaded YAML rule file
        let rules = vec![
            Rule {
                name: "Suspicious Execution".to_string(),
                description: "Detects potentially malicious hidden commands".to_string(),
                target_image: "powershell.exe".to_string(),
                susp_cmd: "-hidden".to_string(),
            },
            Rule {
                name: "Network Discovery".to_string(),
                description: "Detects network scanning/discovery commands".to_string(),
                target_image: "svchost.exe".to_string(),
                susp_cmd: "netsvcs".to_string(),
            }
        ];
        
        Ok(Self { rules })
    }
    
    pub fn evaluate(&self, event: &TelemetryEvent) -> Option<Rule> {
        match event {
            TelemetryEvent::ProcessCreate(pc) => {
                for rule in &self.rules {
                    if pc.image_file_name.to_lowercase().contains(&rule.target_image) && 
                       pc.command_line.to_lowercase().contains(&rule.susp_cmd) {
                        return Some(rule.clone());
                    }
                }
            },
            _ => {}
        }
        None
    }
}
