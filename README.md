# ScPull
Pulls smart contract code from etherscan and creates a new foundry project for it. 

## Installation

You can install ScPull via cargo.
`cargo install scpull`

## Usage

ScPull can be used in the following way:
`scpull <CHAIN> <ADDRESS> <PATH>`

Where chain is either an alias or a chainid and address is the address of the smart contract.

### Aliases
ScPull supports aliases for selecting the chain. Supported aliases are the following. 
```
    eth: Ethereum
    op: Optimism
    bsc: Binance Smart Chain
    poly: Polygon
    base: Base
    arb: Arbitrum
    lin: Linea
    linea: Linea
    era: ZkSync Era
    zksync: ZkSync Era
```
