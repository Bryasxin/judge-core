use std::{collections::HashMap, str::FromStr};

#[derive(Debug)]
#[allow(unused)]
pub struct CpuStats {
    pub usage_usec: u64,
    pub user_usec: u64,
    pub system_usec: u64,
}

impl FromStr for CpuStats {
    type Err = ParseCpuStatsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut stats = HashMap::new();

        for line in s.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 2 {
                return Err(Self::Err::InvalidRow(line.to_string()));
            }

            match parts[1].parse::<u64>() {
                Ok(value) => stats.insert(parts[0], value),
                Err(_) => return Err(Self::Err::InvalidNumber(parts[1].to_string())),
            };
        }

        let usage_usec = *stats
            .get("usage_usec")
            .ok_or(Self::Err::MissingImportantField("usage_usec"))?;
        let user_usec = *stats
            .get("user_usec")
            .ok_or(Self::Err::MissingImportantField("user_usec"))?;
        let system_usec = *stats
            .get("system_usec")
            .ok_or(Self::Err::MissingImportantField("system_usec"))?;

        Ok(Self {
            usage_usec,
            user_usec,
            system_usec,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseCpuStatsError {
    #[error("Invalid number: \"{0}\"")]
    InvalidNumber(String),
    #[error("Invalid row: \"{0}\"")]
    InvalidRow(String),
    #[error("Missing important field: \"{0}\"")]
    MissingImportantField(&'static str),
}
