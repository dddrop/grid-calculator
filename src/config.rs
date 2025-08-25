use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub base: BaseConfig,
    pub grid: GridConfig,
    pub position: PositionConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategies: Option<Vec<Strategy>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BaseConfig {
    pub initial_price: f64,
    pub grid_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GridConfig {
    pub levels: Vec<f64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PositionConfig {
    pub mode: String,
    pub base_size: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiplier: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Strategy {
    pub name: String,
    pub initial_price: f64,
    pub grid_type: String,
    pub levels: Vec<f64>,
    pub position_mode: String,
    pub base_size: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiplier: Option<f64>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        // Validate grid_type
        if !["fixed", "average"].contains(&self.base.grid_type.as_str()) {
            return Err(format!("Invalid grid_type: {}. Must be 'fixed' or 'average'", self.base.grid_type));
        }

        // Validate position mode
        if !["fixed", "current-multiple", "increment-multiple"].contains(&self.position.mode.as_str()) {
            return Err(format!("Invalid position mode: {}. Must be 'fixed', 'current-multiple', or 'increment-multiple'", self.position.mode));
        }

        // Validate multiplier is present when needed
        if self.position.mode.contains("multiple") && self.position.multiplier.is_none() {
            return Err("Multiplier is required for multiple position modes".to_string());
        }

        // Validate levels
        if self.grid.levels.is_empty() {
            return Err("Grid levels cannot be empty".to_string());
        }

        for level in &self.grid.levels {
            if *level <= 0.0 || *level >= 100.0 {
                return Err(format!("Invalid grid level: {}. Must be between 0 and 100", level));
            }
        }

        Ok(())
    }
}

impl Strategy {
    pub fn validate(&self) -> Result<(), String> {
        // Validate grid_type
        if !["fixed", "average"].contains(&self.grid_type.as_str()) {
            return Err(format!("Invalid grid_type in strategy '{}': {}. Must be 'fixed' or 'average'", self.name, self.grid_type));
        }

        // Validate position mode
        if !["fixed", "current-multiple", "increment-multiple"].contains(&self.position_mode.as_str()) {
            return Err(format!("Invalid position mode in strategy '{}': {}. Must be 'fixed', 'current-multiple', or 'increment-multiple'", self.name, self.position_mode));
        }

        // Validate multiplier is present when needed
        if self.position_mode.contains("multiple") && self.multiplier.is_none() {
            return Err(format!("Multiplier is required for strategy '{}' with position mode '{}'", self.name, self.position_mode));
        }

        // Validate levels
        if self.levels.is_empty() {
            return Err(format!("Grid levels cannot be empty for strategy '{}'", self.name));
        }

        for level in &self.levels {
            if *level <= 0.0 || *level >= 100.0 {
                return Err(format!("Invalid grid level in strategy '{}': {}. Must be between 0 and 100", self.name, level));
            }
        }

        Ok(())
    }

    pub fn to_config(&self) -> Config {
        Config {
            base: BaseConfig {
                initial_price: self.initial_price,
                grid_type: self.grid_type.clone(),
            },
            grid: GridConfig {
                levels: self.levels.clone(),
            },
            position: PositionConfig {
                mode: self.position_mode.clone(),
                base_size: self.base_size,
                multiplier: self.multiplier,
            },
            strategies: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = Config {
            base: BaseConfig {
                initial_price: 100.0,
                grid_type: "fixed".to_string(),
            },
            grid: GridConfig {
                levels: vec![1.0, 2.0, 3.0],
            },
            position: PositionConfig {
                mode: "fixed".to_string(),
                base_size: 100.0,
                multiplier: None,
            },
            strategies: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_grid_type() {
        let config = Config {
            base: BaseConfig {
                initial_price: 100.0,
                grid_type: "invalid".to_string(),
            },
            grid: GridConfig {
                levels: vec![1.0, 2.0, 3.0],
            },
            position: PositionConfig {
                mode: "fixed".to_string(),
                base_size: 100.0,
                multiplier: None,
            },
            strategies: None,
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_missing_multiplier() {
        let config = Config {
            base: BaseConfig {
                initial_price: 100.0,
                grid_type: "fixed".to_string(),
            },
            grid: GridConfig {
                levels: vec![1.0, 2.0, 3.0],
            },
            position: PositionConfig {
                mode: "current-multiple".to_string(),
                base_size: 100.0,
                multiplier: None,
            },
            strategies: None,
        };

        assert!(config.validate().is_err());
    }
}