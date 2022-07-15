
1. Get your tooling all set up (at least `rustup target add wasm32-unknown-unknown`)
1. Run ./release.sh

You can just serve this directory and load `./index.html` in a browser.

If you're dumb like me you can:

```
cargo install dsfs
./release.sh
dsfs
```

And point your browser to the URL printed on the console.

Messages are successfully sent to `Eip1193Interface`. Reading from `Eip1193Interface` results in `receiving from an empty channel`. If that means the channel doesn't have a message waiting, that's _one_ problem. If the channel is somehow "not connected" to `Eip1193Task`, that's a _different_ problem.

The `unbounded` calls in `Eip1193Plugin::build` are supposed to "hook up" the channels implicitly, so that 

```
Eip1193Interface.sender -> Eip1193Task.receiver
Eip1193Task.sender -> Eip1193Interface.receiver
```

Further, `unbounded` infers the types from the above `struct`'s member's types. `Sender<String>` and `Receiver<String>` in this case.

What I expect to happen:

Clicking the `[metamask]` button causes `interface.sender` to queue up the string `"eth_requestAccounts"` in the task that contains the `Eip1193Task` instance that lives in the `IoTaskPool.task`.

[This line](https://github.com/stnbu/bevy-web3-wasm/blob/88aaa8b71c2813a27181bff2d4066f0f2130fb2d/src/main.rs#L46) `recv`'s from `Eip1193Task.receiver` and call's `.execute()` on the eip-1193 transport. If `Ok(message)`, then `message` is sent to `Eip1193Interface.receiver` which should be received [here](https://github.com/stnbu/bevy-web3-wasm/blob/88aaa8b71c2813a27181bff2d4066f0f2130fb2d/src/main.rs#L116) `ui_example` is called once per frame; egui is ["immediate mode"](https://en.wikipedia.org/wiki/Immediate_mode_GUI).

What up with that?

----

Much code and ideas stolen from

* https://github.com/kauly/bevy-metamask
* https://github.com/kiwiyou/bevy-ws-server

The flaws are all mine.