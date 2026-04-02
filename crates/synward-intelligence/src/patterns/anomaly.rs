//! Anomaly Detection - Detect unusual code patterns

use crate::patterns::CodeFeatures;
use serde::{Deserialize, Serialize};

/// An detected anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Anomaly type
    pub anomaly_type: AnomalyType,

    /// Severity (1-5)
    pub severity: u8,

    /// Description
    pub description: String,

    /// Location (if known)
    pub location: Option<String>,

    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Type of anomaly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Unusually high complexity
    HighComplexity,

    /// Excessive nesting
    DeepNesting,

    /// Too many unwrap/expect
    ExcessiveUnwrap,

    /// Missing error handling
    MissingErrorHandler,

    /// Unusual pattern (not in any cluster)
    UnusualPattern,

    /// Code quality issue
    QualityIssue,
}

/// Detect anomalies in code features
pub struct AnomalyDetector {
    /// Thresholds
    complexity_threshold: u32,
    nesting_threshold: u32,
    unwrap_threshold: u32,
}

impl AnomalyDetector {
    /// Create a new detector
    pub fn new() -> Self {
        Self {
            complexity_threshold: 15,
            nesting_threshold: 4,
            unwrap_threshold: 5,
        }
    }

    /// Detect anomalies in features
    pub fn detect(&self, features: &CodeFeatures) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        // Check complexity
        if features.cyclomatic_complexity > self.complexity_threshold {
            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::HighComplexity,
                severity: 3,
                description: format!(
                    "High cyclomatic complexity: {} (threshold: {})",
                    features.cyclomatic_complexity, self.complexity_threshold
                ),
                location: None,
                suggestion: Some("Consider breaking into smaller functions".to_string()),
            });
        }

        // Check nesting
        if features.max_nesting_depth > self.nesting_threshold {
            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::DeepNesting,
                severity: 2,
                description: format!(
                    "Deep nesting: {} levels (threshold: {})",
                    features.max_nesting_depth, self.nesting_threshold
                ),
                location: None,
                suggestion: Some("Extract nested logic into separate functions".to_string()),
            });
        }

        // Check unwrap usage
        if features.unwrap_count > self.unwrap_threshold {
            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::ExcessiveUnwrap,
                severity: 4,
                description: format!(
                    "Excessive unwrap() calls: {} (threshold: {})",
                    features.unwrap_count, self.unwrap_threshold
                ),
                location: None,
                suggestion: Some("Use proper error handling with Result/Option".to_string()),
            });
        }

        anomalies
    }

    /// Detect from raw code
    pub fn detect_from_code(&self, code: &str, language: &str) -> Vec<Anomaly> {
        let extractor = crate::patterns::FeatureExtractor::new();
        let features = extractor.extract(code, language);
        self.detect(&features)
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_features(complexity: u32, nesting: u32, unwrap: u32) -> CodeFeatures {
        CodeFeatures {
            line_count: 100,
            function_count: 5,
            avg_function_length: 20.0,
            cyclomatic_complexity: complexity,
            max_nesting_depth: nesting,
            error_handler_count: 0,
            unwrap_count: unwrap,
            comment_count: 10,
            todo_count: 2,
            vector: vec![],
        }
    }

    #[test]
    fn test_detect_no_anomalies() {
        let detector = AnomalyDetector::new();
        let features = make_features(10, 3, 2);
        let anomalies = detector.detect(&features);

        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_detect_high_complexity() {
        let detector = AnomalyDetector::new();
        let features = make_features(20, 3, 2); // complexity > 15
        let anomalies = detector.detect(&features);

        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, AnomalyType::HighComplexity);
    }

    #[test]
    fn test_detect_deep_nesting() {
        let detector = AnomalyDetector::new();
        let features = make_features(10, 5, 2); // nesting > 4
        let anomalies = detector.detect(&features);

        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, AnomalyType::DeepNesting);
    }

    #[test]
    fn test_detect_excessive_unwrap() {
        let detector = AnomalyDetector::new();
        let features = make_features(10, 3, 10); // unwrap > 5
        let anomalies = detector.detect(&features);

        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].anomaly_type, AnomalyType::ExcessiveUnwrap);
        assert_eq!(anomalies[0].severity, 4);
    }

    #[test]
    fn test_detect_multiple_anomalies() {
        let detector = AnomalyDetector::new();
        let features = make_features(20, 5, 10); // all three
        let anomalies = detector.detect(&features);

        assert_eq!(anomalies.len(), 3);
    }

    #[test]
    fn test_detect_from_code() {
        let detector = AnomalyDetector::new();
        let code = r#"
fn main() {
    let x = a.unwrap();
    let y = b.unwrap();
    let z = c.unwrap();
    let w = d.unwrap();
    let v = e.unwrap();
    let u = f.unwrap();
}
"#;
        let anomalies = detector.detect_from_code(code, "rust");

        assert!(!anomalies.is_empty());
        assert!(anomalies.iter().any(|a| a.anomaly_type == AnomalyType::ExcessiveUnwrap));
    }
}
