use crate::cli::Cli;
use crate::commands::{opts_from, resolve_snapshot};
use crate::diff;
use crate::output::print_json;
use anyhow::Result;

pub fn run(cli: &Cli, left: &str, right: &str) -> Result<i32> {
    let opts = opts_from(cli);
    let a = resolve_snapshot(left)?;
    let b = resolve_snapshot(right)?;
    let report = diff::diff(&a, &b);

    if opts.json {
        print_json(opts, &report)?;
    } else if !opts.quiet {
        println!("diff {} → {}", report.left_id, report.right_id);
        if report.changes.is_empty() {
            println!("  (no changes)");
        }
        for c in &report.changes {
            match c.change_type.as_str() {
                "added" => println!("  + {} {}", c.field, c.after.as_deref().unwrap_or("")),
                "removed" => println!("  - {} {}", c.field, c.before.as_deref().unwrap_or("")),
                _ => println!(
                    "  ~ {} {:?} → {:?}",
                    c.field, c.before, c.after
                ),
            }
        }
    }
    Ok(0)
}
