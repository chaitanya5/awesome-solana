# Token Vault

A simple solana program which demostrates how users could deposit and withdraw their tokens to and from a smart contract ie, a token vault.

## Objective
![Diagram](../public/token-escrow.svg)

## Program Overview
The program can be thought of as a "vault" system—though more accurately, it creates a vault (also called an Associated Token Account, or ATA) to hold tokens of each type.
## Key mechanics
1. ### Vault Authorization
    - Each vault (ATA) is authorized by a Program-Derived Address (PDA), derived from the token’s mint account and the program ID.
1. ### User-Specific Tracking
    - Every user has a dedicated state PDA that records:
        * The token mint they’ve deposited.
        * The amount of tokens they’ve deposited.