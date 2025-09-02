#[derive(clap::ValueEnum, Clone, Debug)]
pub enum UpgradeType {
    Major,
    Minor,
    Latest,
}