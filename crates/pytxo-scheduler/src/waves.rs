use pytxo_core::{ExecutionPlan, ScheduledTask, Task};

use crate::overlap::{find_conflicts, tasks_overlap};

/// Greedy wave assignment: each wave is a maximal set of pairwise non-conflicting tasks.
pub fn build_execution_plan(tasks: &[Task], max_agents: usize) -> ExecutionPlan {
    let conflicts = find_conflicts(tasks);
    let mut remaining: Vec<&Task> = tasks.iter().collect();
    let mut waves: Vec<Vec<ScheduledTask>> = Vec::new();

    while !remaining.is_empty() {
        let mut wave_tasks: Vec<&Task> = Vec::new();
        let mut i = 0;
        while i < remaining.len() {
            let candidate = remaining[i];
            let conflicts_with_wave = wave_tasks.iter().any(|t| tasks_overlap(t, candidate));
            if !conflicts_with_wave && wave_tasks.len() < max_agents {
                wave_tasks.push(candidate);
                remaining.remove(i);
            } else {
                i += 1;
            }
        }
        if wave_tasks.is_empty() {
            // Force progress: take first remaining alone (shouldn't happen if tasks valid)
            wave_tasks.push(remaining.remove(0));
        }
        let wave_index = waves.len() as u32;
        waves.push(
            wave_tasks
                .into_iter()
                .map(|t| ScheduledTask {
                    task_id: t.id.clone(),
                    agent: t.agent.clone(),
                    paths: t.paths.clone(),
                    wave: wave_index,
                    root: t.root.clone(),
                    signal_fidelity: t.signal_fidelity,
                })
                .collect(),
        );
    }

    ExecutionPlan {
        waves,
        conflicts,
        max_agents,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pytxo_core::TaskId;

    fn task(id: &str, paths: &[&str]) -> Task {
        Task {
            id: TaskId(id.to_string()),
            agent: "builder".into(),
            paths: paths.iter().map(|s| (*s).to_string()).collect(),
            depends_on: Vec::new(),
            root: None,
            signal_fidelity: None,
        }
    }

    #[test]
    fn conflicting_tasks_split_across_waves() {
        let tasks = vec![
            task("a", &["package.json"]),
            task("b", &["package.json"]),
            task("c", &["src/a.ts"]),
        ];
        let plan = build_execution_plan(&tasks, 3);
        assert_eq!(plan.waves.len(), 2);
        let wave0_len = plan.waves[0].len();
        assert!((1..=2).contains(&wave0_len));
    }
}
