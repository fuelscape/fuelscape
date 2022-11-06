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
  const walletAddress = localStorage.getItem("WalletAddress");
  const pvtKey = localStorage.getItem("PvtKey");
  const playerName = localStorage.getItem("PlayerName")
  const [ActiveWallet, setActiveWallet] = useState<WalletUnlocked>();
  const [player, setPlayer] = useState(playerName);
  const [walletLinked, setWalletLinked] = useState(false);
  console.log("walletAddress", walletAddress);
  console.log("pvtKey", pvtKey);

  useEffect(() => {
    if (walletAddress && pvtKey) {
      setActiveWallet(
        new WalletUnlocked(
          pvtKey,
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
    localStorage.setItem("PvtKey", newWallet.privateKey);
  };

  let resetWallet = () => {
    setActiveWallet(undefined);
    localStorage.clear();
  };

  let linkWallet = async () => {
    let results = "someresults";
    let body = {player: player, wallet: ActiveWallet?.address.toString()}
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
      localStorage.setItem("PlayerName", player? player : "");
      console.log("json:", json);
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
                      onChange={(event) => setPlayer(event.currentTarget.value)}
                      value={playerName ? playerName : ""}
                      size="xl"
                      disabled={walletLinked}
                    />
                    <br />
                  </div>
                )}
                {!ActiveWallet && (<button onClick={() => generateWallet()}>Generate Wallet</button>)}
                {ActiveWallet && (<button onClick={() => resetWallet()}>Reset Wallet</button>)}
                {ActiveWallet && !walletLinked && (<button onClick={() => linkWallet()}>Link Wallet</button>)}
              </div>
            }
          />
          <Route path="market" element={<Market />} />
          <Route path="inventory/:id" element={<Inventory />} />
          <Route path="linkwallet:id" element={<LinkWallet/>}/>
        </Routes>
      </header>
    </div>
  );
}

export default App;
