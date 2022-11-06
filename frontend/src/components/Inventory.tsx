import React, { useState, useEffect } from "react";
interface InventoryRaw {
    player: string,
    wallet: string,
  balances: Balance[]
}
interface Balance {
item: number,
balance: number
}
function Inventory() {
    // @ts-ignore
  const [inventory, setInventory] = useState<InventoryRaw>();
  let results = "someresults";
 useEffect(() => {
    const url = "https://api.fuelscape.gg/items/test1";
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
  console.log("inventory: ", inventory)


  return (
   
      <div className="Inventory-Wrapper"><h1>Inventory:</h1>
      <div className="Slot-Wrapper">
      <table>
  <tr>
    <th>Icon</th>
    <th>Id</th>
    <th>Name</th>
    <th>Balance</th>
  </tr>
       {inventory?.balances?.map(e => <tr><td><img src={`https://www.osrsbox.com/osrsbox-db/items-icons/${e.item}
      .png`}></img></td><td><p>{e?.item}</p></td><td><p>nombre</p></td><td><p>{e?.balance}</p></td></tr>)}
</table>
      </div>
      </div>
    
  );
}

export default Inventory;
