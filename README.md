# relay-tester

A relay test suite for nostr relay implementations

## How to use

WARNING: This package is not meant to test a relay implementation, not a live relay:

- It will generate a bunch of events that are generally useless outside of the test.
- It might crash some relay implementations.
- The private key you choose to use for testing is not handled with care (make one up!)
- We presume the relay is close (in terms of latency) and not busy, and that a one-second
  timeout is plenty to determine if the relay won't be replying.

### STEP 1

Setup a fresh install of the relay for testing, with no events yet.

### STEP 2

Generate a keypair. We have included a binary "generate_keypair" so you can do this.

### STEP 3

Configure the relay to allow the public key to have full access to the relay (in case
the relay isn't already fully open to the public).

### STEP 4

Run `relay-tester <url> <nsec>`
