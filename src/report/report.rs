use crate::report::table::print_report_table;
use crate::types::PackageJsonReport;

pub fn print_report(report: &PackageJsonReport) {
    let mut separator_line = false;
    // Print dependency table
    for (dependency_type, rows) in &report.dependencies {
        if separator_line {
            println!();
        }
        if rows.is_empty() {
            continue;
        }
        separator_line = true;
        print_report_table(dependency_type.to_string().as_str(), rows);
    }
}
