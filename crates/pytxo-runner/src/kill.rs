use pytxo_core::{PytxoError, Result};

pub fn kill_pid(pid: u32) -> Result<()> {
    if pid == 0 {
        return Ok(());
    }
    if cfg!(windows) {
        let status = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()
            .map_err(|e| PytxoError::Runner(format!("taskkill: {e}")))?;
        if !status.success() {
            return Err(PytxoError::Runner(format!("taskkill failed for pid {pid}")));
        }
    } else {
        let status = std::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .map_err(|e| PytxoError::Runner(format!("kill: {e}")))?;
        if !status.success() {
            let _ = std::process::Command::new("kill")
                .args(["-KILL", &pid.to_string()])
                .status();
        }
    }
    Ok(())
}

pub fn kill_pids(pids: &[u32]) -> Result<()> {
    for pid in pids {
        let _ = kill_pid(*pid);
    }
    Ok(())
}
