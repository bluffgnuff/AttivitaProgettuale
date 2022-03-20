#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use rmps::Serializer;
use serde::{Serialize};
use std::collections::HashMap;
use std::env;
use std::io::Read;
use log::debug;

//Usage env parameters --OPERATION {CRUD operation} --TABLE {TABLE} --FIRSTNAME {FIRSTNAME} --LASTNAME {LASTNAME}  --FIRSTNAME-OP {FIRSTNAME-OP} --LASTNAME-OP {LASTNAME-OP}

fn main() {
    env_logger::init();
    //FIXME argomento da passare come un unica stringa
    let req = env::args().nth(1).unwrap_or("Select * from Customer".to_string());

    let mut stdin = std::io::stdin();
    let mut result = String::new();

    debug!("Query to send: {}", req);

    let mut req_pack = Vec::new();
    req.serialize(&mut Serializer::new(&mut req_pack)).unwrap();

    println!("{:?}",req_pack);
    debug!("Request serialized sent {:?}", req_pack);

    stdin.read_to_string(&mut result);

    debug!("Data received: {:?}",result );

    let req_serialized:Vec<u8> = result.split(", ").map(|x| x.parse().unwrap()).collect();
    debug!("Serialized answer {:?}", req_serialized);

//  Deserialize
    if req.to_uppercase().contains("SELECT"){
        let req :Vec<String> = rmp_serde::from_read_ref(&req_serialized).unwrap();
        debug!("Deserialized answer: {:?}",req);
        for el in req {
            debug!("{:?}", el);
        }
    }
    else{
        let req : String = rmp_serde::from_read_ref(&req_serialized).unwrap();
        debug!("Deserialized answer {:?}", req);
    }
}