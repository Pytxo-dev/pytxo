use std::collections::{HashMap, HashSet, VecDeque};

use pytxo_core::{ExecutionPlan, PytxoError, Result, ScheduledTask, Task};

use crate::overlap::{find_conflicts, find_cross_root_conflicts, tasks_overlap};
use crate::waves::build_execution_plan;

pub fn build_dag_plan(tasks: &[Task], max_agents: usize) -> Result<ExecutionPlan> {
    let mut ids = HashSet::new();
    for task in tasks {
        if task.id.0.trim().is_empty() {
            return Err(PytxoError::Scheduler("task id cannot be empty".into()));
        }
        if !ids.insert(task.id.0.as_str()) {
            return Err(PytxoError::Scheduler(format!(
                "duplicate task id: {}",
                task.id.0
            )));
        }
    }
    for t in tasks {
        for dep in &t.depends_on {
            if !ids.contains(dep.as_str()) {
                return Err(PytxoError::Scheduler(format!(
                    "task {} depends on unknown task {dep}",
                    t.id.0
                )));
            }
        }
    }

    if let Some(cycle) = find_cycle(tasks) {
        if std::env::var("PYTXO_DAG_MOCK").ok().as_deref() != Some("1") {
            return Err(PytxoError::Scheduler(format!(
                "dependency cycle: {}",
                cycle.join(" -> ")
            )));
        }
    }

    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    for t in tasks {
        in_degree.insert(t.id.0.as_str(), 0);
    }
    for t in tasks {
        for _ in &t.depends_on {
            *in_degree.entry(t.id.0.as_str()).or_insert(0) += 1;
        }
    }

    let mut ready: VecDeque<&Task> = tasks
        .iter()
        .filter(|t| in_degree[t.id.0.as_str()] == 0)
        .collect();

    let mut waves: Vec<Vec<ScheduledTask>> = Vec::new();
    let mut scheduled = 0usize;
    let mut conflicts = find_conflicts(tasks);
    for c in find_cross_root_conflicts(tasks) {
        if !conflicts
            .iter()
            .any(|x| x.task_a == c.task_a && x.task_b == c.task_b)
        {
            conflicts.push(c);
        }
    }

    let dag_recovery = std::env::var("PYTXO_DAG_RECOVERY").ok().as_deref() == Some("1");
    let mut recovery_warnings: Vec<String> = Vec::new();

    while scheduled < tasks.len() {
        if ready.is_empty() {
            if dag_recovery {
                let scheduled_ids: HashSet<&str> = waves
                    .iter()
                    .flat_map(|w| w.iter())
                    .map(|s| s.task_id.0.as_str())
                    .collect();
                if let Some(task) = tasks
                    .iter()
                    .find(|t| !scheduled_ids.contains(t.id.0.as_str()))
                {
                    recovery_warnings.push(format!(
                        "dag-recovery: force-scheduled stalled task {}",
                        task.id.0
                    ));
                    ready.push_back(task);
                } else {
                    return Err(PytxoError::Scheduler(
                        "cannot schedule remaining tasks (recovery exhausted)".into(),
                    ));
                }
            } else {
                return Err(PytxoError::Scheduler(
                    "cannot schedule remaining tasks".into(),
                ));
            }
        }

        let wave_index = waves.len() as u32;
        let mut wave_tasks: Vec<&Task> = Vec::new();
        let mut deferred: Vec<&Task> = Vec::new();

        while let Some(candidate) = ready.pop_front() {
            let path_conflict = wave_tasks.iter().any(|t| tasks_overlap(t, candidate));
            if path_conflict || wave_tasks.len() >= max_agents {
                deferred.push(candidate);
            } else {
                wave_tasks.push(candidate);
            }
        }

        if wave_tasks.is_empty() {
            let candidate = deferred
                .pop()
                .ok_or_else(|| PytxoError::Scheduler("stuck".into()))?;
            wave_tasks.push(candidate);
        } else {
            for t in deferred {
                ready.push_back(t);
            }
        }

        for t in &wave_tasks {
            scheduled += 1;
            for other in tasks {
                if other.depends_on.iter().any(|d| d == &t.id.0) {
                    let deg = in_degree.get_mut(other.id.0.as_str()).expect("deg");
                    *deg -= 1;
                    if *deg == 0 {
                        ready.push_back(other);
                    }
                }
            }
        }

        waves.push(
            wave_tasks
                .into_iter()
                .map(|t| ScheduledTask {
                    task_id: t.id.clone(),
                    agent: t.agent.clone(),
                    paths: t.paths.clone(),
                    depends_on: t.depends_on.clone(),
                    wave: wave_index,
                    root: t.root.clone(),
                    signal_fidelity: t.signal_fidelity,
                    verify: t.verify.clone(),
                })
                .collect(),
        );
    }

    Ok(ExecutionPlan {
        waves,
        conflicts,
        max_agents,
        warnings: recovery_warnings,
    })
}

fn find_cycle(tasks: &[Task]) -> Option<Vec<String>> {
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for t in tasks {
        graph.entry(t.id.0.as_str()).or_default();
        for dep in &t.depends_on {
            graph.entry(dep.as_str()).or_default().push(t.id.0.as_str());
        }
    }

    let mut state: HashMap<&str, u8> = HashMap::new();
    let mut stack: Vec<&str> = Vec::new();

    fn dfs<'a>(
        node: &'a str,
        graph: &HashMap<&'a str, Vec<&'a str>>,
        state: &mut HashMap<&'a str, u8>,
        stack: &mut Vec<&'a str>,
    ) -> Option<Vec<String>> {
        match state.get(node).copied().unwrap_or(0) {
            1 => {
                if let Some(pos) = stack.iter().position(|&n| n == node) {
                    let mut cycle: Vec<String> =
                        stack[pos..].iter().map(|s| (*s).to_string()).collect();
                    cycle.push(node.to_string());
                    return Some(cycle);
                }
                return None;
            }
            2 => return None,
            _ => {}
        }
        state.insert(node, 1);
        stack.push(node);
        if let Some(neighbors) = graph.get(node) {
            for &next in neighbors {
                if let Some(c) = dfs(next, graph, state, stack) {
                    return Some(c);
                }
            }
        }
        stack.pop();
        state.insert(node, 2);
        None
    }

    for t in tasks {
        if state.get(t.id.0.as_str()).copied().unwrap_or(0) == 0 {
            if let Some(c) = dfs(t.id.0.as_str(), &graph, &mut state, &mut stack) {
                return Some(c);
            }
        }
    }
    None
}

pub fn build_plan(tasks: &[Task], max_agents: usize, use_dag: bool) -> Result<ExecutionPlan> {
    let has_deps = tasks.iter().any(|t| !t.depends_on.is_empty());
    if use_dag || has_deps {
        build_dag_plan(tasks, max_agents)
    } else {
        Ok(build_execution_plan(tasks, max_agents))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pytxo_core::TaskId;

    fn task(id: &str, paths: &[&str], deps: &[&str]) -> Task {
        Task {
            id: TaskId(id.into()),
            agent: "a".into(),
            paths: paths.iter().map(|s| (*s).to_string()).collect(),
            depends_on: deps.iter().map(|s| (*s).to_string()).collect(),
            root: None,
            signal_fidelity: None,
            verify: vec![],
        }
    }

    #[test]
    fn dependency_orders_waves() {
        let tasks = vec![task("b", &["b.ts"], &["a"]), task("a", &["a.ts"], &[])];
        let plan = build_dag_plan(&tasks, 3).unwrap();
        assert_eq!(plan.waves.len(), 2);
        assert_eq!(plan.waves[0][0].task_id.0, "a");
        assert_eq!(plan.waves[1][0].task_id.0, "b");
        assert_eq!(plan.waves[1][0].depends_on, vec!["a"]);
    }

    #[test]
    fn cycle_errors_by_default() {
        let tasks = vec![task("a", &["a.ts"], &["b"]), task("b", &["b.ts"], &["a"])];
        assert!(build_dag_plan(&tasks, 3).is_err());
    }

    #[test]
    fn duplicate_task_ids_are_rejected_before_scheduling() {
        let tasks = vec![
            task("duplicate", &["a.ts"], &[]),
            task("duplicate", &["b.ts"], &[]),
        ];
        let error = build_dag_plan(&tasks, 3).expect_err("duplicate ids make receipts ambiguous");
        assert!(
            error.to_string().contains("duplicate task id")
                && error.to_string().contains("duplicate"),
            "unexpected error: {error}"
        );
    }
}
