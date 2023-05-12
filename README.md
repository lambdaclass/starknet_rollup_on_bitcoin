# Barknet
Soverign rollup based on Rollkit, Cairo VM for the application layer and Bitcoin as a DAL having native BRC-20 tokens using a one-way bridge


## How to run

Note: This requires `bitcoin-cli` and `bitcoind` to be installed. See [the original guide](https://rollkit.dev/docs/tutorials/bitcoin/) for more information.

### DA Layer

What we need to do to run this is generate a wallet and run the daemon. For this, run

```sh
make bitcoin
```

This runs `./bitcoin/start-daemon.sh` and `./bitcoin/run.sh`. Bitcoin acts as the DA layer.

### Sequencer (app layer)

On another terminal, run the ABCI.

```sh
make abci
```

### Rollkit

If Tendermint is not installed, install and initialize it. This will initialize the required files that rollkit will use when running:

```sh
make consensus_install
bin/tendermint init
```

Notice you can also eventually use Tendermint for running it as a consensus mechanism alongside the sequencer ABCI (see following section).

Build and run Rollkit with Bitcoin DA layer.

```sh
# requires md5sum 
make rollkit_bitcoin
```

### Sequencer (app layer)

```sh
make abci
```

At this point you have a DA layer, the application layer (sequencer ABCI) and rollkit running as a replacement for Tendermint.
