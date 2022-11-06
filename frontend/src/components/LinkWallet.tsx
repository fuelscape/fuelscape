import React, { useState, useEffect } from "react";
import {useParams} from "react-router-dom"
interface LinkWalletRaw {
  player: string;
  wallet: string;
  balances: Balance[];
}
interface Balance {
  item: number;
  balance: number;
}
function LinkWallet() {
     const { id } = useParams();
  // @ts-ignore

  const [LinkWallet, setLinkWallet] = useState<LinkWalletRaw>();
  let results = "someresults";
  let body = {player: 'test1', wallet: "fuel1cvwcrwsl09krl0dfqtttcemht483yux3t0mmh8xmpaz26edn6rrszg706m"}
  useEffect(() => {
    const url = `https://api.fuelscape.gg/links/`;
    const fetchData = async () => {
      try {
        const response = await fetch(url, {
          headers: {
            "Content-Type": "application/json",
            Accept: "application/json",
          },
          method: "GET",
          body: JSON.stringify(body)
        });
        const json = await response.json();
        console.log("json:", json);
        setLinkWallet(json);
      } catch (error) {
        console.log("error", error);
      }
    };

    fetchData();
  }, []);
  console.log("LinkWallet: ", LinkWallet);

  return (
    <div className="LinkWallet-Wrapper">
      <h1>LinkWallet:</h1>

    </div>
  );
}

export default LinkWallet;
