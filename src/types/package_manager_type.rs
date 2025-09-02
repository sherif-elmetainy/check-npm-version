use std::fmt::Display;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum PackageManagerType {
    Npm,
    Yarn,
    Pnpm,
}

impl Display for PackageManagerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageManagerType::Npm => write!(f, "npm"),
            PackageManagerType::Yarn => write!(f, "yarn"),
            PackageManagerType::Pnpm => write!(f, "pnpm"),
        }
    }
}