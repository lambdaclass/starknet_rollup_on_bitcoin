# Barknet PoC

## Overview
Barknet is a PoC soverign rollup based on Rollkit and Starknet in Rust (Cairo VM) for the application layer over Bitcoin as the Data availability layer, allowing BRC-20 tokens to be used as Starknet tokens via a burn/bridge process.

### Design

The design is based on watching specific burn transactions on Bitcoin, and decoding which BRC-20 assets they represent. Whenever a burn is detected, a mint for an ERC-20 compatible token occurs on the rollup's sequencer (Rollkit) which updates the Starknet state object, allowing for Cairo applications (such as AMMs, swaps, etc.) to run on top of it.

## How to run

Requirements:

- `bitcoin-cli` and `bitcoind` have to be installed. See [this guide for running Rollkit with Bitcoin](https://rollkit.dev/docs/tutorials/bitcoin/) for more information.
- `Python3.9` needs to be installed.
- `GMP Library` should be available. You can install it on:
    - Ubuntu ```sudo apt install -y libgmp3-dev```
    - MacOS ```brew install gmp```
- `starknet-compile` needs to be available in order to compile both Cairo contracts (`erc20_mintable.cairo` and `amm.cairo`). There is a make target for compiling the contracts: `make compile-starknet`

### DA Layer

What we need to do to run this is generate a wallet and run the daemon. For this, run

```sh
make bitcoin
```

Which runs `./bitcoin/start-daemon.sh` and `./bitcoin/run.sh`. This starts `bitcoind` in regtest and creates (and selects) a wallet, alongside starting a mining task.
We need this in order to post our Rollkit blocks to Bitcoin as configured in our Golang project (located in `/rollkit-node-bitcoin`).

### Sequencer (app layer)

On another terminal, run the ABCI.

```sh
make abci
```

The ABCI is the binary which interfaces with systems such as Tendermint Core or Rollkit and defines the state transitions that are consequence of the transactions that our blockchain nodes receive.

### Rollkit

If Tendermint Core is not installed, install and initialize it. This will initialize the required files that Rollkit will use when running:

```sh
make consensus_install
bin/tendermint init
```

Notice we could also eventually use Tendermint for running alongside our ABCI as a standalone blockchain consensus mechanism.

After we have this, we can build and run our Rollkit project configured with Bitcoin as the DA layer.

```sh
# requires md5sum 
make rollkit_bitcoin
```

At this point you should have a DA layer running alongside the application layer (ABCI) and Rollkit acting as the sequencer for incoming transactions. You could now run the binary which watches Bitcoin's transactions and decodes the witness into BRC-20 data in order to send mint transactions to the sequencer (`cargo run --bin watcher`). 

### Cairo

Barknet only works with `Cairo 0`, this means cairo-lang should be installed. To do so run:

```sh
make install cairo
```

After the installation is done, `starknet-compile` will be available in order to compile both Cairo contracts (`erc20_mintable.cairo` and `amm.cairo`). There is a make target for compiling the contracts: `make compile-starknet`

## Limitations and future work

For now, Rollkit only runs as a single sequencer. More sequencing modes are planned for the roadmap, but this means that for now this project can only run as a centralized sequencer instead of having a set of multiple nodes which would mean more liveness. Furthermore, there is currently no way to have light nodes that can validate transactions accordingly. Read more about these limitations in Rollkit's [roadmap blog post](https://rollkit.dev/blog/introducing-rollkit/#vision-for-rollkit).

The design of the system involves running a binary that looks into recently-burned Bitcoin assets in order to send minting transactions to the sequencer. This implies a risk since, even when we can verify transactions through signatures, there is a possibility to get hacked and have arbitrary minting. For this we spent some time looking into the ABCI++ protocol to couple block production with minting according to what we saw on Bitcoin, but were not successful since the related methods are currently not supported on Rollkit (there are some experimental branches which we were not able to run correctly).
