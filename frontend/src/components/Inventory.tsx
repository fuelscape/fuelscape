import React, { useState, useEffect } from "react";

function Inventory() {
  const [inventory, setInventory] = useState("");
  let results = "someresults";

  useEffect(() => {
    const url = "https://api.fuelscape.gg/items/";

const body = `{
    "wallet": "fuel1cvwcrwsl09krl0dfqtttcemht483yux3t0mmh8xmpaz26edn6rrszg706m",
    "item": 1323,
    "amount": 1
}`
    const fetchData =async() => {
      
      try {
        const response = await fetch(url, {
           method: "post", mode: 'no-cors',
          headers: {"Content-Type": "application/json; charset=UTF-8" ,"Accept": "*/*"},
          body:body
        }).then(response => console.log(response)).then(data => {
            console.log(data)
        })
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
