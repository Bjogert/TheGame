//! Dependency matrix describing which goods satisfy wellbeing categories.
use std::collections::HashMap;

use crate::economy::components::{Profession, TradeGood};

/// High-level wellbeing categories used when evaluating profession needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependencyCategory {
    Food,
    Tools,
    Housing,
}

impl DependencyCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Food => "food",
            Self::Tools => "tools",
            Self::Housing => "housing",
        }
    }
}

#[derive(Debug, Default, Clone)]
struct DependencyEntry {
    required_categories: Vec<DependencyCategory>,
}

/// Maps professions and goods to wellbeing categories.
#[derive(Debug, Clone)]
pub struct EconomyDependencyMatrix {
    profession_requirements: HashMap<Profession, DependencyEntry>,
    good_categories: HashMap<TradeGood, Vec<DependencyCategory>>,
}

impl Default for EconomyDependencyMatrix {
    fn default() -> Self {
        let mut matrix = Self {
            profession_requirements: HashMap::new(),
            good_categories: HashMap::new(),
        };

        matrix
            .good_categories
            .insert(TradeGood::Grain, vec![DependencyCategory::Food]);
        matrix
            .good_categories
            .insert(TradeGood::Flour, vec![DependencyCategory::Food]);
        matrix
            .good_categories
            .insert(TradeGood::Tools, vec![DependencyCategory::Tools]);

        matrix.set_profession_requirements(
            Profession::Farmer,
            vec![DependencyCategory::Food, DependencyCategory::Tools],
        );
        matrix.set_profession_requirements(
            Profession::Miller,
            vec![DependencyCategory::Food, DependencyCategory::Tools],
        );
        matrix.set_profession_requirements(
            Profession::Blacksmith,
            vec![DependencyCategory::Food, DependencyCategory::Tools],
        );

        matrix
    }
}

impl EconomyDependencyMatrix {
    fn set_profession_requirements(
        &mut self,
        profession: Profession,
        categories: Vec<DependencyCategory>,
    ) {
        self.profession_requirements.insert(
            profession,
            DependencyEntry {
                required_categories: categories,
            },
        );
    }

    pub fn requirements(&self, profession: Profession) -> &[DependencyCategory] {
        self.profession_requirements
            .get(&profession)
            .map(|entry| entry.required_categories.as_slice())
            .unwrap_or(&[])
    }

    pub fn categories_for_good(&self, good: TradeGood) -> &[DependencyCategory] {
        self.good_categories
            .get(&good)
            .map(|list| list.as_slice())
            .unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_matrix_exposes_defaults() {
        let matrix = EconomyDependencyMatrix::default();

        let farmer_needs = matrix.requirements(Profession::Farmer);
        assert!(farmer_needs.contains(&DependencyCategory::Food));
        assert!(farmer_needs.contains(&DependencyCategory::Tools));

        let categories = matrix.categories_for_good(TradeGood::Tools);
        assert_eq!(categories, &[DependencyCategory::Tools]);
        assert_eq!(
            matrix.categories_for_good(TradeGood::Grain)[0],
            DependencyCategory::Food
        );
    }
}
