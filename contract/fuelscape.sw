contract;

use std::{auth::{AuthError, msg_sender}, logging::log};

// Minted is the mint event emitted when a new mob kill is minted.
struct Minted {
    wallet: Address,
    mob: u64,
}

// Burned is the burn event emitted when mob kills are burned.
struct Burned {
    wallet: Address,
    mob: u64,
    amount: u64,
}

// Crafted is the craft event emitted when a new add-on item is crafted.
struct Crafted {
    wallet: Address,
    item: u64,
}

// Sent is the send item emitted when add-on items are transfered.
struct Sent {
    sender: Address,
    receiver: Address,
    item: u64,
    amount: u64,
}

// Kill represents a number of kills of the same mob type.
struct Kill {
    mob: u64,
    amount: u64,
}

abi FuelScape {
    // mintKill mints a mob kill of the given mob ID for the player with the
    // given wallet address.
    #[storage(read, write)]
    fn mintKill(wallet: Address, mob: u64);

    // craftItem crafts a random item for the sending wallet, consuming/burning
    // the provided mob kills.
    #[storage(read, write)]
    fn craftItem(kills []Kill);

    // sendItem sends the given amount of the given item ID to the given
    // receiver from the sending wallet.
    #[storage(read, write)]
    fn sendItem(receiver: Address, item: u64, amount: u64);
}

// ADMIN represents the admin wallet of the backend service, which can mint kills.
const ADMIN = Address::from(0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b);

storage {
    // kills maps an address an mob ID to an amount of mob kills.
    kills: StorageMap<(Address, u64), u64> = StorageMap {},
    // items maps an address an an item ID to an amount of add-on items.
    items: StorageMap<(Address, u64), u64> = StorageMap{},
}

impl FuelScape for Contract {
    #[storage(read, write)]
    fn mintKill(wallet: Address, mob: u64) {
        let result: Result<Identity, AuthError> = msg_sender();
        match sender.unwrap() {
            Identity::Address(address) => assert(address == ADMIN),
            _ => revert(0),
        };

        let amount = storage.balances.get(wallet);
        storage.kills.insert((wallet, mob), amount+1);

        log(Minted{ wallet: wallet, mob: mob });
    }

    #[storage(read, write)]
    fn craftItem(kills Vec<Kill>) {
        assert(kills.len() > 0);

        let result: Result<Identity, AuthError> = msg_sender();
        let sender = match sender.unwrap() {
            Identity::Address(address) => address,
            _ => revert(0),
        };

        let mut i = 0;
        let mut total = 0;
        while i < v.len() {
            let kill = kills.get(i).unwrap();
            let sender_amount = storage.kills.get((sender, kill.wallet));
            assert(amount >= kill.amount);

            storage.kills.insert((sender, kill.mob), sender_amount - amount);
            total = total + kill.amount;

            log(Burned{ wallet: sender, mob: kill.mob, amount });
        }

        let amount = storage.items.get((sender, total));
        storage.items.insert((sender, total), amount + 1);

        log(Crafted{ wallet: sender, item: total });
    }

    #[storage(read, write)]
    fn sendItem(receiver: Address, item: u64, amount: u64) {
        let result: Result<Identity, AuthError> = msg_sender();
        let sender = match sender.unwrap() {
            Identity::Address(address) => address,
            _ => revert(0),
        };

        let sender_amount = storage.balances.get((sender, item));
        assert(sender_amount >= amount);
        storage.items.insert((sender, item), sender_amount - amount);

        let receiver_amount = storage.balances.get((receiver, item));
        storage.items.insert((receiver, item), receiver_amount + amount);

        log(Sent{ sender: sender, receiver: receiver, amount: amount });
    }
}