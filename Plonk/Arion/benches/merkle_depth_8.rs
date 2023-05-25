#![cfg(feature = "alloc")]

use arion::tree::{self, ArionBranch, ArionLeaf, ArionTree};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dusk_plonk::error::Error as PlonkError;
use dusk_plonk::prelude::*;
use nstack::annotation::Keyed;
use rand::rngs::OsRng;
use rand::{CryptoRng, RngCore};

const DEPTH: usize = 8;
const CAPACITY: usize = 15;
type Tree = ArionTree<MockLeaf, u64, DEPTH>;

#[derive(Debug, Default, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct MockLeaf {
    data: BlsScalar,
    pub pos: u64,
}

impl Keyed<u64> for MockLeaf {
    fn key(&self) -> &u64 {
        &self.pos
    }
}

impl MockLeaf {
    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let data = BlsScalar::random(rng);
        let pos = 0;

        Self { data, pos }
    }
}

impl From<u64> for MockLeaf {
    fn from(n: u64) -> MockLeaf {
        MockLeaf {
            data: BlsScalar::from(n),
            pos: 0,
        }
    }
}

impl ArionLeaf for MockLeaf {
    fn arion_hash(&self) -> BlsScalar {
        self.data
    }

    fn pos(&self) -> &u64 {
        &self.pos
    }

    fn set_pos(&mut self, pos: u64) {
        self.pos = pos;
    }
}

#[derive(Default)]
struct MerkleOpeningCircuit {
    branch: ArionBranch<DEPTH>,
}

impl MerkleOpeningCircuit {
    pub fn random<R: RngCore + CryptoRng>(
        rng: &mut R,
        tree: &mut Tree,
    ) -> Self {
        let leaf = MockLeaf::random(rng);
        let pos = tree.push(leaf);

        let branch = tree.branch(pos).expect(
            "Tree should be possible to access at an existing position",
        );

        Self { branch }
    }
}

impl Circuit for MerkleOpeningCircuit {
    fn circuit<C>(&self, composer: &mut C) -> Result<(), PlonkError>
    where
        C: Composer,
    {
        use std::ops::Deref;

        let leaf: BlsScalar = *self.branch.deref();
        let leaf = composer.append_witness(leaf);

        let root = self.branch.root();
        let root = composer.append_witness(*root);

        let root_p =
            tree::merkle_opening::<C, DEPTH>(composer, &self.branch, leaf);

        composer.assert_equal(root_p, root);

        Ok(())
    }
}

fn bench_opening_proof(c: &mut Criterion) {
    // Prepare benchmarks and initialize variables
    let label = b"dusk-network";
    let pp = PublicParameters::setup(1 << CAPACITY, &mut OsRng).unwrap();
    let (prover, verifier) =
        Compiler::compile::<MerkleOpeningCircuit>(&pp, label)
            .expect("Circuit should compile successfully");

    // Benchmark proof creation
    let mut tree = Tree::default();
    let circuit = MerkleOpeningCircuit::random(&mut OsRng, &mut tree);
    let mut proof = Proof::default();
    let mut public_inputs = Vec::new();
    let id = format!("merkle opening proof generation, depth = {}", DEPTH);
    c.bench_function(&id, |b| {
        b.iter(|| {
            (proof, public_inputs) = prover
                .prove(&mut OsRng, black_box(&circuit))
                .expect("Proof generation should succeed");
        })
    });

    // Benchmark proof verification
    let id = format!("merkle opening proof verification, depth = {}", DEPTH);
    c.bench_function(&id, |b| {
        b.iter(|| {
            verifier
                .verify(black_box(&proof), &public_inputs)
                .expect("Proof verification should succeed");
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_opening_proof
}
criterion_main!(benches);
