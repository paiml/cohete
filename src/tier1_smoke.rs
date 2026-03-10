//! Tier 1: Smoke tests — verify all binaries exist and respond to --version/--help.
//!
//! This is the gate tier. If ANY binary fails, the entire run aborts.

use crate::runner;
use crate::types::{BinarySmoke, SmokeResult, BINARIES};

pub fn run() -> SmokeResult {
    let mut binaries = Vec::with_capacity(BINARIES.len());
    let mut all_pass = true;

    for def in BINARIES {
        let exists = std::path::Path::new(def.path).exists();
        let executable = exists && runner::is_executable(def.path);

        let (version, version_ok) = if executable {
            let result = runner::run(def.path, &["--version"]);
            if result.success {
                let ver = if result.stdout.is_empty() {
                    result.stderr.lines().next().map(String::from)
                } else {
                    result.stdout.lines().next().map(String::from)
                };
                (ver, true)
            } else {
                eprintln!("  FAIL: {} --version exited {}", def.name, result.exit_code.unwrap_or(-1));
                (None, false)
            }
        } else {
            if exists {
                eprintln!("  FAIL: {} not executable at {}", def.name, def.path);
            } else {
                eprintln!("  FAIL: {} not found at {}", def.name, def.path);
            }
            (None, false)
        };

        let help_ok = if executable {
            let result = runner::run(def.path, &["--help"]);
            result.success
        } else {
            false
        };

        let pass = exists && executable && version_ok && help_ok;
        if !pass {
            all_pass = false;
        }

        if let Some(ref v) = version {
            eprintln!("  {}: {} {}", if pass { "OK" } else { "FAIL" }, def.name, v);
        }

        binaries.push(BinarySmoke {
            name: def.name.to_string(),
            path: def.path.to_string(),
            exists,
            executable,
            version,
            help_ok,
        });
    }

    SmokeResult {
        tier: 1,
        pass: all_pass,
        binaries,
    }
}
