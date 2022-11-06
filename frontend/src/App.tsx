import React, { useState, useEffect } from "react";
import logo from "./logo.svg";
import "./App.css";
import { Wallet, WalletUnlocked } from "@fuel-ts/wallet";
import { AbstractWallet } from "@fuel-ts/interfaces";
import { WalletManager } from "@fuel-ts/wallet-manager";
import { Routes, Route, Link } from "react-router-dom";
import Market from "./components/Market";
import Inventory from "./components/Inventory";

function App() {
  const localWallet = localStorage.getItem("ActiveWallet");
  const pvtKey = localStorage.getItem("PvtKey");
  const [ActiveWallet, setActiveWallet] = useState<WalletUnlocked>();
  const [isReturnUser, setReturnUser] = useState(false);
  const manager = new WalletManager();
  console.log("localWallet", localWallet);
  console.log("pvtKey", pvtKey);

  useEffect(() => {
    if (localWallet && pvtKey) {
      setActiveWallet(
        new WalletUnlocked(
          "0x9d74ebdca29148547e0dd37e30adfec3e7988d061e7435892be934ef6809b190"
        )
      );
      setReturnUser(true);
      console.log("activewallet: ", ActiveWallet);
    }
  }, []);

  let createNewWallet = () => {
    let newWallet = Wallet.generate();
    setActiveWallet(newWallet);
    localStorage.setItem("ActiveWallet", newWallet.address.toString());
    localStorage.setItem("PvtKey", newWallet.privateKey);
  };

  let copyPvtKey = async () => {
    if (ActiveWallet?.address) {
      await window.navigator.clipboard.writeText(ActiveWallet?.privateKey);
      alert("Copied pvt key to clipboard ");
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
                {isReturnUser && <p> Welcome Back! </p>}
                {ActiveWallet && (
                  <>
                    <p> Active Wallet Address:</p>{" "}
                    <p>{JSON.stringify(ActiveWallet.address).slice(4, -1)}</p>
                  </>
                )}
                <button onClick={() => createNewWallet()}>New Wallet</button>
                <button onClick={() => copyPvtKey()}> Export Pvt Key</button>
              </div>
            }
          />
          <Route path="market" element={<Market />} />
          <Route path="inventory/:id" element={<Inventory />} />
        </Routes>
      </header>
    </div>
  );
}

export default App;
