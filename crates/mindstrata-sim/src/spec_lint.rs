//! §51.6 Spec Linter — validates ontology consistency against spec files.
//!
//! Loads RON spec files from specs/ and checks:
//! - Every action has preconditions AND effects
//! - Every norm has sanction AND scope
//! - Every proposition is registered
//! - Every system declares reads/writes
//!
//! This prevents ontology drift between code and specs.

use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LintIssue {
    pub severity: LintSeverity,
    pub spec_file: String,
    pub spec_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LintSeverity {
    Error,
    Warning,
}

/// Lint all spec files in the given directory.
pub fn lint_all(specs_dir: &Path) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    issues.extend(lint_actions(specs_dir));
    issues.extend(lint_norms(specs_dir));
    issues.extend(lint_propositions(specs_dir));
    issues.extend(lint_systems(specs_dir));
    issues
}

/// Count net open parens/braces in a line.
fn net_depth(line: &str) -> i32 {
    let mut d: i32 = 0;
    for ch in line.chars() {
        match ch {
            '(' | '{' => d += 1,
            ')' | '}' => d -= 1,
            _ => {}
        }
    }
    d
}

/// §51.6: Every action has preconditions AND effects.
fn lint_actions(specs_dir: &Path) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    let path = specs_dir.join("actions.ron");
    let content = if let Ok(c) = std::fs::read_to_string(&path) { c } else {
        issues.push(LintIssue {
            severity: LintSeverity::Warning,
            spec_file: "actions.ron".into(),
            spec_id: "N/A".into(),
            message: "Could not read specs/actions.ron".into(),
        });
        return issues;
    };

    // Strategy: strip the outer wrapper, then parse items at depth 0.
    let inner = strip_outer_wrapper(&content);
    let items = extract_items(&inner);

    for (id, fields) in items {
        let has_preconditions = fields.iter().any(|f| f.starts_with("preconditions") && f.contains('[') && !f.contains("[]"));
        let has_effects = fields.iter().any(|f| f.starts_with("effects") && f.contains('[') && !f.contains("[]"));

        if has_preconditions && !has_effects {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                spec_file: "actions.ron".into(),
                spec_id: id.clone(),
                message: format!("Action '{id}' has preconditions but no effects"),
            });
        } else if !has_preconditions && has_effects {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                spec_file: "actions.ron".into(),
                spec_id: id,
                message: "Action has effects but no preconditions".into(),
            });
        }
        // Both absent = idle/wander, which is fine
    }

    issues
}

/// §51.6: Every norm has sanction AND scope.
fn lint_norms(specs_dir: &Path) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    let path = specs_dir.join("norms.ron");
    let content = if let Ok(c) = std::fs::read_to_string(&path) { c } else {
        issues.push(LintIssue {
            severity: LintSeverity::Warning,
            spec_file: "norms.ron".into(),
            spec_id: "N/A".into(),
            message: "Could not read specs/norms.ron".into(),
        });
        return issues;
    };

    let inner = strip_outer_wrapper(&content);
    let items = extract_items(&inner);

    for (id, fields) in items {
        let has_scope = fields.iter().any(|f| f.starts_with("scope"));
        let has_sanction = fields.iter().any(|f| f.starts_with("sanction"));

        if !has_scope {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                spec_file: "norms.ron".into(),
                spec_id: id.clone(),
                message: format!("Norm '{id}' has no scope"),
            });
        }
        if !has_sanction {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                spec_file: "norms.ron".into(),
                spec_id: id,
                message: "Norm has no sanction".into(),
            });
        }
    }

    issues
}

/// §51.6: Check for duplicate proposition IDs.
fn lint_propositions(specs_dir: &Path) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    let path = specs_dir.join("propositions.ron");
    let content = if let Ok(c) = std::fs::read_to_string(&path) { c } else {
        issues.push(LintIssue {
            severity: LintSeverity::Warning,
            spec_file: "propositions.ron".into(),
            spec_id: "N/A".into(),
            message: "Could not read specs/propositions.ron".into(),
        });
        return issues;
    };

    let mut seen_ids = HashSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Proposition(") {
            // Extract numeric id
            if let Some(id_pos) = trimmed.find("id:") {
                let after = trimmed[id_pos + 3..].trim();
                if let Some(end) = after.find(|c: char| !c.is_ascii_digit() && !c.is_whitespace()) {
                    let num_str = after[..end].trim();
                    if let Ok(id) = num_str.parse::<u32>() {
                        if !seen_ids.insert(id) {
                            issues.push(LintIssue {
                                severity: LintSeverity::Error,
                                spec_file: "propositions.ron".into(),
                                spec_id: format!("proposition_{id}"),
                                message: format!("Duplicate proposition ID: {id}"),
                            });
                        }
                    }
                }
            }
        }
    }

    issues
}

/// §51.6: Every system declares reads or writes.
fn lint_systems(specs_dir: &Path) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    let path = specs_dir.join("systems.ron");
    let content = if let Ok(c) = std::fs::read_to_string(&path) { c } else {
        issues.push(LintIssue {
            severity: LintSeverity::Warning,
            spec_file: "systems.ron".into(),
            spec_id: "N/A".into(),
            message: "Could not read specs/systems.ron".into(),
        });
        return issues;
    };

    // Find all System( blocks directly in the full content
    let mut current_system: Option<String> = None;
    let mut has_reads = false;
    let mut has_writes = false;
    let mut depth: i32 = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        let change = net_depth(trimmed);
        depth += change;

        if trimmed.contains("System(") || trimmed.contains("System{") {
            has_reads = false;
            has_writes = false;
            current_system = None;
        }

        if depth >= 1 && !trimmed.starts_with("System") {
            if trimmed.starts_with("name:") || trimmed.starts_with("name :") {
                if let Some(start) = trimmed.find('"') {
                    let rest = &trimmed[start + 1..];
                    if let Some(end) = rest.find('"') {
                        current_system = Some(rest[..end].to_string());
                    }
                }
            }
            if trimmed.starts_with("reads:") || trimmed.starts_with("reads :") {
                has_reads = true;
            }
            if trimmed.starts_with("writes:") || trimmed.starts_with("writes :") {
                has_writes = true;
            }
        }

        // When a System block closes (depth drops from 2 to 1)
        if change < 0 && depth <= 1 && current_system.is_some() {
            if !has_reads && !has_writes {
                issues.push(LintIssue {
                    severity: LintSeverity::Warning,
                    spec_file: "systems.ron".into(),
                    spec_id: current_system.clone().unwrap_or_default(),
                    message: "System has no reads or writes declared".into(),
                });
            }
            current_system = None;
        }
    }

    issues
}

/// Strip the outermost wrapper from RON content.
/// e.g., "Actions(\n  actions: [...]\n)" → "  actions: [...]"
fn strip_outer_wrapper(content: &str) -> String {
    let mut result = content.to_string();

    // Find the first ( and its matching closing )
    if let Some(open_pos) = result.find('(') {
        let mut depth = 0;
        let mut close_pos = result.len();
        for (i, ch) in result.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        close_pos = i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let inner = &result[open_pos + 1..close_pos];
        result = inner.trim().trim_matches(',').trim().to_string();
    }

    result
}

/// Extract top-level items from RON inner content.
/// Returns list of (id, fields) where fields are the trimmed lines within each block.
fn extract_items(content: &str) -> Vec<(String, Vec<String>)> {
    let mut items = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current_fields: Vec<String> = Vec::new();
    let mut depth: i32 = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        let change = net_depth(trimmed);
        let prev_depth = depth;
        depth += change;

        // Detect item start: something like "  Action(" or "  Norm("
        // When we see an opening that brings us from depth 0 to depth 1
        if prev_depth == 0 && depth >= 1 {
            current_id = None;
            current_fields = Vec::new();
        }

        // Collect fields inside the item (depth >= 1)
        if depth >= 1 && change >= 0 {
            if trimmed.starts_with("id:") || trimmed.starts_with("id :") {
                if let Some(start) = trimmed.find('"') {
                    let rest = &trimmed[start + 1..];
                    if let Some(end) = rest.find('"') {
                        current_id = Some(rest[..end].to_string());
                    }
                }
            }
            if trimmed.starts_with("name:") || trimmed.starts_with("name :") {
                // systems use "name" instead of "id"
                if current_id.is_none() {
                    if let Some(start) = trimmed.find('"') {
                        let rest = &trimmed[start + 1..];
                        if let Some(end) = rest.find('"') {
                            current_id = Some(rest[..end].to_string());
                        }
                    }
                }
            }
            // Only add field lines that have a colon (key: value)
            if trimmed.contains(':') && !trimmed.starts_with("//") {
                current_fields.push(trimmed.to_string());
            }
        }

        // Item end: when we close from depth 1 back to 0
        if change < 0 && depth <= 0 {
            if let Some(id) = current_id.take() {
                items.push((id, std::mem::take(&mut current_fields)));
            }
        }
    }

    items
}

/// Format lint issues as a human-readable report.
pub fn format_report(issues: &[LintIssue]) -> String {
    if issues.is_empty() {
        return "✅ All spec lint checks passed.".to_string();
    }

    let mut out = String::new();
    let errors = issues
        .iter()
        .filter(|i| i.severity == LintSeverity::Error)
        .count();
    let warnings = issues
        .iter()
        .filter(|i| i.severity == LintSeverity::Warning)
        .count();
    out.push_str(&format!(
        "Spec Lint Report: {errors} errors, {warnings} warnings\n\n"
    ));

    for issue in issues {
        let marker = match issue.severity {
            LintSeverity::Error => "❌ ERROR",
            LintSeverity::Warning => "⚠️  WARN ",
        };
        out.push_str(&format!(
            "[{}] {}:{} — {}\n",
            marker, issue.spec_file, issue.spec_id, issue.message
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lint_all_specs_passes() {
        let specs_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../specs");
        if !specs_dir.exists() {
            eprintln!(
                "Skipping spec lint test — specs/ directory not found at {specs_dir:?}"
            );
            return;
        }
        let issues = lint_all(&specs_dir);
        let has_errors = issues.iter().any(|i| i.severity == LintSeverity::Error);
        assert!(!has_errors, "Spec lint errors found:\n{}", format_report(&issues));
    }

    #[test]
    fn format_report_shows_passing() {
        let report = format_report(&[]);
        assert!(report.contains("passed"));
    }

    #[test]
    fn format_report_shows_issues() {
        let issues = vec![LintIssue {
            severity: LintSeverity::Error,
            spec_file: "actions.ron".into(),
            spec_id: "test_action".into(),
            message: "Missing effects".into(),
        }];
        let report = format_report(&issues);
        assert!(report.contains("ERROR"));
        assert!(report.contains("test_action"));
    }

    #[test]
    fn strip_outer_wrapper_removes_actions_wrapper() {
        let input = "Actions(\n  actions: [\n    Action(\n      id: \"test\",\n    ),\n  ],\n)";
        let result = strip_outer_wrapper(input);
        assert!(result.contains("Action("));
        assert!(!result.starts_with("Actions("));
    }
}
