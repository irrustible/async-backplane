use std::future::Future;
mod race;
pub use race::Race;

mod futures_endless;
pub use futures_endless::FuturesEndless;

pub trait FewchoresExt: Sized + Future {
  fn race<F: Future>(self, future: F) -> Race<Self, F>;
}

impl<F: Future> FewchoresExt for F {

  fn race<G: Future>(self, future: G) -> Race<F, G> {
    Race::new(self, future)
  }
  
}
