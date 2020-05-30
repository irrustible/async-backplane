use futures::prelude::*;
use tingle::{Entanglement, Name, Quantum};

#[test]
fn observer() {
    let name_1 = Name::random();
    let name_2 = Name::random();

    let mut quantum_1 = Quantum::<i32>::new(name_1);
    let quantum_2 = Quantum::<i32>::new(name_2);

    assert_eq!(quantum_1.name(), name_1);
    assert_eq!(quantum_2.name(), name_2);

    let tangle_1 = quantum_1.tangle();
    let tangle_2 = quantum_2.tangle();

    assert_eq!(tangle_1.name(), name_1);
    assert_eq!(tangle_2.name(), name_2);

    smol::run(quantum_1.entangle(tangle_2, Entanglement::Observer));

    quantum_2.exit(42);

    assert_eq!(smol::run(quantum_1.next()), Some(42));
}

#[test]
fn entangled() {
    let name_1 = Name::random();
    let name_2 = Name::random();

    let mut quantum_1 = Quantum::<i32>::new(name_1);
    let mut quantum_2 = Quantum::<i32>::new(name_2);

    assert_eq!(quantum_1.name(), name_1);
    assert_eq!(quantum_2.name(), name_2);

    let tangle_1 = quantum_1.tangle();
    let tangle_2 = quantum_2.tangle();

    assert_eq!(tangle_1.name(), name_1);
    assert_eq!(tangle_2.name(), name_2);

    smol::run(quantum_1.entangle(tangle_2, Entanglement::Entangled));

    quantum_1.exit(42);

    assert_eq!(smol::run(quantum_2.next()), Some(42));
}
