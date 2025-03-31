#[derive(Clone)]
pub enum Guard {
    NumberOfAllowedCalls(u16),
    // ... other variants
}

impl Guard {
    pub fn check(&self) -> bool {
        match self {
            Guard::NumberOfAllowedCalls(remaining) => *remaining > 0,
            // handle other variants
        }
    }

    pub fn update(&mut self) -> () {
        match self {
            Guard::NumberOfAllowedCalls(remaining) => {
                if *remaining > 0 {
                    *remaining -= 1;
                }
            }
            // handle other variants
        }
    }
}