# 🚀 Suitdrop Protocol - A Decentralized Merchandise Distribution System 🛰️

Welcome to the Suitdrop protocol, a CosmWasm smart contract system designed to automate the minting and burning of fungible tokens redeemable for physical merchandise. This presents a robust and novel framework for engaging a new class of interactions between developers, validators, and contributors.

## 📝 Table of Contents
1. [🚀 Suitdrop Protocol - A Decentralized Merchandise Distribution System 🛰️](#-suitdrop-protocol---a-decentralized-merchandise-distribution-system-️)
   1. [📝 Table of Contents](#-table-of-contents)
   2. [🌐 Overview](#-overview)
   3. [📚 Definitions](#-definitions)
   4. [⚙️ Suitdrop Primitives](#️-suitdrop-primitives)
   5. [🗓️ Roadmap](#️-roadmap)
   6. [💰 Tokenomics](#-tokenomics)
   7. [❓ FAQs](#-faqs)
   8. [🏁 Getting Started](#-getting-started)
   9. [🤝 Contribute](#-contribute)

## 🌐 Overview

Suitdrop, a composite of "spacesuit + lockdrop", enables the creation of exclusive merchandise drops associated with non-fungible tokens (NFTs). Leveraging the power of the Osmosis blockchain and CosmWasm contracts, this protocol is primed to become the preferred AMM for future suitdrops. 

## 📚 Definitions
**suitdrop:** 🛸 A generic term referring to merchandise drops created by any individual or team using these primitives to deploy exclusive merchandise.

**Suitdrop:** 🚀 Refers to the brand associated with this protocol.

## ⚙️ Suitdrop Primitives
The Suitdrop protocol features two essential CosmWasm contracts:

1. `cw-bonding-pool`: 🔄 A contract for creating a bonding curve between Token A (representing physical merch) and $OSMO as the base token.

2. `cw721-suit`: 🎽 This contract automates the minting/burning of fungible tokens. 

3. `suitdrop-redeem`: 🎁 This subdirectory includes the source code for the redeem contract which handles the redemption mechanism for Suitdrop tokens.

## 🗓️ Roadmap
The proposed roadmap for implementing Suitdrop is as follows:
- 📝 Suitdrop proposal goes onchain
- 📸 Snapshot taken
- 🎽 $SHIRT drop (Bonding Curve Opens)
- 🚀 Future TBD (ETA: Q1/Q2 '24): Suitdrop-as-a-Service

## 💰 Tokenomics
The first suitdrop, $SHIRT, is an exclusive, limited-edition tee designed by YOU! In total, 500 $SHIRT tokens will be minted, with 100 allocated for free to the top 100 qualifying stakers. 

## ❓ FAQs
- [Why is it called "suitdrop"?](./FAQ.md#why-suitdrop)
- [What is Suitdrop-as-a-Service?](./FAQ.md#what-is-saas)

## 🏁 Getting Started
To get started with Suitdrop, navigate to the respective contract folders in the repository:
- [`cw-bonding-pool`](./contracts/cw-bonding-pool/README.md)
- [`cw721-suit`](./contracts/cw721-suit/README.md)
- [`suitdrop-redeem`](./contracts/suitdrop-redeem/README.md)

## 🤝 Contribute
Pull requests to the Suitdrop protocol are always welcomed!

Don't forget to #signtheshirt!

