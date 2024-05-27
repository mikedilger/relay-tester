# relay-tester

A relay test suite for nostr relays

## How to use

### STEP 1

Setup the relay for testing.

WARNING: This package is not meant to test a relay implementation, not a live relay:

- We take no precautions to prevent flooding or filling a relay with junk
- We take no precautions around the private key used to test the relay implementation

### STEP 2

Generate a keypair. We have included a binary "generate_keypair" so you can do this.

### STEP 3

Configure the relay to allow the public key to have full access to the relay (in case
the relay isn't already fully open to the public).

### STEP 4

Run `relay-tester <url> <nsec>`
