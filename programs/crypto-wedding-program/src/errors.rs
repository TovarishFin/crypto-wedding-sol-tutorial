use anchor_lang::prelude::*;

#[error_code]
pub enum WeddingError {
    #[msg("partner data not empty")]
    PartnerDataNotEmpty,
    #[msg("partner lamports not zero")]
    PartnerBalanceNotZero,
    #[msg("signer is not wedding member")]
    NotWeddingMember,
    #[msg("cannot cancel after created status")]
    CannotCancel,
    #[msg("creator does not match wedding storage")]
    InvalidCreator,
    #[msg("partner cannot be closed while wedding is initialized")]
    WeddingInitialized,
    #[msg("partner wedding does not match account wedding")]
    PartnerWeddingNotWedding,
    #[msg("cannot answer during invalid status")]
    InvalidAnswerStatus,
    #[msg("cannot divorce during invalid status")]
    InvalidDivorceStatus,
}
