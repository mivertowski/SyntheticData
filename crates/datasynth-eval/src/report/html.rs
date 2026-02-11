//! HTML report generation.
//!
//! Generates human-readable HTML reports with charts and visualizations.

use super::{EvaluationReport, IssueCategory, IssueSeverity, ReportGenerator};
use crate::error::EvalResult;

/// HTML report generator.
pub struct HtmlReportGenerator {
    /// Include inline CSS.
    include_css: bool,
    /// Include charts.
    include_charts: bool,
}

impl HtmlReportGenerator {
    /// Create a new generator.
    pub fn new() -> Self {
        Self {
            include_css: true,
            include_charts: true,
        }
    }

    /// Set whether to include CSS.
    pub fn with_css(mut self, include: bool) -> Self {
        self.include_css = include;
        self
    }

    /// Set whether to include charts.
    pub fn with_charts(mut self, include: bool) -> Self {
        self.include_charts = include;
        self
    }

    /// Generate CSS styles.
    fn generate_css(&self) -> String {
        r#"
        <style>
            :root {
                --pass-color: #22c55e;
                --fail-color: #ef4444;
                --warning-color: #f59e0b;
                --info-color: #3b82f6;
                --bg-color: #f8fafc;
                --card-bg: #ffffff;
                --text-color: #1e293b;
                --border-color: #e2e8f0;
            }
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: var(--bg-color);
                color: var(--text-color);
                line-height: 1.6;
                margin: 0;
                padding: 20px;
            }
            .container {
                max-width: 1200px;
                margin: 0 auto;
            }
            h1, h2, h3 {
                margin-top: 0;
            }
            .header {
                text-align: center;
                margin-bottom: 30px;
            }
            .status-badge {
                display: inline-block;
                padding: 8px 24px;
                border-radius: 20px;
                font-weight: bold;
                font-size: 1.2em;
            }
            .status-pass {
                background: var(--pass-color);
                color: white;
            }
            .status-fail {
                background: var(--fail-color);
                color: white;
            }
            .card {
                background: var(--card-bg);
                border-radius: 8px;
                box-shadow: 0 1px 3px rgba(0,0,0,0.1);
                padding: 20px;
                margin-bottom: 20px;
            }
            .card-title {
                font-size: 1.2em;
                font-weight: 600;
                margin-bottom: 15px;
                padding-bottom: 10px;
                border-bottom: 1px solid var(--border-color);
            }
            .metric-grid {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                gap: 15px;
            }
            .metric {
                padding: 15px;
                background: var(--bg-color);
                border-radius: 6px;
            }
            .metric-label {
                font-size: 0.85em;
                color: #64748b;
                margin-bottom: 5px;
            }
            .metric-value {
                font-size: 1.5em;
                font-weight: 600;
            }
            .metric-pass { color: var(--pass-color); }
            .metric-fail { color: var(--fail-color); }
            .metric-warning { color: var(--warning-color); }
            .issues-list {
                list-style: none;
                padding: 0;
                margin: 0;
            }
            .issue-item {
                padding: 12px 15px;
                border-left: 4px solid;
                margin-bottom: 10px;
                background: var(--bg-color);
                border-radius: 0 6px 6px 0;
            }
            .issue-critical { border-color: var(--fail-color); }
            .issue-warning { border-color: var(--warning-color); }
            .issue-info { border-color: var(--info-color); }
            .issue-category {
                font-size: 0.75em;
                text-transform: uppercase;
                color: #64748b;
                margin-bottom: 5px;
            }
            table {
                width: 100%;
                border-collapse: collapse;
            }
            th, td {
                padding: 10px;
                text-align: left;
                border-bottom: 1px solid var(--border-color);
            }
            th {
                background: var(--bg-color);
                font-weight: 600;
            }
            .score-bar {
                height: 8px;
                background: var(--border-color);
                border-radius: 4px;
                overflow: hidden;
            }
            .score-fill {
                height: 100%;
                border-radius: 4px;
            }
            .score-good { background: var(--pass-color); }
            .score-medium { background: var(--warning-color); }
            .score-bad { background: var(--fail-color); }
            .metadata {
                font-size: 0.85em;
                color: #64748b;
            }
        </style>
        "#
        .to_string()
    }

    /// Generate summary section.
    fn generate_summary(&self, report: &EvaluationReport) -> String {
        let status_class = if report.passes {
            "status-pass"
        } else {
            "status-fail"
        };
        let status_text = if report.passes { "PASSED" } else { "FAILED" };

        format!(
            r#"
            <div class="header">
                <h1>Synthetic Data Evaluation Report</h1>
                <div class="status-badge {status_class}">{status_text}</div>
                <p class="metadata">
                    Generated: {} | Records: {} | Duration: {}ms
                </p>
            </div>

            <div class="card">
                <div class="card-title">Overall Score</div>
                <div class="metric-grid">
                    <div class="metric">
                        <div class="metric-label">Overall Score</div>
                        <div class="metric-value {}">{:.1}%</div>
                        <div class="score-bar">
                            <div class="score-fill {}" style="width: {:.1}%"></div>
                        </div>
                    </div>
                    <div class="metric">
                        <div class="metric-label">Issues Found</div>
                        <div class="metric-value">{}</div>
                    </div>
                    <div class="metric">
                        <div class="metric-label">Critical Issues</div>
                        <div class="metric-value {}">{}</div>
                    </div>
                </div>
            </div>
            "#,
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            report.metadata.records_evaluated,
            report.metadata.duration_ms,
            self.score_class(report.overall_score),
            report.overall_score * 100.0,
            self.score_bar_class(report.overall_score),
            report.overall_score * 100.0,
            report.all_issues.len(),
            if report.critical_issues().is_empty() {
                "metric-pass"
            } else {
                "metric-fail"
            },
            report.critical_issues().len()
        )
    }

    /// Generate statistical section.
    fn generate_statistical_section(&self, report: &EvaluationReport) -> String {
        let Some(ref stat) = report.statistical else {
            return String::new();
        };

        let mut metrics_html = String::new();

        if let Some(ref benford) = stat.benford {
            metrics_html.push_str(&format!(
                r#"
                <div class="metric">
                    <div class="metric-label">Benford's Law p-value</div>
                    <div class="metric-value {}">{:.4}</div>
                </div>
                <div class="metric">
                    <div class="metric-label">Benford MAD</div>
                    <div class="metric-value {}">{:.4}</div>
                </div>
                <div class="metric">
                    <div class="metric-label">Conformity Level</div>
                    <div class="metric-value">{:?}</div>
                </div>
                "#,
                if benford.passes {
                    "metric-pass"
                } else {
                    "metric-fail"
                },
                benford.p_value,
                if benford.mad <= 0.015 {
                    "metric-pass"
                } else {
                    "metric-warning"
                },
                benford.mad,
                benford.conformity
            ));
        }

        if let Some(ref temporal) = stat.temporal {
            metrics_html.push_str(&format!(
                r#"
                <div class="metric">
                    <div class="metric-label">Temporal Correlation</div>
                    <div class="metric-value {}">{:.2}</div>
                </div>
                <div class="metric">
                    <div class="metric-label">Weekend Ratio</div>
                    <div class="metric-value">{:.1}%</div>
                </div>
                "#,
                if temporal.pattern_correlation >= 0.8 {
                    "metric-pass"
                } else {
                    "metric-warning"
                },
                temporal.pattern_correlation,
                temporal.weekend_ratio * 100.0
            ));
        }

        format!(
            r#"
            <div class="card">
                <div class="card-title">Statistical Quality</div>
                <div class="metric-grid">
                    {metrics_html}
                </div>
            </div>
            "#
        )
    }

    /// Generate coherence section.
    fn generate_coherence_section(&self, report: &EvaluationReport) -> String {
        let Some(ref coh) = report.coherence else {
            return String::new();
        };

        let mut metrics_html = String::new();

        if let Some(ref balance) = coh.balance {
            metrics_html.push_str(&format!(
                r#"
                <div class="metric">
                    <div class="metric-label">Balance Sheet Equation</div>
                    <div class="metric-value {}">{}</div>
                </div>
                <div class="metric">
                    <div class="metric-label">Periods Evaluated</div>
                    <div class="metric-value">{}</div>
                </div>
                "#,
                if balance.equation_balanced {
                    "metric-pass"
                } else {
                    "metric-fail"
                },
                if balance.equation_balanced {
                    "Balanced"
                } else {
                    "Imbalanced"
                },
                balance.periods_evaluated
            ));
        }

        if let Some(ref sub) = coh.subledger {
            metrics_html.push_str(&format!(
                r#"
                <div class="metric">
                    <div class="metric-label">Subledger Reconciliation</div>
                    <div class="metric-value {}">{:.1}%</div>
                </div>
                "#,
                if sub.completeness_score >= 0.99 {
                    "metric-pass"
                } else {
                    "metric-fail"
                },
                sub.completeness_score * 100.0
            ));
        }

        if let Some(ref ic) = coh.intercompany {
            metrics_html.push_str(&format!(
                r#"
                <div class="metric">
                    <div class="metric-label">IC Match Rate</div>
                    <div class="metric-value {}">{:.1}%</div>
                </div>
                "#,
                if ic.match_rate >= 0.95 {
                    "metric-pass"
                } else {
                    "metric-warning"
                },
                ic.match_rate * 100.0
            ));
        }

        format!(
            r#"
            <div class="card">
                <div class="card-title">Semantic Coherence</div>
                <div class="metric-grid">
                    {metrics_html}
                </div>
            </div>
            "#
        )
    }

    /// Generate issues section.
    fn generate_issues_section(&self, report: &EvaluationReport) -> String {
        if report.all_issues.is_empty() {
            return r#"
            <div class="card">
                <div class="card-title">Issues</div>
                <p>No issues found.</p>
            </div>
            "#
            .to_string();
        }

        let mut issues_html = String::new();
        for issue in &report.all_issues {
            let severity_class = match issue.severity {
                IssueSeverity::Critical => "issue-critical",
                IssueSeverity::Warning => "issue-warning",
                IssueSeverity::Info => "issue-info",
            };
            let category_name = match issue.category {
                IssueCategory::Statistical => "Statistical",
                IssueCategory::Coherence => "Coherence",
                IssueCategory::Quality => "Quality",
                IssueCategory::MLReadiness => "ML Readiness",
            };

            issues_html.push_str(&format!(
                r#"
                <li class="issue-item {severity_class}">
                    <div class="issue-category">{category_name}</div>
                    <div>{}</div>
                </li>
                "#,
                issue.description
            ));
        }

        format!(
            r#"
            <div class="card">
                <div class="card-title">Issues ({} found)</div>
                <ul class="issues-list">
                    {issues_html}
                </ul>
            </div>
            "#,
            report.all_issues.len()
        )
    }

    /// Get CSS class for score value.
    fn score_class(&self, score: f64) -> &'static str {
        if score >= 0.9 {
            "metric-pass"
        } else if score >= 0.7 {
            "metric-warning"
        } else {
            "metric-fail"
        }
    }

    /// Get CSS class for score bar.
    fn score_bar_class(&self, score: f64) -> &'static str {
        if score >= 0.9 {
            "score-good"
        } else if score >= 0.7 {
            "score-medium"
        } else {
            "score-bad"
        }
    }
}

impl Default for HtmlReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportGenerator for HtmlReportGenerator {
    fn generate(&self, report: &EvaluationReport) -> EvalResult<String> {
        let css = if self.include_css {
            self.generate_css()
        } else {
            String::new()
        };

        let summary = self.generate_summary(report);
        let statistical = self.generate_statistical_section(report);
        let coherence = self.generate_coherence_section(report);
        let issues = self.generate_issues_section(report);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Evaluation Report - {}</title>
    {css}
</head>
<body>
    <div class="container">
        {summary}
        {statistical}
        {coherence}
        {issues}
    </div>
</body>
</html>"#,
            report.metadata.generated_at.format("%Y-%m-%d")
        );

        Ok(html)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::report::ReportMetadata;
    use chrono::Utc;

    #[test]
    fn test_html_generation() {
        let metadata = ReportMetadata {
            generated_at: Utc::now(),
            version: "1.0.0".to_string(),
            data_source: "test".to_string(),
            thresholds_name: "default".to_string(),
            records_evaluated: 1000,
            duration_ms: 500,
        };

        let report = EvaluationReport::new(metadata, None, None, None, None);
        let generator = HtmlReportGenerator::new();
        let html = generator.generate(&report).unwrap();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("PASSED"));
    }
}
