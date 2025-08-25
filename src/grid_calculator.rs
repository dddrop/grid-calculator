#[derive(Debug, Clone, Copy)]
pub enum GridType {
    Fixed,
    Average,
}

#[derive(Debug, Clone, Copy)]
pub enum PositionMode {
    Fixed,
    CurrentMultiple,
    IncrementMultiple,
}

#[derive(Debug, Clone)]
pub struct GridResult {
    pub grid_price: f64,
    pub position_size: f64,
    pub total_position: f64,
    pub average_price: f64,
    pub total_cost: f64,
}

pub struct GridCalculator {
    initial_price: f64,
    grid_type: GridType,
    position_mode: PositionMode,
    base_size: f64,
    multiplier: f64,
    
    // State tracking
    current_position: f64,
    total_cost: f64,
    average_price: f64,
    last_increment: f64,
    grid_history: Vec<GridResult>,
}

impl GridCalculator {
    pub fn new(
        initial_price: f64,
        grid_type: GridType,
        position_mode: PositionMode,
        base_size: f64,
        multiplier: f64,
    ) -> Self {
        Self {
            initial_price,
            grid_type,
            position_mode,
            base_size,
            multiplier,
            current_position: 0.0,
            total_cost: 0.0,
            average_price: initial_price,
            last_increment: 0.0,
            grid_history: Vec::new(),
        }
    }

    pub fn calculate_grid(&mut self, grid_percent: f64) -> GridResult {
        // Calculate grid price based on type
        let grid_price = match self.grid_type {
            GridType::Fixed => {
                // Grid relative to initial price
                self.initial_price * (1.0 - grid_percent / 100.0)
            }
            GridType::Average => {
                // Grid relative to current average price
                self.average_price * (1.0 - grid_percent / 100.0)
            }
        };

        // Calculate position size based on mode
        let position_size = match self.position_mode {
            PositionMode::Fixed => {
                // Fixed size for each grid
                self.base_size
            }
            PositionMode::CurrentMultiple => {
                // Multiple of current total position
                if self.current_position == 0.0 {
                    self.base_size
                } else {
                    self.current_position * self.multiplier
                }
            }
            PositionMode::IncrementMultiple => {
                // Multiple of last increment
                if self.last_increment == 0.0 {
                    self.base_size
                } else {
                    self.last_increment * self.multiplier
                }
            }
        };

        // Update state
        self.last_increment = position_size;
        self.current_position += position_size;
        self.total_cost += position_size * grid_price;
        
        // Calculate new average price
        if self.current_position > 0.0 {
            self.average_price = self.total_cost / self.current_position;
        }

        let result = GridResult {
            grid_price,
            position_size,
            total_position: self.current_position,
            average_price: self.average_price,
            total_cost: self.total_cost,
        };

        self.grid_history.push(result.clone());
        result
    }

    pub fn reset(&mut self) {
        self.current_position = 0.0;
        self.total_cost = 0.0;
        self.average_price = self.initial_price;
        self.last_increment = 0.0;
        self.grid_history.clear();
    }

    pub fn get_history(&self) -> &[GridResult] {
        &self.grid_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_grid_fixed_size() {
        let mut calc = GridCalculator::new(
            100.0,
            GridType::Fixed,
            PositionMode::Fixed,
            100.0,
            1.0,
        );

        let result1 = calc.calculate_grid(1.0);
        assert_eq!(result1.grid_price, 99.0);
        assert_eq!(result1.position_size, 100.0);
        assert_eq!(result1.total_position, 100.0);

        let result2 = calc.calculate_grid(2.0);
        assert_eq!(result2.grid_price, 98.0);
        assert_eq!(result2.position_size, 100.0);
        assert_eq!(result2.total_position, 200.0);
    }

    #[test]
    fn test_average_grid_fixed_size() {
        let mut calc = GridCalculator::new(
            100.0,
            GridType::Average,
            PositionMode::Fixed,
            100.0,
            1.0,
        );

        let result1 = calc.calculate_grid(1.0);
        assert_eq!(result1.grid_price, 99.0);
        assert_eq!(result1.position_size, 100.0);
        assert_eq!(result1.average_price, 99.0);

        // Second grid is relative to new average price (99.0)
        let result2 = calc.calculate_grid(1.0);
        assert_eq!(result2.grid_price, 99.0 * 0.99);
        assert_eq!(result2.position_size, 100.0);
    }

    #[test]
    fn test_current_multiple_mode() {
        let mut calc = GridCalculator::new(
            100.0,
            GridType::Fixed,
            PositionMode::CurrentMultiple,
            100.0,
            2.0,
        );

        let result1 = calc.calculate_grid(1.0);
        assert_eq!(result1.position_size, 100.0);
        assert_eq!(result1.total_position, 100.0);

        let result2 = calc.calculate_grid(2.0);
        assert_eq!(result2.position_size, 200.0); // 100 * 2
        assert_eq!(result2.total_position, 300.0);

        let result3 = calc.calculate_grid(3.0);
        assert_eq!(result3.position_size, 600.0); // 300 * 2
        assert_eq!(result3.total_position, 900.0);
    }

    #[test]
    fn test_increment_multiple_mode() {
        let mut calc = GridCalculator::new(
            100.0,
            GridType::Fixed,
            PositionMode::IncrementMultiple,
            100.0,
            1.5,
        );

        let result1 = calc.calculate_grid(1.0);
        assert_eq!(result1.position_size, 100.0);

        let result2 = calc.calculate_grid(2.0);
        assert_eq!(result2.position_size, 150.0); // 100 * 1.5

        let result3 = calc.calculate_grid(3.0);
        assert_eq!(result3.position_size, 225.0); // 150 * 1.5
    }
}