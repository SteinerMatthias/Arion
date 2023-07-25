use criterion::{black_box, criterion_group, criterion_main, Criterion};

use dusk_bls12_381::BlsScalar;
use arion::sponge::hash as arion_hash;
use arion_merkle::{Item, Tree};

use rand::{RngCore, SeedableRng};

// set height and arity of the arion merkle tree
const HEIGHT: usize = 28;
const ARITY: usize = 4;

type ArionTree = Tree<(), HEIGHT, ARITY>;
type ArionItem = Item<()>;

fn bench_arion(c: &mut Criterion) {
    let tree = &mut ArionTree::new();
    let rng = &mut rand::rngs::StdRng::seed_from_u64(0xbeef);

    let id = format!("arion insertion, height = {}, arity = {}", HEIGHT, ARITY);
    c.bench_function(&id, |b| {
        b.iter(|| {
            let pos = rng.next_u64() % u32::MAX as u64;
            let hash = arion_hash(&[BlsScalar::from(pos)]);
            let item = ArionItem { hash, data: () };
            tree.insert(black_box(pos), black_box(item));
        })
    });
}
criterion_group! {
    name = benches; 
    config = Criterion::default().sample_size(10);
    targets = bench_arion
}
criterion_main!(benches);
