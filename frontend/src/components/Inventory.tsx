import React, { useState, useEffect } from "react";
interface InventoryRaw {
    player: string,
  balances: Balances
}
interface Balances {
itemID: number,
amount: number
}
function Inventory() {
    // @ts-ignore
  const [inventory, setInventory] = useState<InventoryRaw>();
  let results = "someresults";
 useEffect(() => {
    const url = "https://api.fuelscape.gg/items/fuel1cvwcrwsl09krl0dfqtttcemht483yux3t0mmh8xmpaz26edn6rrszg706m";
    const fetchData = async () => {
      
      try {
        const response = await fetch(url, {
          headers: {'Content-Type': 'application/json', 'Accept': 'application/json'},
          method: "GET",
        });
        const json = await response.json();
        console.log("json:",json);
        setInventory(json);
      } catch (error) {
        console.log("error", error);
      }
    };

    fetchData();
  }, []);

  let balanceArray = inventory?.balances ? Object.entries(inventory.balances) : null
  console.log("balanceArr",balanceArray)
  return (
    <>
      <div className="Inventory-Wrapper">
        <h1>Inventory:</h1>
          {/* @ts-ignore */}
        {balanceArray?.map(e => <div className="Inventory-Slot">{e?.map(e => <p>{e}</p>)}</div>)}
      </div>
    </>
  );
}

export default Inventory;
