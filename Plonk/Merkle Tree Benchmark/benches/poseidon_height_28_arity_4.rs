use criterion::{black_box, criterion_group, criterion_main, Criterion};

use dusk_bls12_381::BlsScalar;
use dusk_poseidon::sponge::hash as poseidon_hash;
use poseidon_merkle::{Item, Tree};

use rand::{RngCore, SeedableRng};

// set height and arity of the poseidon merkle tree
const HEIGHT: usize = 28;
const ARITY: usize = 4;

type PoseidonTree = Tree<(), HEIGHT, ARITY>;
type PoseidonItem = Item<()>;

fn bench_poseidon(c: &mut Criterion) {
    let tree = &mut PoseidonTree::new();
    let rng = &mut rand::rngs::StdRng::seed_from_u64(0xbeef);

    let id = format!("poseidon insertion, height = {}, arity = {}", HEIGHT, ARITY);
    c.bench_function(&id, |b| {
        b.iter(|| {
            let pos = rng.next_u64() % u32::MAX as u64;
            let hash = poseidon_hash(&[BlsScalar::from(pos)]);
            let item = PoseidonItem { hash, data: () };
            tree.insert(black_box(pos), black_box(item));
        })
    });
}
criterion_group! {
    name = benches; 
    config = Criterion::default().sample_size(10);
    targets = bench_poseidon
}
criterion_main!(benches);
