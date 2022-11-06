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
    balance: u64,
}

struct Taken {
    player: Address,
    item: u64,
    balance: u64,
}

struct Sent {
    sender: Address,
    receiver: Address,
    item: u64,
    amount: u64,
}

struct Entry {
    item: u64,
    balance: u64,
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

    // view will return a list of all items a user owns
    #[storage(read)]
    fn view(player: Address);

    // transfer is called by a player to transfer items to another player.
    #[storage(read, write)]
    fn send(to: Address, item: u64, amount: u64);
}

// ADMIN represents the admin wallet of the backend service, which can mint kills.
const ADMIN = ~Address::from(0x688422a9abd94f79248f62d7c7f61be1c7f13eda365dfb20b57f3ecc638456fd);

storage {
    // locks holds a list of players and whether they are locked
    locks: StorageMap<Address, bool> = StorageMap {},
    // items maps an address an an item ID to an amount of add-on items.
    balances: StorageMap<(Address, u64), u64> = StorageMap {},
}

impl FuelScape for Contract {
    #[storage(read, write)]
    fn lock(player: Address) {
        let result = msg_sender();
        match result.unwrap() {
            Identity::Address(address) => assert(address == ADMIN),
            _ => revert(0),
        };

        let locked = storage.locks.get(player);
        assert(!locked);

        storage.locks.insert(player, true);

        log(Locked { player: player });
    }

    #[storage(read, write)]
    fn unlock(player: Address) {
        let result = msg_sender();
        match result.unwrap() {
            Identity::Address(address) => assert(address == ADMIN),
            _ => revert(0),
        };

        let locked = storage.locks.get(player);
        assert(locked);

        storage.locks.insert(player, false);

        log(Unlocked { player: player });
    }

    #[storage(read, write)]
    fn give(player: Address, item: u64, amount: u64) -> u64 {
        assert(amount > 0);

        let result = msg_sender();
        match result.unwrap() {
            Identity::Address(address) => assert(address == ADMIN),
            _ => revert(0),
        };

        let balance = storage.balances.get((player, item));
        storage.balances.insert((player, item), balance + amount);

        log(Given {
            player: player,
            item: item,
            balance: balance + amount,
        });

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

        let balance = storage.balances.get((player, item));
        assert(balance >= amount);
        storage.balances.insert((player, item), balance - amount);

        log(Taken {
            player: player,
            item: item,
            balance: balance - amount,
        });

        return balance - amount;
    }

    #[storage(read)]
    fn view(player: Address) {
        let item = 0;
        while item < 32768 {
            let balance = storage.balances.get((player, item));
            if balance == 0 {
                continue;
            }
            log(Entry {
                item: item,
                balance: balance,
            });
        }
    }

    #[storage(read, write)]
    fn send(receiver: Address, item: u64, amount: u64) {
        assert(amount > 0);

        let result = msg_sender();
        let sender = match result.unwrap() {
            Identity::Address(address) => address,
            _ => revert(0),
        };

        let locked = storage.locks.get(sender);
        assert(!locked);

        let debited = storage.balances.get((sender, item));
        assert(debited >= amount);
        storage.balances.insert((sender, item), debited - amount);

        let credited = storage.balances.get((receiver, item));
        storage.balances.insert((receiver, item), credited + amount);

        log(Sent {
            sender: sender,
            receiver: receiver,
            item: item,
            amount: amount,
        });
    }
}
