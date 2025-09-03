use crate::log_debug;
use crate::report::table::print_report_table;
use crate::types::PackageJsonReport;

pub fn print_report(report: &PackageJsonReport, skip_latest: bool) {
    let mut separator_line = false;
    // Print dependency table
    for (dependency_type, rows) in &report.dependencies {
        if separator_line {
            println!();
        }
        let filtered = if skip_latest {
            log_debug!("skipping latest");
            rows.iter().filter(
                |p| !p.is_latest() 
            ).collect::<Vec<_>>()
        } else {
            log_debug!("not skipping latest");
            rows.iter().filter(
                |p| !p.is_latest()
            ).collect::<Vec<_>>()
        };
        if filtered.is_empty() {
            continue;
        }
        separator_line = true;
        print_report_table(dependency_type.to_string().as_str(), &filtered);
    }
}
