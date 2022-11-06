import React, { useState, useEffect } from "react";
import {useParams} from "react-router-dom"
function Inventory(props: any) {
  console.log(props);
  // @ts-ignore
  return (
    <div className="Inventory-Wrapper">
      <h1>Inventory:</h1>
      <div className="Slot-Wrapper">
        <table>
          <tr>
            <th>Icon</th>
            <th>Id</th>
            <th>Balance</th>
          </tr>
          {props.props?.map((e: any) => (
            <tr key={e}>
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
