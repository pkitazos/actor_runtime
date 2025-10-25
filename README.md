# tiny Actor Runtime

Experimenting building a simple actor runtime in Rust.


## What do we want out of our runtime?

Let's say _you_ are a thread. In this world that means you are also an actor! As an actor you can:
- [ ] spawn an actor and get back a PID
- [ ] send messages to an actor if you know their PID
- [ ] wait to receive messages in your mailbox
- [ ] spawn and link to an actor. If one of you dies, you both die
- [ ] supervise other actors. If one of your supervisees exits, you will hear about it

Another guarantee we need the runtime to provide is that messages sent by one actor to another will always arrive in the same order they were sent relative to each other. There are no guarantees about the ordering of messages of different actors sending messages to the same actor. The dependency exists between messages sent by _the same_ actor.
