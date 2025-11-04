use std::{any::Any, sync::Arc};
use tokio::sync::mpsc::{self, UnboundedSender};

struct Env;

enum Next<B> {
    Stay(B),
    Become(B),
    Stop,
}

#[derive(Clone)]
struct Addr<M> {
    mb: UnboundedSender<Envelope<M>>,
}

impl<M> Addr<M> {
    fn send(&self, envelope: Envelope<M>) {
        let _ = self.mb.send(envelope);
    }
}

#[derive(Clone)]
struct ErasedAddr {
    // unless we know ahead of time exactly what actors we are going to be communicating with,
    // we would not be able to properly type the address that will be included with every message we receive
    // but we know that an address is something that can be sent a message of some shape
    // so we can represent it as a struct with a send field which is a function pointer that sends some thing
    send: Arc<dyn Fn(Box<dyn Any + Send>) + Send + Sync>,
}

impl<M: 'static + Send> From<Addr<M>> for ErasedAddr {
    fn from(a: Addr<M>) -> ErasedAddr {
        ErasedAddr {
            send: Arc::new(move |boxed| {
                if let Ok(envelope) = boxed.downcast::<Envelope<M>>() {
                    let _ = a.send(*envelope);
                } else {
                    // tried to send a message this actor cannot receive
                    // log it or something
                }
            }),
        }
    }
}

struct Envelope<M> {
    msg: M,
    addr: ErasedAddr,
}

trait Behaviour {
    type Msg: Send + 'static;

    fn self_addr(self) -> Addr<Self::Msg>;

    fn on(self, envelope: Envelope<Self::Msg>, env: &mut Env) -> Next<Self>
    where
        Self: Sized;
}

fn spawn_actor<B>(initial: B) -> Addr<B::Msg>
where
    B: Behaviour + Send + 'static,
{
    let (tx, mut rx) = mpsc::unbounded_channel::<Envelope<B::Msg>>();
    let addr = Addr { mb: tx };

    tokio::spawn(async move {
        let mut state = initial;
        let mut env = Env;

        while let Some(envelope) = rx.recv().await {
            state = match state.on(envelope, &mut env) {
                Next::Stay(s) | Next::Become(s) => s,
                Next::Stop => break,
            };
        }
    });

    addr
}

// ---

fn main() {}
