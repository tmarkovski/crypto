use secret_sharing_and_dkg::common::ParticipantId;

use crate::helpers::{IndexIsOutOfBounds, InvalidPairOrItem};

/// An error originated from `Signature`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PSError {
    NoMessages,
    InvalidMessageCount { received: usize, expected: usize },
    MessageIndicesMustBeUniqueAndSorted(InvalidPairOrItem<usize>),
    ZeroSignature,
    MessageIndexIsOutOfBounds(IndexIsOutOfBounds),
    PairingCheckFailed,
}

impl From<IndexIsOutOfBounds> for PSError {
    fn from(err: IndexIsOutOfBounds) -> Self {
        Self::MessageIndexIsOutOfBounds(err)
    }
}

impl From<InvalidPairOrItem<usize>> for PSError {
    fn from(err: InvalidPairOrItem<usize>) -> Self {
        Self::MessageIndicesMustBeUniqueAndSorted(err)
    }
}

/// An error originated from `BlindSignature`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlindPSError {
    NoCommitmentsOrMessages,
    IndexIsOutOfBounds(IndexIsOutOfBounds),
    InvalidCommitmentsAndMessagesCount {
        received: Option<usize>,
        expected: usize,
    },
    BlindingIndicesMustBeUniqueAndSorted(InvalidPairOrItem<usize>),
    IncompatibleVerificationKey,
}

impl From<IndexIsOutOfBounds> for BlindPSError {
    fn from(err: IndexIsOutOfBounds) -> Self {
        Self::IndexIsOutOfBounds(err)
    }
}

impl From<InvalidPairOrItem<usize>> for BlindPSError {
    fn from(err: InvalidPairOrItem<usize>) -> Self {
        Self::BlindingIndicesMustBeUniqueAndSorted(err)
    }
}

/// An error originated from `AggregatedSignature`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregatedPSError {
    NoSignatures,
    InvalidSigma1For(ParticipantId),
    ParticipantIdsMustBeUniqueAndSorted(InvalidPairOrItem<ParticipantId>),
    PSError(PSError),
}

impl From<InvalidPairOrItem<ParticipantId>> for AggregatedPSError {
    fn from(err: InvalidPairOrItem<ParticipantId>) -> Self {
        Self::ParticipantIdsMustBeUniqueAndSorted(err)
    }
}

impl From<PSError> for AggregatedPSError {
    fn from(error: PSError) -> Self {
        Self::PSError(error)
    }
}
