//! `pytxo accounts`: several vendor sign-ins per agent CLI. Each account is
//! the CLI's own config folder; sign-in runs the vendor's command in this
//! terminal, and Pytxo never reads what the CLI writes there.

use anyhow::{bail, Context, Result};
use pytxo_core::agent_accounts::{self as accounts, SessionState, DEFAULT_ACCOUNT};
use pytxo_core::AdeCliSpec;

#[derive(serde::Serialize)]
struct Row {
    cli: &'static str,
    cli_name: &'static str,
    account: String,
    active: bool,
    /// signed_in, signed_out, unknown or not_installed
    state: &'static str,
}

fn spec(cli: &str) -> Result<&'static AdeCliSpec> {
    let spec =
        pytxo_core::resolve_ade(cli).with_context(|| format!("unknown agent CLI {cli:?}"))?;
    if accounts::account_env(spec.id).is_none() {
        bail!(
            "{} keeps one sign-in per OS user; Pytxo can hold accounts for {}",
            spec.display_name,
            accounts::account_clis()
                .map(|s| s.id)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(spec)
}

pub fn list(json: bool) -> Result<()> {
    let root = accounts::default_root();
    let mut rows = Vec::new();
    for spec in accounts::account_clis() {
        let active = accounts::active(&root, spec.id);
        for account in
            std::iter::once(DEFAULT_ACCOUNT.to_string()).chain(accounts::list(&root, spec.id)?)
        {
            rows.push(Row {
                cli: spec.id,
                cli_name: spec.display_name,
                active: account == active,
                account,
                state: "not_installed",
            });
        }
    }
    std::thread::scope(|scope| {
        for row in &mut rows {
            let root = &root;
            scope.spawn(move || {
                let spec = pytxo_core::resolve_ade(row.cli).expect("registry id");
                if pytxo_core::ade_on_path(spec) {
                    row.state = accounts::probe_account(root, row.cli, &row.account)
                        .unwrap_or(SessionState::Unknown)
                        .as_str();
                }
            });
        }
    });
    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }
    for row in &rows {
        let state = match row.state {
            "signed_in" => "signed in",
            "signed_out" => "signed out",
            "not_installed" => "not installed",
            _ => "could not check",
        };
        let active = if row.active {
            "  · used for new work"
        } else {
            ""
        };
        println!("{:<14} {:<12} {state}{active}", row.cli_name, row.account);
    }
    println!();
    println!("Add one: pytxo accounts add claude work · pick it: pytxo accounts use claude work");
    println!(
        "Each account is the CLI's own folder under {}; Pytxo never reads its sign-in.",
        root.display()
    );
    Ok(())
}

pub fn add(cli: &str, name: &str) -> Result<()> {
    let spec = spec(cli)?;
    accounts::add(&accounts::default_root(), spec.id, name)?;
    println!("Added {} account {name}.", spec.display_name);
    if !pytxo_core::ade_on_path(spec) {
        println!(
            "{} is not installed. Install it, then run: pytxo accounts login {} {name}",
            spec.display_name, spec.id
        );
        return Ok(());
    }
    login(spec.id, Some(name))
}

pub fn login(cli: &str, name: Option<&str>) -> Result<()> {
    let spec = spec(cli)?;
    let name = name.unwrap_or(DEFAULT_ACCOUNT);
    let root = accounts::default_root();
    let env = accounts::env_for(&root, spec.id, name)?;
    let (executable, args) = accounts::sign_in_command(spec.id).expect("account CLIs sign in");
    println!(
        "Opening {} sign-in for account {name}. Finish it in the vendor's flow.",
        spec.display_name
    );
    let mut command = accounts::host_command(executable, args);
    if let Some((var, dir)) = env {
        command.env(var, dir);
    }
    let status = command
        .current_dir(accounts::host_auth_dir())
        .status()
        .with_context(|| format!("could not start {executable}"))?;
    if !status.success() {
        bail!("{} sign-in did not finish ({status})", spec.display_name);
    }
    match accounts::probe_account(&root, spec.id, name) {
        Some(SessionState::SignedIn) => println!("{} · {name}: signed in.", spec.display_name),
        _ => println!(
            "{} · {name}: sign-in not confirmed yet. Check with: pytxo accounts list",
            spec.display_name
        ),
    }
    Ok(())
}

pub fn use_account(cli: &str, name: &str) -> Result<()> {
    let spec = spec(cli)?;
    accounts::set_active(&accounts::default_root(), spec.id, name)?;
    println!("New work on {} uses account {name}.", spec.display_name);
    Ok(())
}

pub fn remove(cli: &str, name: &str) -> Result<()> {
    let spec = spec(cli)?;
    let root = accounts::default_root();
    // Sign out first so the vendor revokes the session and drops any keychain
    // entry keyed to this folder; the folder goes either way.
    if let (Ok(Some((var, dir))), true) = (
        accounts::env_for(&root, spec.id, name),
        pytxo_core::ade_on_path(spec),
    ) {
        let (executable, args) =
            accounts::sign_out_command(spec.id).expect("account CLIs sign out");
        let mut command = accounts::host_command(executable, args);
        let _ = command
            .env(var, dir)
            .current_dir(accounts::host_auth_dir())
            .status();
    }
    accounts::remove(&root, spec.id, name)?;
    println!("Removed {} account {name}.", spec.display_name);
    Ok(())
}
