use crate::prelude::StatementProof;
use ark_ec::{pairing::Pairing, AffineRepr};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::{collections::BTreeSet, vec::Vec};
use legogroth16::aggregation;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct AggregatedGroth16<E: Pairing> {
    pub proof: aggregation::groth16::AggregateProof<E>,
    pub statements: BTreeSet<usize>,
}

/// Created by the prover and verified by the verifier
#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Proof<E: Pairing, G: AffineRepr> {
    pub statement_proofs: Vec<StatementProof<E, G>>,
    pub nonce: Option<Vec<u8>>,
    // TODO: Remove this skip
    #[serde(skip)]
    pub aggregated_groth16: Option<Vec<AggregatedGroth16<E>>>,
    // TODO: Remove this skip
    #[serde(skip)]
    pub aggregated_legogroth16: Option<Vec<AggregatedGroth16<E>>>,
}

impl<E: Pairing, G: AffineRepr> PartialEq for Proof<E, G> {
    fn eq(&self, other: &Self) -> bool {
        (self.statement_proofs == other.statement_proofs) && (self.nonce == other.nonce)
        // TODO: Add remaining
    }
}
