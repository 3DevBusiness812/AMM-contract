# Anchor Example: Automated Market Maker

## Build, Deploy and Test

#### Please be patience, running all the test case will take time

First, install dependencies:

```
$ yarn
```

Next, we will build and deploy the program via Anchor.

Get the program ID:

```
$ anchor keys list
amm: ALEhJzSgVas922auhWPu5ettH8QH4RiccZUvrjQ1UUWh
```

Make sure you update your program ID in `Anchor.toml` and `lib.rs`.

Build the program:

```
$ anchor build
```

State Solana Test Validator

```
$ solana-test-validator
```

Let's deploy the program on localnet

```
$ anchor deploy
...

Program Id: ALEhJzSgVas922auhWPu5ettH8QH4RiccZUvrjQ1UUWh

Deploy success
```

Finally, run the test:

```
$ anchor run amm
```
