use std::collections::{BTreeMap};
use std::{env, fs};
use std::path::{Path, PathBuf};
use std::time::{Duration};
use chrono::{DateTime, Utc};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use semver::{Version, VersionReq};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PackageJson {
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
    #[serde(rename = "devDependencies", default)]
    dev_dependencies: BTreeMap<String, String>,
    #[serde(rename = "peerDependencies", default)]
    peer_dependencies: BTreeMap<String, String>,
}
//static TIME_FORMAT: LazyLock<Vec<time::format_description::FormatItem>> = LazyLock::new(|| time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second].[millisecond]Z").unwrap());
#[derive(Debug, Deserialize)]
struct PackageInfo {
    time: BTreeMap<String, String>,
}

#[derive(Debug)]
struct PackageReportRow {
    name: String,
    declared: String,
    latest: Option<Version>,
    latest_time: Option<DateTime<Utc>>,
    latest_same_major: Option<Version>,
    latest_same_major_time: Option<DateTime<Utc>>,
    latest_same_minor: Option<Version>,
    latest_same_minor_time: Option<DateTime<Utc>>,
    latest_satisfying: Option<Version>,
    latest_satisfying_time: Option<DateTime<Utc>>,
}

// Pretty ANSI table printer for PackageReportRow
fn print_report_table(rows: &[PackageReportRow]) {
    // ANSI helpers
    const RESET: &str = "\x1b[0m";
    fn bold(s: &str) -> String { format!("\x1b[1m{}{}", s, RESET) }
    fn color(code: u8, s: &str) -> String { format!("\x1b[{}m{}{}", code, s, RESET) }
    fn bold_color(code: u8, s: &str) -> String { format!("\x1b[1;{}m{}{}", code, s, RESET) }

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
                        if nc == 'm' { break; }
                    }
                    continue;
                }
            }
            out.push(c);
        }
        out
    }

    // Format a version + date cell; date on next line, dim gray
    fn fmt_ver(ver: &Option<Version>, time: &Option<DateTime<Utc>>, style: impl Fn(&str) -> String) -> String {
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
        row.push(fmt_ver(&r.latest_same_major, &r.latest_same_major_time, |s| bold_color(34, s)));

        // Latest (same minor)
        row.push(fmt_ver(&r.latest_same_minor, &r.latest_same_minor_time, |s| bold_color(35, s)));

        // Latest (satisfying)
        row.push(fmt_ver(&r.latest_satisfying, &r.latest_satisfying_time, |s| bold_color(33, s)));

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
            if i + 1 < widths.len() { s.push(mid); }
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


fn resolve_package_path() -> Result<PathBuf, std::io::Error> {
    let mut arg = env::args().nth(1).unwrap_or_else(String::new);

    if arg.trim().is_empty() {
        arg = ".".to_string()
    }
    let path = Path::new(&arg);
    let metadata = path.metadata().map_err(
        |e| std::io::Error::new(e.kind(), format!("path {} is not valid: {}", arg, e.to_string()))
    )?;

    let file_path = if metadata.is_dir() {
        let path = path.join("package.json");
        let metadata = path.metadata().map_err(
            |e| std::io::Error::new(e.kind(), format!("path {} is not valid: {}", path.to_str().unwrap(), e.to_string()))
        )?;
        if !metadata.is_file() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("path {} is not valid: {}", path.to_str().unwrap(), "package.json is not a file")))
        }
        path
    } else {
        path.to_path_buf()
    };

    Ok(file_path)
}

fn build_http_client() -> Result<Client, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str("check-npm-version/1.0 (+https://github.com/sherif-elmetainy/check-npm-version)")?,
    );
    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(20))
        .build()?;
    Ok(client)
}

async fn get_package_info(client: &Client, package_name: &str) -> Result<PackageInfo, Box<dyn std::error::Error>> {
    let request= client.get(format!("https://registry.npmjs.org/{}", package_name)).build()?;

    let response = client.execute(request).await?;
    let text = response.text().await?;
    let package_info: PackageInfo = serde_json::from_str(&text)?;
    Ok(package_info)
}

async fn process_package(client: &Client, package_name: String, declared_version: String) -> Result<PackageReportRow, Box<dyn std::error::Error>> {
    let mut info = get_package_info(client, package_name.as_str()).await?;


    info.time.remove("created");
    info.time.remove("modified");

    let mut row = PackageReportRow {
        name: package_name,
        declared: declared_version,
        latest: None,
        latest_time: None,
        latest_same_major: None,
        latest_same_major_time: None,
        latest_same_minor: None,
        latest_same_minor_time: None,
        latest_satisfying: None,
        latest_satisfying_time: None,
    };

    let declared_req = VersionReq::parse(row.declared.as_str()).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}: {}", row.declared, e.to_string())))?;
    let declared_version = Version::parse(row.declared.as_str());

    
    for (version, time) in std::mem::take(&mut info.time) {
        let parsed_version = Version::parse(&version).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}: {}", version, e.to_string())))?;
        let time = DateTime::parse_from_rfc3339(time.as_str()).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}: {}", time, e.to_string())))?.to_utc();

        if row.latest.is_none() || parsed_version.ge(row.latest.as_ref().unwrap()) {
            row.latest = Some(parsed_version.clone());
            row.latest_time = Some(time)
        }

        if declared_req.matches(&parsed_version) {
            if row.latest_satisfying.is_none() || parsed_version.ge(row.latest_satisfying.as_ref().unwrap()) {
                row.latest_satisfying = Some(parsed_version.clone());
                row.latest_satisfying_time = Some(time)
            }
        }

        if declared_version.as_ref().is_ok() {
            let declared = declared_version.as_ref().unwrap();
            if declared.major == parsed_version.major {
                if row.latest_same_major.is_none() || parsed_version.ge(row.latest_same_major.as_ref().unwrap()) {
                    row.latest_same_major = Some(parsed_version.clone());
                    row.latest_same_major_time = Some(time)
                }
                if declared.minor == parsed_version.minor {
                    if row.latest_same_minor.is_none() || parsed_version.ge(row.latest_same_minor.as_ref().unwrap()) {
                        row.latest_same_minor = Some(parsed_version);
                        row.latest_same_minor_time = Some(time)
                    }
                }
            }
        };
    }
    Ok(row)


}

async fn process_package_json(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 2) Read and parse package.json
    let pkg_json_text = fs::read_to_string(&path)?;
    let mut pkg: PackageJson = serde_json::from_str(&pkg_json_text)?;
    let client = build_http_client()?;
    let mut result : Vec<PackageReportRow> = Vec::new();

    // loop through dependencies
    for (name, version) in std::mem::take(&mut pkg.dependencies) {
        let row = process_package(&client, name, version).await?;
        result.push(row);
    }

    for (name, version) in std::mem::take(&mut pkg.dev_dependencies) {
        let row = process_package(&client, name, version).await?;
        result.push(row);
    }

    for (name, version) in std::mem::take(&mut pkg.peer_dependencies) {
        let row = process_package(&client, name, version).await?;
        result.push(row);
    }

    // Render the collected rows as a pretty, colored table
    print_report_table(&result);

    Ok(())
}


#[tokio::main]
async fn main() {

    let path = resolve_package_path();
    match path {
        Ok(file) => process_package_json(file.clone()).await.unwrap_or_else(
            |e| println!("An error occurred while processing file {}: {}", file.to_str().unwrap(), e)
        ),
        Err(e) => println!("{}", e)
    }
}
