//! The entire reviewed coarse task projection lives here. The disclosure
//! scope hashes these source bytes so changing the projection invalidates old
//! workspace grants even if a caller forgets to bump a manual version label.

use pytxo_core::routing::{TaskContract, TaskKind};
use pytxo_planner::advisor::AdvisorPacket;

/// Never copy free-form task text into the advisor request. A heuristic
/// scrubber cannot prove arbitrary business or personal details were removed.
pub(super) fn redacted_reviewed_task_description(_goal: &str) -> &'static str {
    "Classify reviewed repository task using coarse facts"
}

/// Only fixed wording and reviewed coarse facts enter the outbound packet.
/// Never copy free-form goals, paths, checks, profile metadata or credentials.
pub(crate) fn reviewed_task_advisor_packet(task: &TaskContract) -> anyhow::Result<AdvisorPacket> {
    let kind = match task.task_kind {
        Some(TaskKind::Documentation) => "documentation",
        Some(TaskKind::Formatting) => "formatting",
        Some(TaskKind::Rename) => "rename",
        Some(TaskKind::LocalTransformation) => "local_transformation",
        Some(TaskKind::Diagnosis) => "diagnosis",
        Some(TaskKind::Architecture) => "architecture",
        Some(TaskKind::Other) | None => "unspecified",
    };
    let features = [
        format!("task_kind_{kind}"),
        if task.checks.is_empty() {
            "no_reviewed_checks"
        } else {
            "reviewed_checks"
        }
        .into(),
        match task.cross_component_requirement {
            Some(true) => "cross_component_required",
            Some(false) => "single_component_claimed",
            None => "cross_component_unknown",
        }
        .into(),
        if task.context_complete {
            "context_claimed_complete"
        } else {
            "context_incomplete"
        }
        .into(),
        if task.strong_only {
            "strong_only"
        } else {
            "role_unrestricted"
        }
        .into(),
        if task.required_egress.is_empty() {
            "no_required_egress"
        } else {
            "egress_required"
        }
        .into(),
        match (task.task_kind, task.repeatable_symptom_supplied) {
            (Some(TaskKind::Diagnosis), Some(true)) => "repeatable_symptom_supplied",
            (Some(TaskKind::Diagnosis), Some(false)) => "repeatable_symptom_not_supplied",
            _ => "repeatable_symptom_unknown",
        }
        .into(),
        match (task.task_kind, task.specific_cause_hypothesis_supplied) {
            (Some(TaskKind::Diagnosis), Some(true)) => "specific_cause_hypothesis_supplied",
            (Some(TaskKind::Diagnosis), Some(false)) => "specific_cause_hypothesis_not_supplied",
            _ => "specific_cause_hypothesis_unknown",
        }
        .into(),
    ];
    AdvisorPacket::new(
        redacted_reviewed_task_description(&task.goal),
        features,
        "everyday execution role",
        "strong execution role",
    )
    .map_err(|reason| anyhow::anyhow!("reviewed task advisor packet invalid: {reason:?}"))
}
