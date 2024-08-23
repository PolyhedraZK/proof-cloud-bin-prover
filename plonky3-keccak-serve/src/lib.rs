use p3_baby_bear::BabyBear;
use p3_challenger::{HashChallenger, SerializingChallenger32};
use p3_commit::ExtensionMmcs;
use p3_dft::Radix2DitParallel;
use p3_field::extension::BinomialExtensionField;
use p3_keccak::Keccak256Hash;
use p3_merkle_tree::FieldMerkleTreeMmcs;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher32};
use p3_uni_stark::StarkConfig;
use serde::{Deserialize, Serialize};

pub type Val = BabyBear;
pub type Challenge = BinomialExtensionField<Val, 4>;
pub type ByteHash = Keccak256Hash;
pub type FieldHash = SerializingHasher32<ByteHash>;
use p3_fri::TwoAdicFriPcs;

// pub const NUM_HASHES: usize = 1365;
pub const NUM_HASHES: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MyWitness {
    pub inputs: Vec<[u64; 25]>,
}

pub type MyCompress = CompressionFunctionFromHasher<u8, ByteHash, 2, 32>;
pub type ValMmcs = FieldMerkleTreeMmcs<Val, u8, FieldHash, MyCompress, 32>;
pub type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
pub type Dft = Radix2DitParallel;
pub type Challenger = SerializingChallenger32<Val, HashChallenger<u8, ByteHash, 32>>;
pub type Pcs = TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>;
pub type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;
