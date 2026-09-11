# check-npm-version

A small, fast CLI that tells you how out of date the dependencies in a
`package.json` are — and can upgrade them for you with npm, yarn or pnpm.

Written in Rust, with no Node.js runtime required to produce the report.

## What it does

For every entry in `dependencies`, `devDependencies`, `peerDependencies` and
`optionalDependencies`, it queries the npm registry and reports four candidate
versions, each with its publishing date:

| Column                  | Meaning                                                        |
|-------------------------|----------------------------------------------------------------|
| **Latest**              | Newest published version, ignoring the declared range entirely |
| **Latest (same major)** | Newest version with the same major as the declared version     |
| **Latest (same minor)** | Newest version with the same major *and* minor                 |
| **Latest (satisfying)** | Newest version that satisfies the declared semver range        |

Pre-release versions are skipped unless the declared version is itself a
pre-release. Versions the registry lists that aren't valid semver (older
packages have plenty, e.g. `3.0.0beta1`) are ignored with a warning.

## Installation

Requires a Rust toolchain (edition 2024).

```bash
git clone https://github.com/sherif-elmetainy/check-npm-version.git
cd check-npm-version
cargo build --release
```

The binary lands in `target/release/check-npm-version`. Copy it anywhere on your
`PATH`, or use the bundled helper, which builds and installs into `~/.local/bin`
(`~/.local` on Windows/Msys):

```bash
./build.sh
```

## Usage

```
check-npm-version [OPTIONS] <COMMAND>

Commands:
  report   Print a report of the package.json file
  upgrade  Upgrade the packages in the package.json file
  version  Print version information
  help     Print help for a given subcommand

Options:
  -l, --log-level <LEVEL>  off | error | warn | info | debug | trace  [default: warn]
```

### report

```bash
# current directory
check-npm-version report

# an explicit directory or file
check-npm-version report -p ../my-app
check-npm-version report -p ../my-app/package.json

# quiet: suppress the "ignoring invalid version" warnings
check-npm-version -l off report
```

```
Normal Dependencies:
┌─────────┬──────────┬────────────┬─────────────────────┬─────────────────────┬─────────────────────┐
│ Package │ Declared │ Latest     │ Latest (same major) │ Latest (same minor) │ Latest (satisfying) │
├─────────┼──────────┼────────────┼─────────────────────┼─────────────────────┼─────────────────────┤
│ express │ ^4.18.2  │ 5.2.1      │ 4.22.2              │ 4.18.3              │ 4.22.2              │
│         │          │ 2025-12-01 │ 2026-05-11          │ 2024-02-29          │ 2026-05-11          │
└─────────┴──────────┴────────────┴─────────────────────┴─────────────────────┴─────────────────────┘

Development Dependencies:
┌────────────┬──────────┬────────────┬─────────────────────┬─────────────────────┬─────────────────────┐
│ Package    │ Declared │ Latest     │ Latest (same major) │ Latest (same minor) │ Latest (satisfying) │
├────────────┼──────────┼────────────┼─────────────────────┼─────────────────────┼─────────────────────┤
│ typescript │ ~5.3.3   │ 7.0.2      │ 5.9.3               │ 5.3.3               │ 5.3.3               │
│            │          │ 2026-07-08 │ 2025-09-30          │ 2023-12-06          │ 2023-12-06          │
└────────────┴──────────┴────────────┴─────────────────────┴─────────────────────┴─────────────────────┘
```

Packages already pinned at the newest published version are left out of the
table, so an empty report means everything is current.

Options:

| Flag                | Description                                                          |
|---------------------|----------------------------------------------------------------------|
| `-p, --path <PATH>` | Directory containing `package.json`, or the file itself. Default `.` |
| `-s, --skip-latest` | Omit packages already at the latest version                          |

### upgrade

Runs your package manager to install the newer versions and rewrite
`package.json`. Nothing is edited by hand — the `install` command does the work,
so lockfiles and `node_modules` stay consistent.

```bash
# bump within the current major (default)
check-npm-version upgrade

# bump only the patch level within the current minor
check-npm-version upgrade -t minor

# jump to the newest published version, majors included
check-npm-version upgrade -t latest

# force a particular package manager
check-npm-version upgrade -m pnpm
```

Options:

| Flag                | Description                                                          |
|---------------------|----------------------------------------------------------------------|
| `-p, --path <PATH>` | Directory containing `package.json`, or the file itself. Default `.` |
| `-t, --type <TYPE>` | `major` (default), `minor`, or `latest`                              |
| `-m <MANAGER>`      | `npm`, `yarn`, or `pnpm`                                             |

The `~`/`^` prefix on each declared range is preserved: `^4.18.2` upgraded with
`-t major` becomes `^4.22.2`, not a bare `4.22.2`.

When `-m` is omitted, the package manager is detected from the lockfile in the
target directory — `yarn.lock` → yarn, `pnpm-lock.yaml` → pnpm,
`package-lock.json` → npm, falling back to npm. Each dependency type is
installed in its own invocation with the appropriate save flag
(`--save-dev`, `--save-peer`, and so on).

## Private registries

`.npmrc` files are read the way npm reads them, so scoped packages hosted on a
private registry work without extra configuration. Files are loaded in
increasing order of precedence:

1. The built-in `npmrc` next to the `npm` executable found on `PATH`
2. `$HOME/.npmrc`
3. The `.npmrc` in the project directory

Recognized keys are `registry`, `@scope:registry`, `_authToken`, and
`//host/path/:_authToken`. Auth tokens are sent as a bearer token to the
matching registry. The default registry is `https://registry.npmjs.org/`.

## Caching

Registry responses are cached on disk with HTTP cache semantics, so repeated
runs are fast and gentle on the registry. The cache lives under the platform
cache directory in `code-art/check-npm-version` (with an extra `cache`
subdirectory on Windows); delete it to force fresh requests.

## Project layout

```
src/
├── cli/        Argument parsing (clap) and command dispatch
├── processor/  package.json loading, .npmrc parsing, registry lookups,
│               and version resolution
├── report/     ANSI table rendering
├── upgrade/    Package manager detection and install invocation
├── types/      Shared data types
└── util/       HTTP client with caching, logging macros, Defer guard
```

## License

[MIT](LICENSE)
