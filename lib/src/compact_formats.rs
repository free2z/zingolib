use ff::PrimeField;
use group::GroupEncoding;
use orchard::note::ExtractedNoteCommitment;
use std::convert::TryInto;

use zcash_note_encryption::COMPACT_NOTE_SIZE;
use zcash_primitives::{
    block::{BlockHash, BlockHeader},
    consensus::BlockHeight,
};

tonic::include_proto!("cash.z.wallet.sdk.rpc");

impl CompactBlock {
    /// Returns the [`BlockHash`] for this block.
    ///
    /// # Panics
    ///
    /// This function will panic if [`CompactBlock.header`] is not set and
    /// [`CompactBlock.hash`] is not exactly 32 bytes.
    ///
    /// [`CompactBlock.header`]: #structfield.header
    /// [`CompactBlock.hash`]: #structfield.hash
    pub fn hash(&self) -> BlockHash {
        if let Some(header) = self.header() {
            header.hash()
        } else {
            BlockHash::from_slice(&self.hash)
        }
    }

    /// Returns the [`BlockHash`] for this block's parent.
    ///
    /// # Panics
    ///
    /// This function will panic if [`CompactBlock.header`] is not set and
    /// [`CompactBlock.prevHash`] is not exactly 32 bytes.
    ///
    /// [`CompactBlock.header`]: #structfield.header
    /// [`CompactBlock.prevHash`]: #structfield.prevHash
    pub fn prev_hash(&self) -> BlockHash {
        if let Some(header) = self.header() {
            header.prev_block
        } else {
            BlockHash::from_slice(&self.prev_hash)
        }
    }

    /// Returns the [`BlockHeader`] for this block if present.
    ///
    /// A convenience method that parses [`CompactBlock.header`] if present.
    ///
    /// [`CompactBlock.header`]: #structfield.header
    pub fn header(&self) -> Option<BlockHeader> {
        if self.header.is_empty() {
            None
        } else {
            BlockHeader::read(&self.header[..]).ok()
        }
    }

    /// Returns the [`BlockHeight`] for this block.
    ///
    /// A convenience method that wraps [`CompactBlock.height`]
    ///
    /// [`CompactBlock.height`]: #structfield.height
    pub fn height(&self) -> BlockHeight {
        BlockHeight::from(self.height as u32)
    }
}

impl CompactSaplingOutput {
    /// Returns the note commitment for this output.
    ///
    /// A convenience method that parses [`CompactSaplingOutput.cmu`].
    ///
    /// [`CompactSaplingOutput.cmu`]: #structfield.cmu
    pub fn cmu(&self) -> Result<bls12_381::Scalar, ()> {
        let mut repr = [0; 32];
        repr.as_mut().copy_from_slice(&self.cmu[..]);
        Option::from(bls12_381::Scalar::from_repr(repr)).ok_or(())
    }

    /// Returns the ephemeral public key for this output.
    ///
    /// A convenience method that parses [`CompactSaplingOutput.epk`].
    ///
    /// [`CompactSaplingOutput.epk`]: #structfield.epk
    pub fn epk(&self) -> Result<jubjub::ExtendedPoint, ()> {
        let p = jubjub::ExtendedPoint::from_bytes(&self.epk[..].try_into().map_err(|_| ())?);
        if p.is_some().into() {
            Ok(p.unwrap())
        } else {
            Err(())
        }
    }
}

impl From<CompactSaplingOutput>
    for zcash_client_backend::proto::compact_formats::CompactSaplingOutput
{
    fn from(value: CompactSaplingOutput) -> Self {
        Self {
            cmu: value.cmu,
            ephemeralKey: value.epk,
            ciphertext: value.ciphertext,
            ..Default::default()
        }
    }
}

impl TryFrom<&CompactOrchardAction> for orchard::note_encryption::CompactAction {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &CompactOrchardAction) -> Result<Self, Self::Error> {
        Ok(Self::from_parts(
            Option::from(orchard::note::Nullifier::from_bytes(&<[u8; 32]>::try_from(
                value.nullifier.as_slice(),
            )?))
            .ok_or("bad nullifier")?,
            Option::from(ExtractedNoteCommitment::from_bytes(&<[u8; 32]>::try_from(
                value.cmx.as_slice(),
            )?))
            .ok_or("bad enc")?,
            <[u8; 32]>::try_from(value.ephemeral_key.as_slice())?.into(),
            <[u8; COMPACT_NOTE_SIZE]>::try_from(value.ciphertext.as_slice())?,
        ))
    }
}
