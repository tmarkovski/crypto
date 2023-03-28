use ark_ec::{pairing::Pairing, AffineRepr};
use ark_std::{collections::BTreeMap, io::Write, rand::RngCore, vec::Vec};

use dock_crypto_utils::{
    iter::take_while_satisfy, misc::check_seq_from,
    randomized_pairing_check::RandomizedPairingChecker, try_iter::CheckLeft,
};

use coconut_crypto::{proof::*, setup::*};
use itertools::Itertools;

use crate::{error::ProofSystemError, statement_proof::StatementProof};

use super::merge_indexed_messages_with_blindings;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PSSignaturePoK<'a, E: Pairing> {
    pub id: usize,
    pub revealed_messages: &'a BTreeMap<usize, E::ScalarField>,
    pub signature_params: &'a SignatureParams<E>,
    pub public_key: &'a PublicKey<E>,
    pub protocol: Option<Protocol<'a, E>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LeakedVec<'a, T> {
    slice: &'a [T],
    cap: usize,
}

impl<'a, T> LeakedVec<'a, T> {
    fn new(mut vector: Vec<T>) -> Self {
        vector.shrink_to_fit();
        let cap = vector.capacity();

        Self {
            slice: Vec::leak(vector),
            cap,
        }
    }
}

impl<'a, T> Drop for LeakedVec<'a, T> {
    fn drop(&mut self) {
        let cap = self.cap;
        let len = self.slice.len();

        unsafe { Vec::from_raw_parts(self.slice as *const _ as *mut T, len, cap) };
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Protocol<'a, E: Pairing> {
    generator: SignaturePoKGenerator<'a, E>,
    blinded_messages: LeakedVec<'a, (usize, E::ScalarField)>,
}

impl<'a, E: Pairing> PSSignaturePoK<'a, E> {
    pub fn new(
        id: usize,
        revealed_messages: &'a BTreeMap<usize, E::ScalarField>,
        signature_params: &'a SignatureParams<E>,
        public_key: &'a PublicKey<E>,
    ) -> Self {
        Self {
            id,
            revealed_messages,
            signature_params,
            public_key,
            protocol: None,
        }
    }

    pub fn init<R: RngCore>(
        &mut self,
        rng: &mut R,
        blindings: BTreeMap<usize, E::ScalarField>,
        witness: crate::witness::PoKPSSignature<E>,
    ) -> Result<(), ProofSystemError> {
        if self.protocol.is_some() {
            return Err(ProofSystemError::SubProtocolAlreadyInitialized(self.id));
        }
        let total_message_count = self.revealed_messages.len() + witness.unrevealed_messages.len();
        if total_message_count != self.signature_params.supported_message_count() {
            Err(ProofSystemError::PSProtocolInvalidMessageCount(
                total_message_count,
                self.signature_params.supported_message_count(),
            ))?
        }

        let messages_to_blind = LeakedVec::new(witness.unrevealed_messages.into_iter().collect());

        let mut invalid_blinding_idx = None;
        let messages_to_blind_with_blindings = merge_indexed_messages_with_blindings(
            messages_to_blind.slice.iter().map(|(idx, msg)| (*idx, msg)),
            blindings,
            CommitMessage::BlindMessageRandomly,
            CommitMessage::blind_message_with,
            &mut invalid_blinding_idx,
        );
        let mut non_seq_idx = None;
        let all_messages = take_while_satisfy(
            messages_to_blind_with_blindings.merge_by(
                self.revealed_messages
                    .iter()
                    .map(|(idx, _)| (*idx, CommitMessage::RevealMessage)),
                |(a, _), (b, _)| a < b,
            ),
            CheckLeft(check_seq_from(0)),
            &mut non_seq_idx,
        )
        .map(|(_, msg)| msg);

        let protocol = SignaturePoKGenerator::init(
            rng,
            all_messages,
            &witness.signature,
            self.public_key,
            self.signature_params,
        );
        if let Some(idx) = invalid_blinding_idx {
            Err(ProofSystemError::PSProtocolInvalidBlindingIndex(idx))?
        } else if let Some(invalid) = non_seq_idx {
            Err(invalid.over(
                ProofSystemError::PSProtocolMessageIndicesMustStartFromZero,
                ProofSystemError::PSProtocolNonSequentialMessageIndices,
            ))?
        }

        self.protocol = Some(Protocol {
            generator: protocol?,
            blinded_messages: messages_to_blind,
        });
        Ok(())
    }

    pub fn challenge_contribution<W: Write>(&self, writer: W) -> Result<(), ProofSystemError> {
        self.protocol
            .as_ref()
            .ok_or(ProofSystemError::SubProtocolNotReadyToGenerateChallenge(
                self.id,
            ))?
            .generator
            .challenge_contribution(writer, self.public_key, self.signature_params)?;
        Ok(())
    }

    pub fn gen_proof_contribution<G: AffineRepr>(
        &mut self,
        challenge: &E::ScalarField,
    ) -> Result<StatementProof<E, G>, ProofSystemError> {
        let protocol =
            self.protocol
                .take()
                .ok_or(ProofSystemError::SubProtocolNotReadyToGenerateProof(
                    self.id,
                ))?;
        let proof = protocol.generator.gen_proof(challenge)?;
        Ok(StatementProof::PoKPSSignature(proof))
    }

    pub fn verify_proof_contribution(
        &self,
        challenge: &E::ScalarField,
        proof: &SignaturePoK<E>,
        pk: impl Into<PreparedPublicKey<E>>,
        params: impl Into<PreparedSignatureParams<E>>,
        pairing_checker: &mut Option<RandomizedPairingChecker<E>>,
    ) -> Result<(), ProofSystemError> {
        match pairing_checker {
            Some(c) => proof.verify_with_randomized_pairing_checker(
                challenge,
                self.revealed_messages.iter().map(|(idx, msg)| (*idx, msg)),
                &pk.into(),
                &params.into(),
                c,
            )?,
            None => proof.verify(
                challenge,
                self.revealed_messages.iter().map(|(idx, msg)| (*idx, msg)),
                &pk.into(),
                &params.into(),
            )?,
        }
        Ok(())
    }
}
