use super::Tick;

/// Band scale for categorical data with width
/// Used for bar charts where each category gets a band of space
#[derive(Debug, Clone)]
pub struct BandScale {
    domain: Vec<String>,
    range_min: f64,
    range_max: f64,
    padding_inner: f64,
    padding_outer: f64,
}

impl BandScale {
    pub fn new(domain: Vec<String>, range: (f64, f64)) -> Self {
        Self {
            domain,
            range_min: range.0,
            range_max: range.1,
            padding_inner: 0.1,
            padding_outer: 0.1,
        }
    }

    /// Set inner padding (between bands) as fraction of step
    pub fn padding_inner(mut self, padding: f64) -> Self {
        self.padding_inner = padding.clamp(0.0, 1.0);
        self
    }

    /// Set outer padding (at edges) as fraction of step
    pub fn padding_outer(mut self, padding: f64) -> Self {
        self.padding_outer = padding.clamp(0.0, 1.0);
        self
    }

    /// Set both inner and outer padding
    pub fn padding(self, padding: f64) -> Self {
        self.padding_inner(padding).padding_outer(padding)
    }

    /// Get the step size (band + padding)
    pub fn step(&self) -> f64 {
        let n = self.domain.len();
        if n == 0 {
            return 0.0;
        }

        let range_span = (self.range_max - self.range_min).abs();
        range_span / (n as f64 + self.padding_outer * 2.0 - self.padding_inner)
    }

    /// Get the bandwidth (just the band, without padding)
    pub fn bandwidth(&self) -> f64 {
        self.step() * (1.0 - self.padding_inner)
    }

    /// Map category to start position of band
    pub fn scale(&self, value: &str) -> Option<f64> {
        let index = self.domain.iter().position(|v| v == value)?;
        let step = self.step();
        let offset = self.padding_outer * step;
        Some(self.range_min + offset + index as f64 * step)
    }

    /// Map category to center position of band
    pub fn scale_center(&self, value: &str) -> Option<f64> {
        self.scale(value).map(|start| start + self.bandwidth() / 2.0)
    }

    /// Get the domain
    pub fn domain(&self) -> &[String] {
        &self.domain
    }

    /// Get range
    pub fn range(&self) -> (f64, f64) {
        (self.range_min, self.range_max)
    }

    /// Generate ticks (one per category)
    pub fn ticks(&self) -> Vec<Tick> {
        self.domain
            .iter()
            .enumerate()
            .filter_map(|(_, cat)| {
                let pos = self.scale_center(cat)?;
                Some(Tick {
                    value: pos,
                    label: cat.clone(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_band_scale() {
        let scale = BandScale::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            (0.0, 300.0),
        )
        .padding(0.0);

        assert_eq!(scale.bandwidth(), 100.0);
        assert_eq!(scale.scale("A"), Some(0.0));
        assert_eq!(scale.scale("B"), Some(100.0));
        assert_eq!(scale.scale("C"), Some(200.0));
    }

    #[test]
    fn test_band_scale_with_padding() {
        let scale = BandScale::new(
            vec!["A".to_string(), "B".to_string()],
            (0.0, 200.0),
        )
        .padding_inner(0.2)
        .padding_outer(0.0);

        // With 2 bands and 20% inner padding:
        // step = 200 / (2 + 0 - 0.2) = 200 / 1.8 = 111.11
        // bandwidth = step * 0.8 = 88.89
        assert!((scale.step() - 111.11).abs() < 0.1);
        assert!((scale.bandwidth() - 88.89).abs() < 0.1);
    }
}
