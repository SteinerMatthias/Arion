use criterion::{criterion_group, criterion_main, Criterion};

use dusk_plonk::prelude::*;
use arion_merkle::{zk::opening_gadget, Item, Opening, Tree};

use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

// set max circuit size to 2^13 gates
const CAPACITY: usize = 16;

// set height and arity of the arion merkle tree
const HEIGHT: usize = 16;
const ARITY: usize = 4;

type ArionTree = Tree<(), HEIGHT, ARITY>;
type ArionItem = Item<()>;

// Create a circuit for the opening
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
struct OpeningCircuit {
    opening: Opening<(), HEIGHT, ARITY>,
    leaf: ArionItem,
}

impl Default for OpeningCircuit {
    fn default() -> Self {
        let mut tree = ArionTree::new();
        let empty_item = ArionItem {
            hash: BlsScalar::zero(),
            data: (),
        };
        tree.insert(0, empty_item);
        let opening = tree.opening(0).expect("There is a leaf at position 0");
        Self {
            opening,
            leaf: empty_item,
        }
    }
}

impl OpeningCircuit {
    /// Create a new OpeningCircuit
    pub fn new(
        opening: Opening<(), HEIGHT, ARITY>,
        leaf: ArionItem,
    ) -> Self {
        Self { opening, leaf }
    }
}

impl Circuit for OpeningCircuit {
    fn circuit<C>(&self, composer: &mut C) -> Result<(), Error>
    where
        C: Composer,
    {
        // append the leaf and opening gadget to the circuit
        let leaf = composer.append_witness(self.leaf.hash);
        let computed_root = opening_gadget(composer, &self.opening, leaf);

        // append the public root as public input to the circuit
        // and ensure it is equal to the computed root
        let constraint = Constraint::new()
            .left(-BlsScalar::one())
            .a(computed_root)
            .public(self.opening.root().hash);
        composer.append_gate(constraint);

        Ok(())
    }
}

fn bench_zk(c: &mut Criterion) {
    // create the prover and verifier circuit descriptions
    let label = b"merkle opening";
    let rng = &mut StdRng::seed_from_u64(0xdea1);
    let pp = PublicParameters::setup(1 << CAPACITY, rng).unwrap();
    let (prover, verifier) = Compiler::compile::<OpeningCircuit>(&pp, label)
        .expect("Circuit should compile successfully");

    // create a new tree and insert 100 leaves at random positions
    let tree = &mut ArionTree::new();
    let rng = &mut rand::rngs::StdRng::seed_from_u64(0xbeef);
    for _ in 0..100 {
        let pos = rng.next_u64() % u32::MAX as u64;
        let leaf = ArionItem {
            hash: arion::sponge::hash(&[pos.into()]),
            data: (),
        };
        tree.insert(pos, leaf);
    }

    // insert new leaf in the tree at random position to create opening
    let pos = rng.next_u64() % u32::MAX as u64;
    let leaf = ArionItem {
        hash: arion::sponge::hash(&[pos.into()]),
        data: (),
    };
    tree.insert(pos, leaf);

    // create a new opening circuit for the last leaf we inserted
    let opening = tree.opening(pos as u64).unwrap();
    // sanity check
    assert!(opening.verify(leaf.clone()));
    let circuit = OpeningCircuit::new(opening, leaf);
    let public_inputs = [opening.root().hash];

    let mut proof = Proof::default();
    let id = format!("arion opening proof generation, height = {}, arity = {}", HEIGHT, ARITY);
    c.bench_function(&id, |b| {
        b.iter(|| {
            (proof, _) = prover
                .prove(rng, &circuit)
                .expect("Proof generation should succeed");
        })
    });
    let id = format!("arion opening proof verification, height = {}, arity = {}", HEIGHT, ARITY);
    c.bench_function(&id, |b| {
        b.iter(|| {
            verifier
                .verify(&proof, &public_inputs)
                .expect("Proof verification should succeed");
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_zk
}
criterion_main!(benches);
