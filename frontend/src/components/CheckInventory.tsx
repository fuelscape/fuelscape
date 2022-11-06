import React, { useState, useEffect } from "react";
import {useParams} from "react-router-dom"
interface InventoryRaw {
  player: string;
  wallet: string;
  balances: Balance[];
}
interface Balance {
  item: number;
  balance: number;
}
function Inventory() {
     const { id } = useParams();
  // @ts-ignore
  const [inventory, setInventory] = useState<InventoryRaw>();
  let results = "someresults";
  useEffect(() => {
    const url = `https://api.fuelscape.gg/items/${id}`;
    const fetchData = async () => {
      try {
        const response = await fetch(url, {
          headers: {
            "Content-Type": "application/json",
            Accept: "application/json",
          },
          method: "GET",
        });
        const json = await response.json();
        console.log("json:", json);
        setInventory(json);
      } catch (error) {
        console.log("error", error);
      }
    };

    fetchData();
  }, []);
  console.log("inventory: ", inventory);

  return (
    <div className="Inventory-Wrapper">
      <h3>{id}'s Inventory:</h3>
      <div className="Slot-Wrapper">
        <table>
          <tr>
            <th>Item</th>
            <th>Token #</th>
            <th>Balance</th>
          </tr>
          {inventory?.balances?.map((e) => (
            <tr>
              <td>
                <img
                  src={`https://www.osrsbox.com/osrsbox-db/items-icons/${e.item}.png`}
                ></img>
              </td>
              <td>
                <p>{e?.item}</p>
              </td>
              <td>
                <p>{e?.balance}</p>
              </td>
            </tr>
          ))}
        </table>
      </div>
    </div>
  );
}

export default Inventory;