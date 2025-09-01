use chrono::{DateTime, Utc};
use semver::Version;

// Report row describing versions for a package
#[derive(Debug)]
pub struct PackageReportRow {
    pub name: String,
    pub declared: String,
    pub latest: Option<Version>,
    pub latest_time: Option<DateTime<Utc>>,
    pub latest_same_major: Option<Version>,
    pub latest_same_major_time: Option<DateTime<Utc>>,
    pub latest_same_minor: Option<Version>,
    pub latest_same_minor_time: Option<DateTime<Utc>>,
    pub latest_satisfying: Option<Version>,
    pub latest_satisfying_time: Option<DateTime<Utc>>,
}

// Pretty ANSI table printer for PackageReportRow
pub fn print_report_table(rows: &[PackageReportRow]) {
    // ANSI helpers
    const RESET: &str = "\x1b[0m";
    fn bold(s: &str) -> String {
        format!("\x1b[1m{}{}", s, RESET)
    }
    fn color(code: u8, s: &str) -> String {
        format!("\x1b[{}m{}{}", code, s, RESET)
    }
    fn bold_color(code: u8, s: &str) -> String {
        format!("\x1b[1;{}m{}{}", code, s, RESET)
    }

    // Strip ANSI codes to compute widths
    fn strip_ansi(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                if matches!(chars.peek(), Some('[')) {
                    // consume '['
                    let _ = chars.next();
                    // consume until 'm' or end
                    while let Some(nc) = chars.next() {
                        if nc == 'm' {
                            break;
                        }
                    }
                    continue;
                }
            }
            out.push(c);
        }
        out
    }

    // Format a version + date cell; date on next line, dim gray
    fn fmt_ver(
        ver: &Option<Version>,
        time: &Option<DateTime<Utc>>,
        style: impl Fn(&str) -> String,
    ) -> String {
        match (ver, time) {
            (Some(v), Some(t)) => {
                let date = t.format("%Y-%m-%d").to_string();
                let ver_s = style(&v.to_string());
                let date_s = color(90, &date); // bright black (gray)
                format!("{ver_s}\n{date_s}")
            }
            (Some(v), None) => style(&v.to_string()),
            _ => color(90, "-"),
        }
    }

    // Build header and rows with styling
    let header = vec![
        bold_color(36, "Package"),                  // cyan
        bold_color(37, "Declared"),                 // white
        bold_color(32, "Latest"),                   // green
        bold_color(34, "Latest (same major)"),      // blue
        bold_color(35, "Latest (same minor)"),      // magenta
        bold_color(33, "Latest (satisfying)"),      // yellow
    ];

    let mut table: Vec<Vec<String>> = Vec::with_capacity(rows.len() + 1);
    table.push(header);

    for r in rows {
        let mut row: Vec<String> = Vec::with_capacity(6);
        row.push(bold(&r.name)); // package name

        // Declared
        row.push(bold_color(37, r.declared.as_str()));

        // Latest
        row.push(fmt_ver(&r.latest, &r.latest_time, |s| bold_color(32, s)));

        // Latest (same major)
        row.push(fmt_ver(
            &r.latest_same_major,
            &r.latest_same_major_time,
            |s| bold_color(34, s),
        ));

        // Latest (same minor)
        row.push(fmt_ver(
            &r.latest_same_minor,
            &r.latest_same_minor_time,
            |s| bold_color(35, s),
        ));

        // Latest (satisfying)
        row.push(fmt_ver(
            &r.latest_satisfying,
            &r.latest_satisfying_time,
            |s| bold_color(33, s),
        ));

        table.push(row);
    }

    // Compute column widths (based on longest line in each cell, sans ANSI)
    let col_count = table.first().map(|r| r.len()).unwrap_or(0);
    let mut col_widths = vec![0usize; col_count];

    for row in &table {
        for (i, cell) in row.iter().enumerate() {
            let width = strip_ansi(cell)
                .lines()
                .map(|l| l.chars().count())
                .max()
                .unwrap_or(0);
            if width > col_widths[i] {
                col_widths[i] = width;
            }
        }
    }

    // Helpers to draw borders
    fn line_with_joints(left: char, mid: char, right: char, horiz: char, widths: &[usize]) -> String {
        let mut s = String::new();
        s.push(left);
        for (i, w) in widths.iter().enumerate() {
            s.push_str(&std::iter::repeat(horiz).take(w + 2).collect::<String>());
            if i + 1 < widths.len() {
                s.push(mid);
            }
        }
        s.push(right);
        s
    }

    let top = line_with_joints('┌', '┬', '┐', '─', &col_widths);
    let mid = line_with_joints('├', '┼', '┤', '─', &col_widths);
    let bot = line_with_joints('└', '┴', '┘', '─', &col_widths);

    // Render each row (handle multi-line cells)
    println!("{}", top);
    for (ri, row) in table.iter().enumerate() {
        // Split cells into lines
        let cell_lines: Vec<Vec<&str>> = row.iter().map(|c| c.split('\n').collect()).collect();
        let row_height = cell_lines.iter().map(|v| v.len()).max().unwrap_or(1);

        for line_idx in 0..row_height {
            let mut line = String::new();
            line.push('│');
            for (ci, _cell) in row.iter().enumerate() {
                let lines = &cell_lines[ci];
                let content = if line_idx < lines.len() { lines[line_idx] } else { "" };
                let width = col_widths[ci];
                let visible_len = strip_ansi(content).chars().count();
                let padding = width.saturating_sub(visible_len);
                line.push(' ');
                line.push_str(content);
                line.push_str(&" ".repeat(padding));
                line.push(' ');
                line.push('│');
            }
            println!("{}", line);
        }

        // Separator after header or after each data row; use mid for header/data split, bot at end
        if ri == 0 {
            println!("{}", mid);
        } else if ri + 1 == table.len() {
            println!("{}", bot);
        }
    }
}
