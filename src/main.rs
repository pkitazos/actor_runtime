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
    fn send(&self, msg: M) {
        let _ = self.mb.send(msg);
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

enum Pinger {
    Idle,
    Murderous,
    Dead,
}

impl Pinger {
    fn send_to(to: Addr<PingMsg>, msg: PingMsg) {
        to.send(msg);
    }
}

enum PingMsg {
    Ping { reply_to: Addr<PongMsg> },
    DeadlyPing,
}

impl Behaviour for Pinger {
    type Msg = PongMsg;
    fn apply(&mut self, msg: Self::Msg, self_addr: &Addr<Self::Msg>) -> Effect {
        let PongMsg::Pong { reply_to } = msg;

        match self {
            Pinger::Idle => {
                println!("[Pinger] Got pong. If bro says one more word..");
                reply_to.send(PingMsg::Ping {
                    reply_to: self_addr.clone(),
                });
                *self = Pinger::Murderous;
                Effect::Continue
            }
            Pinger::Murderous => {
                reply_to.send(PingMsg::DeadlyPing);
                println!("[Pinger] Got pong. I must now end this mf");
                *self = Pinger::Dead; // dies from guilt
                Effect::Continue
            }
            Pinger::Dead => Effect::Die,
        }
    }
}

enum Ponger {
    Idle,
    Dead,
}

impl Ponger {
    fn send_to(to: Addr<PongMsg>, msg: PongMsg) {
        to.send(msg);
    }
}
enum PongMsg {
    Pong { reply_to: Addr<PingMsg> },
}

impl Behaviour for Ponger {
    type Msg = PingMsg;
    fn apply(&mut self, msg: Self::Msg, self_addr: &Addr<Self::Msg>) -> Effect {
        match (&*self, msg) {
            (Ponger::Idle, PingMsg::Ping { reply_to }) => {
                println!("[Ponger] Got ping!");
                reply_to.send(PongMsg::Pong {
                    reply_to: self_addr.clone(),
                });
                *self = Ponger::Idle;
                Effect::Continue
            }
            (Ponger::Idle, PingMsg::DeadlyPing) => {
                println!("[Ponger] Got pint! Man, this ping looks funny...");
                *self = Ponger::Dead;
                Effect::Die
            }
            _ => Effect::Die, // already dead or unexpected message
        }
    }
}

#[tokio::main]
async fn main() {
    let pinger = spawn_actor(Pinger::Idle);
    let ponger = spawn_actor(Ponger::Idle);

    Pinger::send_to(ponger, PingMsg::Ping { reply_to: pinger });

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
}
