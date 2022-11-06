import React, { useState, useEffect } from "react";
import {useParams} from "react-router-dom";
import { useForm } from '@mantine/form';
import { Button, TextInput, TextInputProps, ActionIcon, useMantineTheme } from '@mantine/core';
import { IconSearch, IconArrowRight, IconArrowLeft } from '@tabler/icons';

function LinkWallet() {
     const { id } = useParams();
  // @ts-ignore
 const theme = useMantineTheme();

  let body = {player: 'test1', wallet: "fuel1cvwcrwsl09krl0dfqtttcemht483yux3t0mmh8xmpaz26edn6rrszg706m"}


  
  let walletLink = () => {
    const url = `https://api.fuelscape.gg/links/`;
    const fetchData = async () => {
      try {
        const response = await fetch(url, {
          headers: {
            "Content-Type": "application/json",
            Accept: "application/json",
          },
          method: "POST",
          body: JSON.stringify(body)
        });
        const json = await response.json();
        console.log("json:", json);
      } catch (error) {
        console.log("error", error);
      }
    };
    fetchData();
  };
  console.log("LinkWallet: ", LinkWallet);

  return (
    <div className="Inventory-Wrapper">
    
 <TextInput
      icon={<IconSearch size={18} stroke={1.5} />}
      radius="xl"
      size="md"
      rightSection={
        <ActionIcon size={32} radius="xl" color={theme.primaryColor} variant="filled">
          {theme.dir === 'ltr' ? (
            <IconArrowRight size={18} stroke={1.5} />
          ) : (
            <IconArrowLeft size={18} stroke={1.5} />
          )}
        </ActionIcon>
      }
      placeholder="fuel1c......706m"
      rightSectionWidth={42}
   onClick={()=>walletLink()}
    />
    </div>
  );
}

export default LinkWallet;
