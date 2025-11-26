use super::{format_number, nice_ticks, Tick};

/// Linear scale for quantitative data
#[derive(Debug, Clone)]
pub struct LinearScale {
    domain_min: f64,
    domain_max: f64,
    range_min: f64,
    range_max: f64,
    clamp: bool,
}

impl LinearScale {
    pub fn new(domain: (f64, f64), range: (f64, f64)) -> Self {
        Self {
            domain_min: domain.0,
            domain_max: domain.1,
            range_min: range.0,
            range_max: range.1,
            clamp: false,
        }
    }

    /// Create scale with nice domain boundaries
    pub fn nice(mut self) -> Self {
        let ticks = nice_ticks(self.domain_min, self.domain_max, 10);
        if let (Some(&first), Some(&last)) = (ticks.first(), ticks.last()) {
            self.domain_min = first;
            self.domain_max = last;
        }
        self
    }

    /// Include zero in the domain
    pub fn zero(mut self) -> Self {
        if self.domain_min > 0.0 {
            self.domain_min = 0.0;
        }
        if self.domain_max < 0.0 {
            self.domain_max = 0.0;
        }
        self
    }

    /// Clamp output to range
    pub fn clamp(mut self, clamp: bool) -> Self {
        self.clamp = clamp;
        self
    }

    /// Map domain value to range value
    pub fn scale(&self, value: f64) -> f64 {
        let domain_span = self.domain_max - self.domain_min;
        if domain_span == 0.0 {
            return self.range_min;
        }

        let t = (value - self.domain_min) / domain_span;
        let result = self.range_min + t * (self.range_max - self.range_min);

        if self.clamp {
            result.clamp(self.range_min.min(self.range_max), self.range_min.max(self.range_max))
        } else {
            result
        }
    }

    /// Map range value back to domain value
    pub fn invert(&self, value: f64) -> f64 {
        let range_span = self.range_max - self.range_min;
        if range_span == 0.0 {
            return self.domain_min;
        }

        let t = (value - self.range_min) / range_span;
        self.domain_min + t * (self.domain_max - self.domain_min)
    }

    /// Generate tick values
    pub fn ticks(&self, count: usize) -> Vec<Tick> {
        nice_ticks(self.domain_min, self.domain_max, count)
            .into_iter()
            .map(|value| Tick {
                value,
                label: format_number(value),
            })
            .collect()
    }

    /// Get domain
    pub fn domain(&self) -> (f64, f64) {
        (self.domain_min, self.domain_max)
    }

    /// Get range
    pub fn range(&self) -> (f64, f64) {
        (self.range_min, self.range_max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_scale() {
        let scale = LinearScale::new((0.0, 100.0), (0.0, 500.0));
        assert_eq!(scale.scale(0.0), 0.0);
        assert_eq!(scale.scale(50.0), 250.0);
        assert_eq!(scale.scale(100.0), 500.0);
    }

    #[test]
    fn test_linear_scale_invert() {
        let scale = LinearScale::new((0.0, 100.0), (0.0, 500.0));
        assert_eq!(scale.invert(0.0), 0.0);
        assert_eq!(scale.invert(250.0), 50.0);
        assert_eq!(scale.invert(500.0), 100.0);
    }

    #[test]
    fn test_linear_scale_zero() {
        let scale = LinearScale::new((10.0, 100.0), (0.0, 500.0)).zero();
        assert_eq!(scale.domain(), (0.0, 100.0));
    }
}
