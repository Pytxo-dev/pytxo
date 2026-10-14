//! Pybench: deterministic, offline benchmarks of the landing gate.
//!
//! `land` builds a throwaway repository per scenario and compares two
//! policies on the same agent branches: *per-branch CI* (test each branch
//! alone, merge every branch that passed, never test the result) and
//! `pytxo land` (test the combination, refuse what moved). The scenarios are
//! synthetic mechanism tests: they show what each policy does in a known
//! situation, not how often those situations happen in real repositories.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::checks::run_check;
use crate::git::Git;
use crate::{land, LandInput, LandOptions};

pub const SCHEMA: &str = "pybench/v1";
const CHECK: &str = "node run-tests.js";

#[derive(Clone, Debug, Serialize)]
pub struct BenchReport {
    pub schema: &'static str,
    pub pytxo_version: &'static str,
    pub created_at: String,
    pub machine: Machine,
    pub land: Option<LandSuite>,
    pub overhead: Option<OverheadSuite>,
    pub signal: Option<SignalSuite>,
    pub notes: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Machine {
    pub os: &'static str,
    pub arch: &'static str,
    pub cpus: usize,
    pub git: String,
    pub node: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LandSuite {
    pub skipped: Option<String>,
    pub scenarios: Vec<ScenarioResult>,
    pub summary: LandSummary,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct LandSummary {
    pub scenarios: usize,
    /// Pytxo decision matched the scenario's known-correct outcome.
    pub correct: usize,
    /// Broken main states per-branch CI landed without noticing.
    pub naive_landed_broken: usize,
    /// Of those, how many `pytxo land` refused.
    pub pytxo_prevented: usize,
    /// Scenarios that should land but `pytxo land` refused.
    pub false_refusals: usize,
    pub conflicts_caught: usize,
    pub stale_refused: usize,
    pub symbol_collisions_reported: usize,
    pub median_gate_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScenarioResult {
    pub id: &'static str,
    pub title: &'static str,
    pub inputs: usize,
    pub expected: &'static str,
    pub pytxo_decision: String,
    /// What blocked it: `conflict`, `combined_check`, `stale`, or empty.
    pub pytxo_layer: String,
    pub symbol_collisions: usize,
    /// `landed_ok | landed_broken | manual_conflict | not_modeled`.
    pub naive_outcome: &'static str,
    pub correct: bool,
    pub gate_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct OverheadSuite {
    pub rows: Vec<OverheadRow>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OverheadRow {
    pub inputs: usize,
    pub files: usize,
    /// Median of 3: sequential `git merge --no-ff` of every input.
    pub git_merge_ms: u64,
    /// Median of 3: `pytxo land` in shadow mode, no checks (gate cost only).
    pub pytxo_land_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct SignalSuite {
    pub repo: String,
    pub files: usize,
    pub original_bytes: usize,
    pub scaffold_bytes: usize,
    pub weighted_reduction_pct: f64,
    pub rule: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suite {
    Land,
    Overhead,
    Signal,
}

pub fn run(
    suites: &[Suite],
    signal_repo: Option<&Path>,
    mut progress: impl FnMut(&str),
) -> Result<BenchReport> {
    let git = Command::new("git")
        .arg("--version")
        .output()
        .context("git is required")?;
    let node = Command::new("node")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let machine = Machine {
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        cpus: std::thread::available_parallelism().map_or(1, |n| n.get()),
        git: String::from_utf8_lossy(&git.stdout).trim().to_string(),
        node: node.clone(),
    };
    let land = suites
        .contains(&Suite::Land)
        .then(|| land_suite(node.is_some(), &mut progress))
        .transpose()?;
    let overhead = suites
        .contains(&Suite::Overhead)
        .then(|| overhead_suite(&mut progress))
        .transpose()?;
    let signal = suites
        .contains(&Suite::Signal)
        .then(|| signal_suite(signal_repo.unwrap_or(Path::new(".")), &mut progress))
        .transpose()?;
    Ok(BenchReport {
        schema: SCHEMA,
        pytxo_version: env!("CARGO_PKG_VERSION"),
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        machine,
        land,
        overhead,
        signal,
        notes: vec![
            "Land scenarios are synthetic mechanism tests in throwaway repositories. They show what each policy does in a known situation, not how often it happens in real projects.",
            "Per-branch CI = test each branch alone against the target, merge every branch that passed, never test the merged result.",
            "No model or agent is called. Timings depend on this machine's git and disk speed.",
        ],
    })
}

// ---------------------------------------------------------------- land suite

enum Edit {
    Write(&'static str, String),
    Delete(&'static str),
}

struct Scenario {
    id: &'static str,
    title: &'static str,
    expected: &'static str,
    branches: Vec<(&'static str, Vec<Edit>)>,
    /// The check commits to the target while the gate runs.
    moves_target: bool,
}

const RUNNER: &str = "const fs = require('fs');\nconst path = require('path');\nlet failed = 0;\nfor (const f of fs.readdirSync('tests').sort()) {\n  try {\n    require(path.resolve('tests', f));\n    console.log('ok   ' + f);\n  } catch (e) {\n    failed++;\n    console.log('FAIL ' + f + ': ' + e.message);\n  }\n}\nprocess.exit(failed ? 1 : 0);\n";

const FORMAT: &str = "const config = { symbol: '$' };\n\nfunction money(n) {\n  return config.symbol + n.toFixed(2);\n}\n\nmodule.exports = { money, config };\n";

const CART: &str = "const { money } = require('./format.js');\n\nfunction price(item) {\n  return item.cost;\n}\n\nfunction total(items) {\n  let sum = 0;\n  for (const i of items) sum += price(i);\n  return sum;\n}\n\nfunction label(items) {\n  return 'Total: ' + money(total(items));\n}\n\nmodule.exports = { price, total, label };\n";

const CART_TEST: &str = "const assert = require('assert');\nconst { total, label } = require('../src/cart.js');\nassert.strictEqual(total([{ cost: 1 }, { cost: 2 }]), 3);\nassert.strictEqual(label([{ cost: 1 }]), 'Total: $1.00');\n";

const LEGACY: &str = "function legacy() {\n  return 'v1';\n}\n\nmodule.exports = { legacy };\n";

fn base_files() -> Vec<(&'static str, String)> {
    vec![
        ("run-tests.js", RUNNER.into()),
        ("src/format.js", FORMAT.into()),
        ("src/cart.js", CART.into()),
        ("src/legacy.js", LEGACY.into()),
        ("tests/cart.test.js", CART_TEST.into()),
        ("README.md", "# shop\n".into()),
    ]
}

fn w(path: &'static str, body: impl Into<String>) -> Edit {
    Edit::Write(path, body.into())
}

fn scenarios() -> Vec<Scenario> {
    let tax = || {
        vec![
            w("src/tax.js", "function tax(n) {\n  return Math.round(n * 10) / 100;\n}\n\nmodule.exports = { tax };\n"),
            w("tests/tax.test.js", "const assert = require('assert');\nconst { tax } = require('../src/tax.js');\nassert.strictEqual(tax(10), 1);\n"),
        ]
    };
    vec![
        Scenario {
            id: "disjoint-files",
            title: "Two agents, separate files",
            expected: "ready",
            branches: vec![("agent/tax", tax()), ("agent/readme", vec![w("README.md", "# shop\n\nAdds tax.\n")])],
            moves_target: false,
        },
        Scenario {
            id: "same-file-separate-functions",
            title: "Same file, different functions",
            expected: "ready",
            branches: vec![
                ("agent/qty", vec![
                    w("src/cart.js", CART.replace("return item.cost;", "return item.cost * (item.qty ?? 1);")),
                    w("tests/qty.test.js", "const assert = require('assert');\nconst { total } = require('../src/cart.js');\nassert.strictEqual(total([{ cost: 2, qty: 3 }]), 6);\n"),
                ]),
                ("agent/wording", vec![
                    w("src/cart.js", CART.replace("'Total: '", "'Order total: '")),
                    w("tests/cart.test.js", CART_TEST.replace("'Total: $1.00'", "'Order total: $1.00'")),
                ]),
            ],
            moves_target: false,
        },
        Scenario {
            id: "same-function-compatible",
            title: "Same function, compatible edits (reported, not blocked)",
            expected: "ready",
            branches: vec![
                ("agent/comment", vec![w("src/cart.js", CART.replace("let sum = 0;", "let sum = 0; // running total"))]),
                ("agent/round", vec![w("src/cart.js", CART.replace("return sum;", "return Math.round(sum * 100) / 100;"))]),
            ],
            moves_target: false,
        },
        Scenario {
            id: "textual-conflict",
            title: "Both agents rewrite the same line",
            expected: "blocked:conflict",
            branches: vec![
                ("agent/sum", vec![
                    w("src/cart.js", CART.replace("'Total: '", "'Sum: '")),
                    w("tests/cart.test.js", CART_TEST.replace("'Total: $1.00'", "'Sum: $1.00'")),
                ]),
                ("agent/grand", vec![
                    w("src/cart.js", CART.replace("'Total: '", "'Grand total: '")),
                    w("tests/cart.test.js", CART_TEST.replace("'Total: $1.00'", "'Grand total: $1.00'")),
                ]),
            ],
            moves_target: false,
        },
        Scenario {
            id: "rename-vs-new-caller",
            title: "One agent renames a function, another adds a caller of the old name",
            expected: "blocked:combined_check",
            branches: vec![
                ("agent/rename", vec![w("src/cart.js", CART.replace("price", "unitPrice"))]),
                ("agent/discount", vec![
                    w("src/discount.js", "const { price } = require('./cart.js');\n\nfunction discounted(item) {\n  return price(item) * 0.9;\n}\n\nmodule.exports = { discounted };\n"),
                    w("tests/discount.test.js", "const assert = require('assert');\nconst { discounted } = require('../src/discount.js');\nassert.strictEqual(discounted({ cost: 10 }), 9);\n"),
                ]),
            ],
            moves_target: false,
        },
        Scenario {
            id: "signature-change",
            title: "One agent adds a required parameter, another adds a call without it",
            expected: "blocked:combined_check",
            branches: vec![
                ("agent/symbol-param", vec![
                    w("src/format.js", FORMAT.replace("function money(n) {\n  return config.symbol + n.toFixed(2);", "function money(n, symbol) {\n  if (!symbol) throw new Error('money() needs a symbol');\n  return symbol + n.toFixed(2);")),
                    w("src/cart.js", CART.replace("money(total(items))", "money(total(items), '$')")),
                ]),
                ("agent/receipt", vec![
                    w("src/receipt.js", "const { money } = require('./format.js');\n\nfunction line(name, n) {\n  return name + ' ' + money(n);\n}\n\nmodule.exports = { line };\n"),
                    w("tests/receipt.test.js", "const assert = require('assert');\nconst { line } = require('../src/receipt.js');\nassert.strictEqual(line('tea', 2), 'tea $2.00');\n"),
                ]),
            ],
            moves_target: false,
        },
        Scenario {
            id: "deleted-helper",
            title: "One agent deletes a helper it thinks is unused, another starts using it",
            expected: "blocked:combined_check",
            branches: vec![
                ("agent/cleanup", vec![
                    w("src/cart.js", CART.replace("\nfunction label(items) {\n  return 'Total: ' + money(total(items));\n}\n", "").replace("{ price, total, label }", "{ price, total }").replace("const { money } = require('./format.js');\n\n", "")),
                    w("tests/cart.test.js", "const assert = require('assert');\nconst { total } = require('../src/cart.js');\nassert.strictEqual(total([{ cost: 1 }, { cost: 2 }]), 3);\n"),
                ]),
                ("agent/email", vec![
                    w("src/email.js", "const { label } = require('./cart.js');\n\nfunction body(items) {\n  return 'Thanks! ' + label(items);\n}\n\nmodule.exports = { body };\n"),
                    w("tests/email.test.js", "const assert = require('assert');\nconst { body } = require('../src/email.js');\nassert.strictEqual(body([{ cost: 1 }]), 'Thanks! Total: $1.00');\n"),
                ]),
            ],
            moves_target: false,
        },
        Scenario {
            id: "test-contract-flip",
            title: "One agent changes a default, another adds a test that relies on the old one",
            expected: "blocked:combined_check",
            branches: vec![
                ("agent/euro", vec![
                    w("src/format.js", FORMAT.replace("symbol: '$'", "symbol: '€'")),
                    w("tests/cart.test.js", CART_TEST.replace("'Total: $1.00'", "'Total: €1.00'")),
                ]),
                ("agent/format-tests", vec![w("tests/format.test.js", "const assert = require('assert');\nconst { money } = require('../src/format.js');\nassert.strictEqual(money(2), '$2.00');\n")]),
            ],
            moves_target: false,
        },
        Scenario {
            id: "target-moved",
            title: "The target branch moves while the gate is checking",
            expected: "stale",
            branches: vec![("agent/tax", tax())],
            moves_target: true,
        },
        Scenario {
            id: "three-way-fan-in",
            title: "Three agents, separate files",
            expected: "ready",
            branches: vec![
                ("agent/tax", tax()),
                ("agent/readme", vec![w("README.md", "# shop\n\nNow with tax.\n")]),
                ("agent/guide", vec![w("docs/guide.md", "# Guide\n\nRun `node run-tests.js`.\n")]),
            ],
            moves_target: false,
        },
        Scenario {
            id: "add-add",
            title: "Both agents create the same new file differently",
            expected: "blocked:conflict",
            branches: vec![
                ("agent/util-a", vec![w("src/util.js", "module.exports = { id: (x) => x };\n")]),
                ("agent/util-b", vec![w("src/util.js", "module.exports = { noop: () => {} };\n")]),
            ],
            moves_target: false,
        },
        Scenario {
            id: "modify-delete",
            title: "One agent deletes a file the other edits",
            expected: "blocked:conflict",
            branches: vec![
                ("agent/drop-legacy", vec![Edit::Delete("src/legacy.js")]),
                ("agent/legacy-v2", vec![w("src/legacy.js", LEGACY.replace("'v1'", "'v2'"))]),
            ],
            moves_target: false,
        },
    ]
}

fn sh_git(dir: &Path, args: &[&str]) -> Result<String> {
    Git::owned(dir, Some(("Pybench", "pybench@pytxo.invalid"))).run(args)
}

fn write_files(dir: &Path, files: &[(&str, String)]) -> Result<()> {
    for (path, body) in files {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().expect("relative path"))?;
        std::fs::write(p, body)?;
    }
    Ok(())
}

fn build_repo(scenario: &Scenario) -> Result<tempdir::Dir> {
    let dir = tempdir::Dir::new("pybench")?;
    let d = dir.path();
    sh_git(d, &["init", "-q", "-b", "main"])?;
    sh_git(d, &["config", "core.autocrlf", "false"])?;
    sh_git(d, &["config", "user.name", "Pybench"])?;
    sh_git(d, &["config", "user.email", "pybench@pytxo.invalid"])?;
    write_files(d, &base_files())?;
    sh_git(d, &["add", "-A"])?;
    sh_git(d, &["commit", "-q", "-m", "base"])?;
    for (name, edits) in &scenario.branches {
        sh_git(d, &["checkout", "-q", "-b", name, "main"])?;
        for edit in edits {
            match edit {
                Edit::Write(path, body) => write_files(d, &[(path, body.clone())])?,
                Edit::Delete(path) => std::fs::remove_file(d.join(path))?,
            }
        }
        sh_git(d, &["add", "-A"])?;
        sh_git(d, &["commit", "-q", "-m", name])?;
        sh_git(d, &["checkout", "-q", "main"])?;
    }
    Ok(dir)
}

/// Per-branch CI: test each branch alone, merge the ones that passed, never
/// test the merged result. Returns the outcome on a scratch copy of main.
fn naive_policy(repo: &Path, branches: &[&str]) -> Result<&'static str> {
    let scratch = repo.join(".pybench-naive");
    sh_git(
        repo,
        &[
            "worktree",
            "add",
            "-q",
            "--detach",
            &scratch.to_string_lossy(),
            "main",
        ],
    )?;
    let g = Git::owned(&scratch, Some(("Pybench", "pybench@pytxo.invalid")));
    let base = g.run(&["rev-parse", "HEAD"])?;
    let mut passing = Vec::new();
    for b in branches {
        g.run(&["reset", "-q", "--hard", &base])?;
        if g.ok(&["merge", "-q", "--no-ff", "--no-edit", b])
            && run_check("solo", CHECK, &scratch, Duration::from_secs(120)).passed
        {
            passing.push(*b);
        } else {
            let _ = g.output(&["merge", "--abort"]);
        }
    }
    g.run(&["reset", "-q", "--hard", &base])?;
    let mut outcome = "landed_ok";
    for b in &passing {
        if !g.ok(&["merge", "-q", "--no-ff", "--no-edit", b]) {
            let _ = g.output(&["merge", "--abort"]);
            outcome = "manual_conflict";
        }
    }
    if outcome == "landed_ok"
        && !run_check("after", CHECK, &scratch, Duration::from_secs(120)).passed
    {
        outcome = "landed_broken";
    }
    let _ = sh_git(
        repo,
        &["worktree", "remove", "--force", &scratch.to_string_lossy()],
    );
    Ok(outcome)
}

fn land_suite(has_node: bool, progress: &mut impl FnMut(&str)) -> Result<LandSuite> {
    if !has_node {
        return Ok(LandSuite {
            skipped: Some(
                "node was not found on PATH; land scenarios run `node run-tests.js` as their check"
                    .into(),
            ),
            scenarios: Vec::new(),
            summary: LandSummary::default(),
        });
    }
    let mut results = Vec::new();
    for scenario in scenarios() {
        progress(&format!("land  {}", scenario.id));
        let dir = build_repo(&scenario)?;
        let d = dir.path();
        let names: Vec<&str> = scenario.branches.iter().map(|(n, _)| *n).collect();
        let naive = if scenario.moves_target {
            "not_modeled"
        } else {
            naive_policy(d, &names)?
        };

        let mut opts = LandOptions::new(d, names.iter().map(|n| LandInput::parse(n)).collect());
        opts.checks = vec![CHECK.into()];
        opts.receipts_dir = Some(d.join(".pybench-receipts"));
        opts.check_timeout = Duration::from_secs(120);
        if scenario.moves_target {
            opts.apply = true;
            opts.checks.push(format!(
                "git -C \"{}\" commit -q --allow-empty -m moved",
                d.display().to_string().replace('\\', "/")
            ));
        }
        let started = Instant::now();
        let outcome = land(&opts)?;
        let gate_ms = started.elapsed().as_millis() as u64;
        let p = &outcome.receipt.predicate;
        let layer = if p.decision == "stale" {
            "stale"
        } else if p.inputs.iter().any(|i| i.status == "conflict") {
            "conflict"
        } else if p.checks.iter().any(|c| c.scope == "combined" && !c.passed) {
            "combined_check"
        } else {
            ""
        };
        let got = if layer.is_empty() || p.decision == "stale" {
            p.decision.clone()
        } else {
            format!("{}:{layer}", p.decision)
        };
        let collisions = p
            .shared_files
            .iter()
            .map(|f| f.symbol_collisions.len())
            .sum();
        results.push(ScenarioResult {
            id: scenario.id,
            title: scenario.title,
            inputs: names.len(),
            expected: scenario.expected,
            correct: got == scenario.expected,
            pytxo_decision: p.decision.clone(),
            pytxo_layer: layer.into(),
            symbol_collisions: collisions,
            naive_outcome: naive,
            gate_ms,
        });
    }
    let mut gate: Vec<u64> = results.iter().map(|r| r.gate_ms).collect();
    gate.sort_unstable();
    let summary = LandSummary {
        scenarios: results.len(),
        correct: results.iter().filter(|r| r.correct).count(),
        naive_landed_broken: results
            .iter()
            .filter(|r| r.naive_outcome == "landed_broken")
            .count(),
        pytxo_prevented: results
            .iter()
            .filter(|r| {
                r.naive_outcome == "landed_broken"
                    && r.pytxo_decision != "ready"
                    && r.pytxo_decision != "landed"
            })
            .count(),
        false_refusals: results
            .iter()
            .filter(|r| r.expected == "ready" && r.pytxo_decision != "ready")
            .count(),
        conflicts_caught: results
            .iter()
            .filter(|r| r.pytxo_layer == "conflict")
            .count(),
        stale_refused: results
            .iter()
            .filter(|r| r.pytxo_decision == "stale")
            .count(),
        symbol_collisions_reported: results.iter().map(|r| r.symbol_collisions).sum(),
        median_gate_ms: gate.get(gate.len() / 2).copied().unwrap_or(0),
    };
    Ok(LandSuite {
        skipped: None,
        scenarios: results,
        summary,
    })
}

// ------------------------------------------------------------ overhead suite

fn overhead_suite(progress: &mut impl FnMut(&str)) -> Result<OverheadSuite> {
    let mut rows = Vec::new();
    for &(inputs, files) in &[
        (2usize, 10usize),
        (5, 10),
        (10, 10),
        (2, 100),
        (5, 100),
        (10, 100),
    ] {
        progress(&format!("overhead  {inputs} inputs × {files} files"));
        let dir = tempdir::Dir::new("pybench-overhead")?;
        let d = dir.path();
        sh_git(d, &["init", "-q", "-b", "main"])?;
        sh_git(d, &["config", "core.autocrlf", "false"])?;
        let base: Vec<(&str, String)> = Vec::new();
        write_files(d, &base)?;
        for f in 0..files {
            let path = d.join(format!("src/m{f}.js"));
            std::fs::create_dir_all(path.parent().expect("dir"))?;
            std::fs::write(
                path,
                format!("function f{f}() {{\n  return {f};\n}}\nmodule.exports = {{ f{f} }};\n"),
            )?;
        }
        sh_git(d, &["add", "-A"])?;
        sh_git(d, &["commit", "-q", "-m", "base"])?;
        let names: Vec<String> = (0..inputs).map(|i| format!("agent/{i}")).collect();
        for (i, name) in names.iter().enumerate() {
            sh_git(d, &["checkout", "-q", "-b", name, "main"])?;
            // Each input owns a disjoint slice of the files.
            for f in (i..files).step_by(inputs) {
                std::fs::write(d.join(format!("src/m{f}.js")), format!("function f{f}() {{\n  return {f} + {i};\n}}\nmodule.exports = {{ f{f} }};\n"))?;
            }
            sh_git(d, &["commit", "-q", "-am", name])?;
            sh_git(d, &["checkout", "-q", "main"])?;
        }
        let mut merge_ms = Vec::new();
        let mut land_ms = Vec::new();
        for _ in 0..3 {
            let scratch = d.join(".pybench-merge");
            sh_git(
                d,
                &[
                    "worktree",
                    "add",
                    "-q",
                    "--detach",
                    &scratch.to_string_lossy(),
                    "main",
                ],
            )?;
            let g = Git::owned(&scratch, Some(("Pybench", "pybench@pytxo.invalid")));
            let t = Instant::now();
            for n in &names {
                g.run(&["merge", "-q", "--no-ff", "--no-edit", n])?;
            }
            merge_ms.push(t.elapsed().as_millis() as u64);
            sh_git(
                d,
                &["worktree", "remove", "--force", &scratch.to_string_lossy()],
            )?;

            let mut opts = LandOptions::new(d, names.iter().map(|n| LandInput::parse(n)).collect());
            opts.receipts_dir = Some(d.join(".pybench-receipts"));
            opts.keep_ref = false;
            let t = Instant::now();
            let outcome = land(&opts)?;
            land_ms.push(t.elapsed().as_millis() as u64);
            if outcome.decision() != "ready" {
                bail!(
                    "overhead fixture did not combine cleanly: {:?}",
                    outcome.receipt.predicate.reasons
                );
            }
        }
        merge_ms.sort_unstable();
        land_ms.sort_unstable();
        rows.push(OverheadRow {
            inputs,
            files,
            git_merge_ms: merge_ms[1],
            pytxo_land_ms: land_ms[1],
        });
    }
    Ok(OverheadSuite { rows })
}

// -------------------------------------------------------------- signal suite

fn signal_suite(repo: &Path, progress: &mut impl FnMut(&str)) -> Result<SignalSuite> {
    progress("signal  scaffolding tracked source");
    let git = Git::user(repo);
    let root = PathBuf::from(
        git.run(&["rev-parse", "--show-toplevel"])
            .context("signal suite needs a git repository")?,
    );
    let files = Git::user(&root).lines(&["ls-files"])?;
    let excluded = [
        "test",
        "tests",
        "fixtures",
        "fixture",
        "vendor",
        "node_modules",
        "dist",
        "build",
        "generated",
        "stories",
        "e2e",
        "__tests__",
        "target",
    ];
    let (mut n, mut original, mut scaffold) = (0usize, 0usize, 0usize);
    for rel in files {
        let lower = rel.to_ascii_lowercase();
        if lower.split('/').any(|seg| excluded.contains(&seg))
            || lower.contains(".test.")
            || lower.contains(".spec.")
            || lower.contains(".stories.")
        {
            continue;
        }
        let path = root.join(&rel);
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        if source.len() < 1024 {
            continue;
        }
        let Ok(result) =
            pytxo_signal::scaffold_source(&path, &source, pytxo_core::FidelityTier::Low)
        else {
            continue;
        };
        if result.fallback_raw {
            continue;
        }
        n += 1;
        original += result.stats.original_bytes;
        scaffold += result.stats.scaffolded_bytes;
    }
    let pct = if original == 0 {
        0.0
    } else {
        (original - scaffold) as f64 * 100.0 / original as f64
    };
    Ok(SignalSuite {
        repo: root.file_name().map_or_else(|| "repository".into(), |n| n.to_string_lossy().into_owned()),
        files: n,
        original_bytes: original,
        scaffold_bytes: scaffold,
        weighted_reduction_pct: (pct * 10.0).round() / 10.0,
        rule: "Tracked files in a Signal Core language, at least 1 KiB, excluding tests, fixtures, stories, vendor, build and generated paths. Byte reduction of the structural scaffold; not a token or cost measurement.",
    })
}

// ------------------------------------------------------------------ render

pub fn render_text(r: &BenchReport) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    let _ = writeln!(
        s,
        "Pybench {}  ·  pytxo {}  ·  {} {} ({} cpus)",
        r.created_at, r.pytxo_version, r.machine.os, r.machine.arch, r.machine.cpus
    );
    if let Some(l) = &r.land {
        let _ = writeln!(s, "\nland: what lands on main");
        if let Some(why) = &l.skipped {
            let _ = writeln!(s, "  skipped: {why}");
        } else {
            let _ = writeln!(
                s,
                "  {:<30} {:<26} {:<16} {:>8}",
                "scenario", "pytxo land", "per-branch CI", "gate"
            );
            for x in &l.scenarios {
                let got = if x.pytxo_layer.is_empty() || x.pytxo_decision == "stale" {
                    x.pytxo_decision.clone()
                } else {
                    format!("{} ({})", x.pytxo_decision, x.pytxo_layer)
                };
                let _ = writeln!(
                    s,
                    "  {:<30} {:<26} {:<16} {:>6}ms{}",
                    x.id,
                    got,
                    x.naive_outcome,
                    x.gate_ms,
                    if x.correct { "" } else { "  UNEXPECTED" }
                );
            }
            let m = &l.summary;
            let _ = writeln!(s, "\n  {}/{} decisions as expected", m.correct, m.scenarios);
            let _ = writeln!(s, "  per-branch CI landed a broken main in {} scenarios; pytxo land refused {} of them", m.naive_landed_broken, m.pytxo_prevented);
            let _ = writeln!(s, "  false refusals {} · conflicts caught {} · stale refused {} · symbol collisions reported {}", m.false_refusals, m.conflicts_caught, m.stale_refused, m.symbol_collisions_reported);
        }
    }
    if let Some(o) = &r.overhead {
        let _ = writeln!(s, "\noverhead: gate cost without checks (median of 3)");
        let _ = writeln!(
            s,
            "  {:>6} {:>6} {:>12} {:>14}",
            "inputs", "files", "git merge", "pytxo land"
        );
        for row in &o.rows {
            let _ = writeln!(
                s,
                "  {:>6} {:>6} {:>10}ms {:>12}ms",
                row.inputs, row.files, row.git_merge_ms, row.pytxo_land_ms
            );
        }
    }
    if let Some(g) = &r.signal {
        let _ = writeln!(
            s,
            "\nsignal: {} files of {}, {} → {} bytes, {}% smaller scaffold",
            g.files, g.repo, g.original_bytes, g.scaffold_bytes, g.weighted_reduction_pct
        );
    }
    for note in &r.notes {
        let _ = writeln!(s, "\nnote: {note}");
    }
    s
}

/// Minimal owned temp directory (removed on drop) so the CLI binary does not
/// need `tempfile` at runtime.
mod tempdir {
    use std::path::{Path, PathBuf};

    pub struct Dir(PathBuf);

    impl Dir {
        pub fn new(prefix: &str) -> std::io::Result<Self> {
            let path =
                std::env::temp_dir().join(format!("{prefix}-{}", uuid::Uuid::new_v4().simple()));
            std::fs::create_dir_all(&path)?;
            Ok(Self(path))
        }

        pub fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "runs the full Pybench land suite (minutes); cargo test -p pytxo-land -- --ignored"]
    fn every_scenario_meets_its_expectation() {
        if Command::new("node").arg("--version").output().is_err() {
            return;
        }
        let report = run(&[Suite::Land], None, |_| {}).unwrap();
        let land = report.land.unwrap();
        for s in &land.scenarios {
            assert!(
                s.correct,
                "{} expected {} got {} ({})",
                s.id, s.expected, s.pytxo_decision, s.pytxo_layer
            );
        }
        assert_eq!(land.summary.naive_landed_broken, 4);
        assert_eq!(land.summary.pytxo_prevented, 4);
        assert_eq!(land.summary.false_refusals, 0);
    }
}
