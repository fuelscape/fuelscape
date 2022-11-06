import React, { useState, useEffect } from "react";

function Inventory() {
  const [inventory, setInventory] = useState("");
  let results = "someresults";

  useEffect(() => {
    const url = "http://127.0.0.1:8080/items/";
const body = {
    'wallet': "fuel1cvwcrwsl09krl0dfqtttcemht483yux3t0mmh8xmpaz26edn6rrszg706m",
    'item': 1323,
    'amount': 1
}
    const fetchData = async () => {
      
      try {
        const response = await fetch(url, {
            mode: 'no-cors',
          headers: {'Accept': '*/*',
      'Content-Type': 'application/json'},
          method: "POST",
          body: JSON.stringify(body),
        });
        const json = await response.json();
        console.log(json);
        setInventory(json);
      } catch (error) {
        console.log("error", error);
      }
    };

    fetchData();
  }, []);
  return (
    <>
      <div className="Inventory-Wrapper">
        <h1>Inventory:</h1>
        <p>{results}</p>
      </div>
    </>
  );
}

export default Inventory;
