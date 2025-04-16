#![allow(missing_docs)]

use core::fmt;
use std::{cmp::Reverse, collections::BTreeSet, fmt::Debug};

use hashbrown::HashMap;
use itertools::Itertools;
use p3_matrix::{
    dense::{RowMajorMatrix, RowMajorMatrixView},
    stack::VerticalPair,
    Matrix,
};
use serde::{Deserialize, Serialize};

use super::{Challenge, Com, OpeningProof, StarkGenericConfig, Val};

pub type QuotientOpenedValues<T> = Vec<T>;

pub struct ShardMainData<SC: StarkGenericConfig, M, P> {
    pub traces: Vec<M>,
    pub main_commit: Com<SC>,
    pub main_data: P,
    pub chip_ordering: HashMap<String, usize>,
}

impl<SC: StarkGenericConfig, M, P> ShardMainData<SC, M, P> {
    pub const fn new(
        traces: Vec<M>,
        main_commit: Com<SC>,
        main_data: P,
        chip_ordering: HashMap<String, usize>,
    ) -> Self {
        Self { traces, main_commit, main_data, chip_ordering }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardCommitment<C> {
    pub main_commit: C,
    pub permutation_commit: C,
    pub quotient_commit: C,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize"))]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct AirOpenedValues<T> {
    pub local: Vec<T>,
    pub next: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize"))]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct ChipOpenedValues<T> {
    pub preprocessed: AirOpenedValues<T>,
    pub main: AirOpenedValues<T>,
    pub permutation: AirOpenedValues<T>,
    pub quotient: Vec<Vec<T>>,
    pub cumulative_sum: T,
    pub log_degree: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardOpenedValues<T> {
    pub chips: Vec<ChipOpenedValues<T>>,
}

/// The maximum number of elements that can be stored in the public values vec.
pub const PROOF_MAX_NUM_PVS: usize = 0;

#[derive(Serialize, Deserialize, Clone)]
#[serde(bound = "")]
pub struct ShardProof<SC: StarkGenericConfig> {
    pub commitment: ShardCommitment<Com<SC>>,
    pub opened_values: ShardOpenedValues<Challenge<SC>>,
    pub opening_proof: OpeningProof<SC>,
    pub chip_ordering: HashMap<String, usize>,
}

impl<SC: StarkGenericConfig> Debug for ShardProof<SC> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShardProof").finish()
    }
}

impl<T: Send + Sync + Clone> AirOpenedValues<T> {
    #[must_use]
    pub fn view(&self) -> VerticalPair<RowMajorMatrixView<'_, T>, RowMajorMatrixView<'_, T>> {
        let a = RowMajorMatrixView::new_row(&self.local);
        let b = RowMajorMatrixView::new_row(&self.next);
        VerticalPair::new(a, b)
    }
}

impl<SC: StarkGenericConfig> ShardProof<SC> {
    pub fn cumulative_sum(&self) -> Challenge<SC> {
        self.opened_values
            .chips
            .iter()
            .map(|c| c.cumulative_sum)
            .sum()
    }

    pub fn log_degree_cpu(&self) -> usize {
        let idx = self.chip_ordering.get("Cpu").expect("Cpu chip not found");
        self.opened_values.chips[*idx].log_degree
    }

    pub fn contains_cpu(&self) -> bool {
        self.chip_ordering.contains_key("Cpu")
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(bound = "")]
pub struct MachineProof<SC: StarkGenericConfig> {
    pub shard_proof: ShardProof<SC>,
}

impl<SC: StarkGenericConfig> Debug for MachineProof<SC> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Proof").finish()
    }
}
