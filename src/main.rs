use std::ops::Add;

use tokio::sync::mpsc::{self, UnboundedSender};

struct Addr<M> {
    mb: UnboundedSender<M>,
}

impl<M> Clone for Addr<M> {
    fn clone(&self) -> Self {
        Addr {
            mb: self.mb.clone(),
        }
    }
}

impl<M> Addr<M> {
    fn send(&self, envelope: M) {
        let _ = self.mb.send(envelope);
    }
}

// a behaviour just takes a message and returns some effect
trait Behaviour: Send + 'static {
    type Msg: Send + 'static;

    fn apply(&mut self, msg: Self::Msg, self_addr: &Addr<Self::Msg>) -> Effect;
}

enum Effect {
    Continue, // either continue doing something
    Die,      // or die
}

fn spawn_actor<B>(initial: B) -> Addr<B::Msg>
where
    B: Behaviour + Send + 'static,
{
    let (tx, mut rx) = mpsc::unbounded_channel::<B::Msg>();
    let addr = Addr { mb: tx };
    let addr_copy = addr.clone();

    tokio::spawn(async move {
        let mut state = initial;

        while let Some(msg) = rx.recv().await {
            match state.apply(msg, &addr_copy) {
                Effect::Continue => continue,
                Effect::Die => break,
            };
        }
    });

    addr
}

// ---

fn main() {}
