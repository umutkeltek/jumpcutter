use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum ProcessEvent {
    Log {
        message: String,
    },
    SilenceDetected {
        count: usize,
        intervals: Vec<[f64; 2]>,
    },
    Progress {
        percent: f64,
    },
    Complete {
        output_path: String,
    },
    Error {
        message: String,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessOptions {
    pub input_path: String,
    pub output_path: String,
    pub noise_db: f64,
    pub min_silence_duration: f64,
    pub failure_tolerance: f64,
    pub edge_padding: f64,
    pub mode: String,
    pub silence_speed: f64,
    pub min_loud_duration: f64,
    pub codec: Option<String>,
    pub bitrate: Option<String>,
}
