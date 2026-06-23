//! Hypervisor fleet manifest: cross-repo DAG orchestration.
//! See [[hypervisor-fleet-dag]] and [[ADR-0015-hypervisor-fleet-dag]].

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{PytxoError, Result};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetMeta {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// One node in a hypervisor fleet DAG — dispatches to a separate execution domain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetNode {
    pub id: String,
    pub repo: PathBuf,
    pub cmd: String,
    #[serde(default = "default_agents")]
    pub agents: usize,
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Optional per-node `pytxo.toml` path override.
    #[serde(default)]
    pub config: Option<PathBuf>,
}

fn default_agents() -> usize {
    1
}

/// Parsed fleet manifest (`[fleet]` + `[[node]]`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetManifest {
    pub fleet: FleetMeta,
    #[serde(default)]
    pub node: Vec<FleetNode>,
}

/// One wave of fleet nodes ready to dispatch in parallel.
#[derive(Clone, Debug, Serialize)]
pub struct FleetWave {
    pub wave: u32,
    pub nodes: Vec<FleetNode>,
}

/// Topological execution plan for a fleet.
#[derive(Clone, Debug, Serialize)]
pub struct FleetPlan {
    pub fleet_id: String,
    pub waves: Vec<FleetWave>,
}

impl FleetManifest {
    pub fn parse(raw: &str) -> Result<Self> {
        let manifest: FleetManifest = toml::from_str(raw)
            .map_err(|e| PytxoError::Config(format!("parse fleet toml: {e}")))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| PytxoError::Config(format!("read {}: {e}", path.display())))?;
        Self::parse(&raw)
    }

    fn validate(&self) -> Result<()> {
        if self.fleet.id.trim().is_empty() {
            return Err(PytxoError::Config("fleet.id must not be empty".into()));
        }
        if self.node.is_empty() {
            return Err(PytxoError::Config(
                "fleet must declare at least one [[node]] entry".into(),
            ));
        }
        let ids: HashSet<_> = self.node.iter().map(|n| n.id.as_str()).collect();
        if ids.len() != self.node.len() {
            return Err(PytxoError::Config("fleet node ids must be unique".into()));
        }
        for n in &self.node {
            if n.id.trim().is_empty() {
                return Err(PytxoError::Config("fleet node id must not be empty".into()));
            }
            if n.cmd.trim().is_empty() {
                return Err(PytxoError::Config(format!(
                    "fleet node '{}' must set cmd",
                    n.id
                )));
            }
            for dep in &n.depends_on {
                if !ids.contains(dep.as_str()) {
                    return Err(PytxoError::Config(format!(
                        "fleet node '{}' depends on unknown node '{dep}'",
                        n.id
                    )));
                }
            }
        }
        if let Some(cycle) = find_cycle(&self.node) {
            return Err(PytxoError::Config(format!(
                "fleet dependency cycle: {}",
                cycle.join(" -> ")
            )));
        }
        Ok(())
    }

    /// Build topological waves for hypervisor dispatch.
    pub fn plan(&self) -> Result<FleetPlan> {
        self.validate()?;
        let waves = build_fleet_waves(&self.node)?;
        Ok(FleetPlan {
            fleet_id: self.fleet.id.clone(),
            waves,
        })
    }

    pub fn node_by_id(&self, id: &str) -> Option<&FleetNode> {
        self.node.iter().find(|n| n.id == id)
    }

    /// Standard manifest location for a fleet id under the user config dir.
    pub fn user_manifest_path(id: &str) -> Option<PathBuf> {
        dirs_home().map(|h| h.join(".pytxo").join("fleets").join(format!("{id}.toml")))
    }

    /// Discover a manifest: explicit path, else `~/.pytxo/fleets/<id>.toml`.
    pub fn discover(explicit: Option<&Path>, id: Option<&str>) -> Option<PathBuf> {
        if let Some(p) = explicit {
            return Some(p.to_path_buf());
        }
        if let Some(id) = id {
            if let Some(p) = Self::user_manifest_path(id) {
                if p.exists() {
                    return Some(p);
                }
            }
        }
        None
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn build_fleet_waves(nodes: &[FleetNode]) -> Result<Vec<FleetWave>> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    for n in nodes {
        in_degree.insert(n.id.as_str(), 0);
    }
    for n in nodes {
        for _ in &n.depends_on {
            *in_degree.entry(n.id.as_str()).or_insert(0) += 1;
        }
    }

    let mut ready: Vec<&FleetNode> = nodes
        .iter()
        .filter(|n| in_degree[n.id.as_str()] == 0)
        .collect();

    let mut waves = Vec::new();
    let mut scheduled = 0usize;

    while scheduled < nodes.len() {
        if ready.is_empty() {
            return Err(PytxoError::Config(
                "cannot schedule remaining fleet nodes".into(),
            ));
        }

        let wave_index = waves.len() as u32;
        let wave_nodes: Vec<FleetNode> = ready.drain(..).cloned().collect();
        scheduled += wave_nodes.len();

        for n in &wave_nodes {
            for other in nodes {
                if other.depends_on.iter().any(|d| d == &n.id) {
                    let deg = in_degree.get_mut(other.id.as_str()).expect("deg");
                    *deg -= 1;
                    if *deg == 0 {
                        ready.push(other);
                    }
                }
            }
        }

        waves.push(FleetWave {
            wave: wave_index,
            nodes: wave_nodes,
        });
    }

    Ok(waves)
}

fn find_cycle(nodes: &[FleetNode]) -> Option<Vec<String>> {
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for n in nodes {
        graph.entry(n.id.as_str()).or_default();
        for dep in &n.depends_on {
            graph.entry(dep.as_str()).or_default().push(n.id.as_str());
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

    for n in nodes {
        if state.get(n.id.as_str()).copied().unwrap_or(0) == 0 {
            if let Some(c) = dfs(n.id.as_str(), &graph, &mut state, &mut stack) {
                return Some(c);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[fleet]
id = "api-then-web"
name = "Fix API then deploy web"

[[node]]
id = "fix-api"
repo = "/tmp/api"
cmd = "echo fix"
agents = 2

[[node]]
id = "deploy-web"
repo = "/tmp/web"
cmd = "echo deploy"
depends_on = ["fix-api"]
"#;

    #[test]
    fn parses_and_plans_waves() {
        let m = FleetManifest::parse(SAMPLE).unwrap();
        let plan = m.plan().unwrap();
        assert_eq!(plan.waves.len(), 2);
        assert_eq!(plan.waves[0].nodes[0].id, "fix-api");
        assert_eq!(plan.waves[1].nodes[0].id, "deploy-web");
    }

    #[test]
    fn rejects_unknown_dep() {
        let bad = r#"
[fleet]
id = "x"
[[node]]
id = "a"
repo = "/a"
cmd = "echo"
depends_on = ["missing"]
"#;
        assert!(FleetManifest::parse(bad).is_err());
    }

    #[test]
    fn rejects_cycle() {
        let bad = r#"
[fleet]
id = "x"
[[node]]
id = "a"
repo = "/a"
cmd = "echo"
depends_on = ["b"]
[[node]]
id = "b"
repo = "/b"
cmd = "echo"
depends_on = ["a"]
"#;
        assert!(FleetManifest::parse(bad).is_err());
    }

    #[test]
    fn parallel_wave_when_independent() {
        let raw = r#"
[fleet]
id = "parallel"
[[node]]
id = "a"
repo = "/a"
cmd = "echo a"
[[node]]
id = "b"
repo = "/b"
cmd = "echo b"
"#;
        let plan = FleetManifest::parse(raw).unwrap().plan().unwrap();
        assert_eq!(plan.waves.len(), 1);
        assert_eq!(plan.waves[0].nodes.len(), 2);
    }
}
