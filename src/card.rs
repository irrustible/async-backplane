use crate::{Address, Name};

#[derive(Clone)]
pub struct Card<T: Clone> {
  pub name: Name,
  pub address: Address<T>,
}

impl<T: Clone> Card<T> {
  pub fn new(name: Name, address: Address<T>) -> Card<T> {
    Card { name, address }
  }
}

impl<T: Clone> Unpin for Card<T> {}
