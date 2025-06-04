# Private Route Example

Demonstration of the creation of private routes in a basic client/server style pattern.

## Running this example

To use, start a *server* that creates a private route:

```
# cargo run --example private-route-example
```
which will produce output like this:
```
Veilid Private Routing Example
Waiting.................
Route id created: lRETsplk7wWg2QiF7OodCOSC9SbhFx9tyWtk2KVHs70
Connect with this private route blob:
cargo run --example private-route-example -- --connect ARAeUAECAQJRBAEBURgBAg8wRExWEAT/NZxHC0sOm9oDPYts06hETjdBFhQ7akbgwU9cR/+HIODlAAARBARBEAL/XwgITBMjGe8DIS8kAw0VyKcu6d65YES+lapmgYaJmK8SEQQDMQ3yAf9mmhQquLvXsgm1h/MEQma0jQEJmg6dajJdNqQ7EZjyViM9Vp+zA/e2/QG8xyRiMmDKJuJ6ixQ+CW8HQcSSKdgp0wC0ATUrZTTCQmZ8m/BWlJ8/0qySnSFj
Press ctrl-c when you are finished.
```

Then, in a second terminal, or on another machine, run a *client* that connects to this private route:

```
# cargo run --example private-route-example -- --connect ARAeUAECAQJRBAEBURgBAg8wRExWEAT/NZxHC0sOm9oDPYts06hETjdBFhQ7akbgwU9cR/+HIODlAAARBARBEAL/XwgITBMjGe8DIS8kAw0VyKcu6d65YES+lapmgYaJmK8SEQQDMQ3yAf9mmhQquLvXsgm1h/MEQma0jQEJmg6dajJdNqQ7EZjyViM9Vp+zA/e2/QG8xyRiMmDKJuJ6ixQ+CW8HQcSSKdgp0wC0ATUrZTTCQmZ8m/BWlJ8/0qySnSFj
```

Now, you can send lines from the *client* to the *server* and the will show up in the *server*'s terminal:

```
AppMessage received: testing 1 2 3
AppMessage received: yay it works
```

## Note

You may notice spurious route failures and errors in the console of this test program.

For example:

```
VeilidRouteChange { dead_routes: [RouteId(Osifkt3Q3j6O5x03o85iBtpw8sBe5gUhLQW1n6bd7Ws)], dead_remote_routes: [] }
```

This example is a work in progress and is being used as a testbed to improve the quality of Veilid's private routing API. While there exist ways to stabilize the existing private routing mechanism, they are beyond the scope of this example code. Rather, we are working on putting all of the required logic into veilid-core itself. Once this is done, this example will be updated, and this note will be removed.
