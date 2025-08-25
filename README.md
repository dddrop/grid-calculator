# Grid Trading Calculator

A calculator supporting multiple grid trading strategies, written in Rust.

## Features

### Grid Types

1. **Fixed Price Grid (Fixed)**: All grids are calculated relative to the initial price.
2. **Average Price Grid (Average)**: Each grid is calculated relative to the current average price.

### Position Sizing Modes

1. **Fixed Amount (Fixed)**: Each grid uses a fixed position size.
2. **Current Position Multiple (CurrentMultiple)**: Position size is a multiple of the current total position.
3. **Increment Multiple (IncrementMultiple)**: Position size is a multiple of the last increment.
