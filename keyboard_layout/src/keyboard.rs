//! This module provides a struct representing a keyboard.

use crate::key::{Finger, Hand, Key, MatrixPosition, Position};

use ahash::{AHashMap, AHashSet};
use anyhow::Result;
use serde::Deserialize;
use std::fs::File;

/// The index of a [`Key`] in the `keys` vec of a [`Keyboard`]
pub type KeyIndex = u8;

/// A struct representing a keyboard as a list of keys
#[derive(Clone, Debug)]
pub struct Keyboard {
    /// The keys of the keyboard
    pub keys: Vec<Key>,
    plot_template: String,
    plot_template_short: String,
}

/// A collection of all relevant properties for the keys on a keyboard (configuration).
///
/// Corresponds to (parts of) a YAML configuration file.
#[derive(Deserialize, Debug)]
pub struct KeyboardYAML {
    matrix_positions: Vec<Vec<MatrixPosition>>,
    positions: Vec<Vec<Position>>,
    hands: Vec<Vec<Hand>>,
    fingers: Vec<Vec<Finger>>,
    key_costs: Vec<Vec<f64>>,
    symmetries: Vec<Vec<u8>>,
    unbalancing_positions: Vec<Vec<f64>>,
    plot_template: String,
    plot_template_short: String,
}

/// Takes a slice of some iterable and checks whether that iterable contains
/// duplicates of any of its elements.
fn contains_duplicates<T: PartialEq>(v: &[T]) -> bool {
    // Cycle through all elements
    v.iter().enumerate().any(|(first_idx, checked_pos)| {
        // Get index of last element that is equal to `checked_pos`
        let last_idx = v.iter().rposition(|pos| pos == checked_pos).unwrap();
        // See if that last element is different from the first one,
        // which would be a duplicate.
        first_idx != last_idx
    })
}

impl KeyboardYAML {
    /// Checks the [`KeyboardYAML`] for common errors.
    pub fn validate(&self) -> Result<(), String> {
        let flat_matrix_positions = self.matrix_positions.concat();
        let flat_positions = self.positions.concat();

        // Make sure that all settings that should have the same number of elements
        // do in fact have the same number of elements.
        let mut lengths = AHashSet::default();
        lengths.insert(flat_matrix_positions.len());
        lengths.insert(flat_positions.len());
        lengths.insert(self.hands.concat().len());
        lengths.insert(self.fingers.concat().len());
        lengths.insert(self.key_costs.concat().len());
        lengths.insert(self.symmetries.concat().len());
        lengths.insert(self.unbalancing_positions.concat().len());
        if lengths.len() > 1 {
            return Err(
                "Not every description of the keyboard contains the same number of keys."
                    .to_string(),
            );
        }

        // Make sure there are no duplicates in `matrix_positions`.
        if contains_duplicates(&flat_matrix_positions) {
            return Err("Duplicate `matrix_positions` found.".to_string());
        }

        // Make sure there are no duplicates in `positions`.
        if contains_duplicates(&flat_positions) {
            return Err("Duplicate `positions` found.".to_string());
        }

        Ok(())
    }
}

impl Keyboard {
    /// Generate a [`Keyboard`] from a [`KeyboardYAML`] object
    pub fn from_yaml_object(k: KeyboardYAML) -> Self {
        let keys = k
            .hands
            .into_iter()
            .flatten()
            .zip(k.fingers.into_iter().flatten())
            .zip(k.matrix_positions.into_iter().flatten())
            .zip(k.positions.into_iter().flatten())
            .zip(k.symmetries.into_iter().flatten())
            .zip(k.key_costs.into_iter().flatten())
            .zip(k.unbalancing_positions.into_iter().flatten())
            .map(
                |(
                    (((((hand, finger), matrix_position), position), symmetry_index), cost),
                    unbalancing,
                )| Key {
                    hand,
                    finger,
                    matrix_position,
                    position,
                    symmetry_index,
                    cost,
                    unbalancing,
                },
            )
            .collect();

        Keyboard {
            keys,
            plot_template: k.plot_template,
            plot_template_short: k.plot_template_short,
        }
    }

    /// Generate a [`Keyboard`] from a YAML file
    pub fn from_yaml_file(filename: &str) -> Result<Self> {
        let f = File::open(filename)?;
        let k: KeyboardYAML = serde_yaml::from_reader(f)?;
        Ok(Keyboard::from_yaml_object(k))
    }

    /// Generate a [`Keyboard`] from a YAML string
    pub fn from_yaml_str(data: &str) -> Result<Self> {
        let k: KeyboardYAML = serde_yaml::from_str(data)?;
        Ok(Keyboard::from_yaml_object(k))
    }

    /// Plot a graphical representation of the keyboard with given key labels
    pub fn plot(&self, key_labels: &[char]) -> String {
        let mut reg = handlebars::Handlebars::new();
        reg.register_escape_fn(handlebars::no_escape);
        let labels: AHashMap<usize, char> = key_labels.iter().cloned().enumerate().collect();
        reg.render_template(&self.plot_template, &labels).unwrap()
    }

    /// Plot a compact graphical representation of the keyboard with given key labels without borders (compatible with ArneBab's input strings)
    pub fn plot_compact(&self, key_labels: &[char]) -> String {
        let mut reg = handlebars::Handlebars::new();
        reg.register_escape_fn(handlebars::no_escape);
        let labels: AHashMap<usize, char> = key_labels.iter().cloned().enumerate().collect();
        reg.render_template(&self.plot_template_short, &labels)
            .unwrap()
    }
}
