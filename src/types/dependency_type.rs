use std::fmt::Display;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub enum DependencyType {
    Normal,
    Dev,
    Peer,
    Optional,
}

impl Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::Normal => write!(f, "Normal Dependencies"),
            DependencyType::Dev => write!(f, "Development Dependencies"),
            DependencyType::Peer => write!(f, "Peer Dependencies"),
            DependencyType::Optional => write!(f, "Optional Dependencies"),
        }
    }
}
