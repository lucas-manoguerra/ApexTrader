//! Rango de velas visibles en el área de dibujo.
//!
//! El renderer debe iterar solo `first..=last`, de modo que el trabajo por
//! frame sea O(N_visible) en lugar de O(N_total).

/// Rango de velas visibles en el área de dibujo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleRange {
    pub first: usize,
    pub last: usize,
}

impl VisibleRange {
    /// Crea un rango vacío (no hay velas visibles).
    pub fn empty() -> Self {
        Self { first: 1, last: 0 }
    }

    /// Crea un rango con las velas dadas.
    pub fn new(first: usize, last: usize) -> Self {
        Self { first, last }
    }

    /// Devuelve `true` si no hay velas visibles.
    pub fn is_empty(&self) -> bool {
        self.first > self.last
    }

    /// Devuelve la cantidad de velas visibles.
    pub fn len(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.last - self.first + 1
        }
    }
}

impl Default for VisibleRange {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_range_is_empty() {
        let r = VisibleRange::empty();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn valid_range_reports_length() {
        let r = VisibleRange::new(10, 20);
        assert!(!r.is_empty());
        assert_eq!(r.len(), 11);
    }
}
