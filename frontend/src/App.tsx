import React, {useState, useEffect} from "react";
import logo from "./logo.svg";
import "./App.css";
import { Wallet } from "fuels";

function App() {
  const localWallet = localStorage.getItem('ActiveWallet')
  const [ActiveWallet, setActiveWallet] = useState('');
  const [isReturnUser, setReturnUser] = useState(false);
  console.log("localWallet", localWallet)

   useEffect(() => {
    if (localWallet){
setActiveWallet(localWallet);
setReturnUser(true);
    }
  
  },[]);
  
// if (localWallet !== (null && ActiveWallet)){setActiveWallet(localWallet)};
  let newWallet = () => {
    let newWallet = JSON.stringify(Wallet.generate().address)
   setActiveWallet(newWallet);
   localStorage.setItem('ActiveWallet', newWallet);
  }
  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <button onClick={() =>  newWallet()}>
          Generate Random Wallet
        </button>
        {isReturnUser && <p> Welcome Back! </p>}
        {ActiveWallet && <p>{ActiveWallet}</p>}
        
      </header>
    </div>
  );
}

export default App;
