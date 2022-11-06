import React, { useState, useEffect } from "react";
import logo from "./logo.svg";
import "./App.css";
import { Wallet, WalletUnlocked } from "@fuel-ts/wallet";
import { AbstractWallet } from "@fuel-ts/interfaces";
import { WalletManager } from "@fuel-ts/wallet-manager";
import { Routes, Route, Link } from "react-router-dom";
import Market from "./components/Market";
import Inventory from "./components/Inventory";
import LinkWallet from "./components/LinkWallet";
import { Button, TextInput, useMantineTheme } from "@mantine/core";

function App() {
  const localAddress = localStorage.getItem("WalletAddress");
  const localKey = localStorage.getItem("WalletKey");
  const localPlayer = localStorage.getItem("PlayerName")
  const [ActiveWallet, setActiveWallet] = useState<WalletUnlocked>();
  const [playerName, setPlayerName] = useState(localPlayer);
  const [walletLinked, setWalletLinked] = useState(localPlayer? true : false);
  const [balances, setBalances] = useState([]);
  console.log("localAddress", localAddress);
  console.log("localKey", localKey);
  localStorage.clear();

  useEffect(() => {
    if (localAddress && localKey) {
      setActiveWallet(
        new WalletUnlocked(
          localKey,
        )
      );
      console.log("activewallet: ", ActiveWallet);
      console.log("playername: ", playerName);
    }
  }, []);

  let generateWallet = () => {
    let newWallet = Wallet.generate();
    setActiveWallet(newWallet);
    localStorage.setItem("WalletAddress", newWallet.address.toString());
    localStorage.setItem("WalletKey", newWallet.privateKey);
  };

  let resetWallet = () => {
    setActiveWallet(undefined);
    setWalletLinked(false);
    setPlayerName(null);
    setBalances([]);
  };

  let linkWallet = async () => {
    console.log("PlayerName: " + playerName);
    let results = "someresults";
    let body = {player: playerName, wallet: ActiveWallet?.address.toString()}
    const url = `https://api.fuelscape.gg/links/`;
    try {
      const response = await fetch(url, {
        headers: {
          "Content-Type": "application/json",
          "Accept": "application/json",
        },
        method: "POST",
        body: JSON.stringify(body)
      });
      const json = await response.json();
      setWalletLinked(true);
      localStorage.setItem("PlayerName", playerName? playerName : "");
      console.log("json:", json);
    } catch (error) {
      console.log("error", error);
    }
  };

  let refreshInventory = async () => {
    const url = 'https://api.fuelscape.gg/items/' + playerName;
    try {
      const response = await fetch(url, {
        headers: {
          "Content-Type": "application/json",
          "Accept": "application/json",
        },
        method: "GET",
        });
        const json = await response.json();
        console.log("json", json);
        setBalances(json.balances);
      } catch (error) {
        console.log("error", error);
      }
  };

  return (
    <div className="App">
      <header className="App-header">
        <Routes>
          <Route
            path="/"
            element={
              <div className="Body-wrapper">
                {ActiveWallet && (
                  <div className="Input-fields">

                    <TextInput
                      type="text"
                      label="Player Address"
                      value={ActiveWallet.address.toString()}
                      size="xl"
                      disabled
                    />
                    <br />
                    <TextInput
                      type="text"
                      label="Player Name"
                      value={playerName ? playerName : ""}
                      onChange={(event) => setPlayerName(event.currentTarget.value)}
                      size="xl"
                      disabled={walletLinked}
                    />
                    <br />
                  </div>
                )}
                {!ActiveWallet && (<button onClick={() => generateWallet()}>Generate Wallet</button>)}
                {ActiveWallet && (<button onClick={() => resetWallet()}>Reset Wallet</button>)}
                {ActiveWallet && !walletLinked && (<button onClick={() => linkWallet()}>Link Wallet</button>)}
                {ActiveWallet && walletLinked && (<button onClick={() => refreshInventory()}>Refresh Inventory</button>)}
              </div>
            }
          />
          <Route path="market" element={<Market />} />
          <Route path="inventory/:id" element={<Inventory />} />
          <Route path="linkwallet:id" element={<LinkWallet/>}/>
        </Routes>
        <Inventory props={balances} />
      </header>
    </div>
  );
}

export default App;
