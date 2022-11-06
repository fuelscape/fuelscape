# FuelScape

## Overview

FuelScape is a fork of the open source RuneScape Classic framework (Open-RSC) server to integrate the game mechanics with the Fuel VM.

The system is made up of the following components:

- A frontend application to:
  - generate a Fuel wallet address;
  - export the associated private key;
  - view player name and inventory.
- A backend service to:
  - give and take items from players during game play;
  - lock and unlock wallets when players log in / log out;
  - provide player name for wallet address;
  - provide player on-chain inventory by player name.
- A modified game server to:
  - notifiy backend of logins and logouts; and
  - notify backend of inventory changes.
- A Fuel smart contract to:
  - mint and burn inventory items as NFTs;
  - enabled / disable wallet transfers by address; and
  - transfer inventory NFTs between wallets.

## Frontend Application

The frontend application should allow a player to opt into the add-on inventory system.

In order to do so, the user should:

1. generate a wallet and back up private key
2. associate the wallet address with his player name
3. display inventory for the player's wallet

### Generate Wallet

Generating wallet is basically equivalent to generating a private key, storing it in browser storage for persistance and creating the wallet object with the Fuel TypeScript SDK.

1. Generate private key as hex-encoded string:
   - [`generatePrivateKey(entropy?)`](https://fuellabs.github.io/fuels-ts/packages/fuel-ts-signer/classes/Signer.html#generateprivatekey)
2. Store private key in browser local storage
3. Create wallet object:
   - [`new Wallet(privateKey, provider?)`](https://fuellabs.github.io/fuels-ts/packages/fuel-ts-wallet/classes/Wallet.html#constructor)

### Link Wallet

Linking the wallet to a player is done by calling a route on the REST API of our backend service, which will take care of the associations.

1. Call backend service route:
   - `POST` to `<backend_url>/links/`
   - Body of `{ "player": "<name>", "wallet": "<address>" }`

### List Inventory

All mob kills and add-on inventory items are represented by tokens on the chain.
Fuel does not have fully developed token support yet, so we have to interact directly with the contract ABIs of the token contracts.

1. Load the ABI for the smart contract
2. Use hard-coded Contract ID, ABI and wallet to initialize interface:
   - [`new Contract(id, abi, walletOrProvider?`](https://fuellabs.github.io/fuels-ts/packages/fuel-ts-contract/classes/Contract.html#constructor)
3. Use the correct entry in the `functions` array of the contract object to get contract state:
   - [`functions: InvokeFunctions = {}`](https://fuellabs.github.io/fuels-ts/packages/fuel-ts-contract/classes/Contract.html#functions)

## Backend Service

### Link Wallet

One route creates a link between a player and a wallet address.
This is needed because the RuneScape server doesn't know anything about wallet addresses.
Keeping it agnostic of player's wallet addresses minimizes the changes needed to the server code.
Instead, the backend service will keep track of the link between player names and wallets.
This way, the server only needs needs to send notifications of game-related data.

Route:

`POST /links/`

Request:

```json
{
    "player": "FuelGasm",
    "wallet": "fuel1xgg70aemkfcnjyemapv24v33wa4t428ju6x5f87y648dnsu6w3hqds9m9f"
}
```

Response:

`204 Created`

=> link successfully created

No-op:

`409 Conflict`

=> the link already exists

Error:

`500 Internal Server Error`

=> any other error for now

### Mint Kill

In order to successfully mint a kill, the player name needs to be linked to a wallet.
Then, the kill is minted by ID of the mob (meaning each mob has its own kill NFTs that are fungible within themselves).

Route:

`POST /kills/`

Request:

```json
{
    "player": "FuelGasm",
    "mob": 12345
}
```

Response:

`204 Created`

=> kill successfully minted

No-op:

`404 Not Found`

=> player not associated to wallet

Error:

`500 Internal Server Error`

=> any other error

## Smart Contract

The smart contract manages to types of NFTS:

- mob kills; and
- add-on items.

For each type of mob, a wallet can hold multiple kills.
For each type of item, a wallet can hold multiple copies.

### Contract ABI

```
struct Kill {
    mob: u64,
    amount: u64,
}

abi FuelScape {
    // lock will disable item transfers for a wallet
    #[storage(read, write)]
    fn lock(player: Address);

    // unlock will enabled item transfers for a wallet
    #[storage(read, write)]
    fn unlock(player: Address);

    // give will create new inventory items by giving them to a wallet
    #[storage(read, write)]
    fn give(player: Address, item: u16, amount: u32);

    // take will delete existing inventory items by removing them from a wallet
    #[storage(read, write)]
    fn take(player: Address, item: u16, amount: u32);

    // send will transfer inventory items from one wallet to another wallet
    #[storage(read, write)]
    fn send(receiver: Address, item: u16, amount: u32)

    // Potential extensions:
    // lend: to lend an item, with possibility to reclaim it
    // reclaim: to reclaim a lent item
    // trade: to atomically trade an item for tokens
}
```