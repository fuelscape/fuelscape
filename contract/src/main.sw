contract;

use std::storage::StorageMap;
use std::chain::auth::AuthError;
use std::chain::auth::msg_sender;
use std::logging::log;

struct Locked {
    player: Address,
}

struct Unlocked {
    player: Address,
}

struct Given {
    player: Address,
    item: u64,
    amount: u64,
}

struct Taken {
    player: Address,
    item: u64,
    amount: u64,
}

struct Sent {
    sender: Address,
    receiver: Address,
    item: u64,
    amount: u64,
}

abi FuelScape {
    // lock is called by the admin to lock a player's wallet.
    #[storage(read, write)]
    fn lock(player: Address);

    // unlock is called by the admin to unlock a player's wallet.
    #[storage(read, write)]
    fn unlock(player: Address);

    // give is called by the admin account to give a number of items to a player.
    #[storage(read, write)]
    fn give(player: Address, item: u64, amount: u64) -> u64;

    // take is called by the admin account to take a number of items from a player.
    #[storage(read, write)]
    fn take(player: Address, item: u64, amount: u64) -> u64;

    // transfer is called by a player to transfer items to another player.
    #[storage(read, write)]
    fn send(to: Address, item: u64, amount: u64);
}

// ADMIN represents the admin wallet of the backend service, which can mint kills.
const ADMIN = ~Address::from(0x688422a9abd94f79248f62d7c7f61be1c7f13eda365dfb20b57f3ecc638456fd);

storage {
    // players holds a list of players and whether they are locked
    players: StorageMap<Address, bool> = StorageMap{},
    // items maps an address an an item ID to an amount of add-on items.
    items: StorageMap<(Address, u64), u64> = StorageMap{},
}

impl FuelScape for Contract {
    #[storage(read, write)]
    fn lock(player: Address) {
        let result = msg_sender();
        match result.unwrap() {
            Identity::Address(address) => assert(address == ADMIN),
            _ => revert(0),
        };

        let locked = storage.players.get(player);
        assert(!locked);

        storage.players.insert(player, true);

        log(Locked{ player: player });
    }

    #[storage(read, write)]
    fn unlock(player: Address) {
        let result = msg_sender();
        match result.unwrap() {
            Identity::Address(address) => assert(address == ADMIN),
            _ => revert(0),
        };

        let locked = storage.players.get(player);
        assert(locked);

        storage.players.insert(player, false);

        log(Unlocked{ player: player });
    }

    #[storage(read, write)]
    fn give(player: Address, item: u64, amount: u64) -> u64 {
        assert(amount > 0);

        let result = msg_sender();
        match result.unwrap() {
            Identity::Address(address) => assert(address == ADMIN),
            _ => revert(0),
        };

        let balance = storage.items.get((player, item));
        storage.items.insert((player, item), balance + amount);

        log(Given{ player: player, item: item, amount: amount });

        return balance + amount;
    }

    #[storage(read, write)]
    fn take(player: Address, item: u64, amount: u64) -> u64 {
       assert(amount > 0);

        let result = msg_sender();
        match result.unwrap() {
            Identity::Address(address) => assert(address == ADMIN),
            _ => revert(0),
        };

        let balance = storage.items.get((player, item));
        assert(balance >= amount);
        storage.items.insert((player, item), balance - amount);

        log(Taken{ player: player, item: item, amount: amount });

        return balance - amount;
    }

    #[storage(read, write)]
    fn send(receiver: Address, item: u64, amount: u64) {
        assert(amount > 0);

        let result = msg_sender();
        let sender = match result.unwrap() {
            Identity::Address(address) => address,
            _ => revert(0),
        };

        let debited = storage.items.get((sender, item));
        assert(debited >= amount);
        storage.items.insert((sender, item), debited - amount);

        let credited = storage.items.get((receiver, item));
        storage.items.insert((receiver, item), credited + amount);

        log(Sent{ sender: sender, receiver: receiver, item: item, amount: amount });
    }
}