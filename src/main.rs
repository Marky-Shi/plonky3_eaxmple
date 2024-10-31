use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

use p3_baby_bear::{BabyBear, DiffusionMatrixBabyBear};
use p3_challenger::DuplexChallenger;
use p3_commit::ExtensionMmcs;
use p3_dft::Radix2DitParallel;
use p3_field::extension::BinomialExtensionField;
use p3_field::{AbstractField, Field};
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_merkle_tree::MerkleTreeMmcs;
use p3_poseidon2::{Poseidon2, Poseidon2ExternalMatrixGeneral};
use p3_symmetric::{PaddingFreeSponge, TruncatedPermutation};
use p3_uni_stark::{prove, verify, StarkConfig};
use rand::thread_rng;

use plonky3_example::basics::*;


fn main(){
    println!("start setup env");

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();

        // config for baby bear
        type Val = BabyBear;
        type Perm = Poseidon2<Val, Poseidon2ExternalMatrixGeneral, DiffusionMatrixBabyBear, 16, 7>;
        type MyHash = PaddingFreeSponge<Perm, 16, 8, 8>;
        type MyCompress = TruncatedPermutation<Perm, 2, 8, 16>;
        type ValMmcs =
            MerkleTreeMmcs<<Val as Field>::Packing, <Val as Field>::Packing, MyHash, MyCompress, 8>;
        type Challenge = BinomialExtensionField<Val, 4>;
        type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
        type Challenger = DuplexChallenger<Val, Perm, 16, 8>;
        type Dft = Radix2DitParallel<Val>;
        type Pcs = TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>;
        type MyConfig = StarkConfig<Pcs, Challenge, Challenger>;
        
        
        let perm = Perm::new_from_rng_128(
            Poseidon2ExternalMatrixGeneral,
            DiffusionMatrixBabyBear::default(),
            &mut thread_rng(),
        );
        let hash = MyHash::new(perm.clone());
        let compress = MyCompress::new(perm.clone());
        let val_mmcs = ValMmcs::new(hash, compress);
        let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
        let dft = Dft::default();
        let num_steps = 8; // Choose the number of Fibonacci steps
        let final_value = 21; // Choose the final Fibonacci value
        let air = MyAir { num_steps, final_value };
        let trace = generate_air_trace::<Val>(num_steps);
        let fri_config = FriConfig {
            log_blowup: 2,
            num_queries: 28,
            proof_of_work_bits: 8,
            mmcs: challenge_mmcs,
        };
        let pcs = Pcs::new(dft, val_mmcs, fri_config);
        let config = MyConfig::new(pcs);
        let mut challenger = Challenger::new(perm.clone());
        let pis = vec![
            BabyBear::from_canonical_u64(0),
            BabyBear::from_canonical_u64(1),
            BabyBear::from_canonical_u64(21),
        ];
        println!("start generating proof .... ");
        let proof = prove(&config, &air, &mut challenger, trace, &pis);
        let mut challenger = Challenger::new(perm);
        verify(&config, &air, &mut challenger, &proof, &pis).expect("verification failed");
        println!("verify Success!");
}
