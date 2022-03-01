#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use rmps::{Deserializer, from_read_ref, Serializer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{env, io, time::SystemTime};
use std::io::{BufRead, Read, Write};
use std::ptr::null;
use log::{debug, error};

//Usage env parameters --OPERATION {CRUD operation} --FIRSTNAME {FIRSTNAME} --LASTNAME {FIRSTNAME} --TABLE {TABLE}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
enum Op {
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
struct Request {
    op: Op,
    table: String,
    param: HashMap<String, String>,
    param_to_up: Option<HashMap<String, String>>,
}

fn main() {
    env_logger::init();
    let operation = env::var("OPERATION").unwrap_or("Read".to_string());
    let firstname = env::var("TABLE").unwrap_or("Customers".to_string());
    let firstname = env::var("FIRSTNAME").unwrap_or("Mario".to_string());
    let lastname = env::var("LASTNAME").unwrap_or("Rossi".to_string());
    let firstname_opt = env::var("FIRSTNAME-OP").unwrap_or("Luca".to_string());
    let lastname_opt = env::var("LASTNAME-OP").unwrap_or("Rossi".to_string());

    let mut stdin = std::io::stdin();
    let mut result = String::new();
    let mut customer: HashMap<String, String> = HashMap::new();
    customer.insert("FIRSTNAME".to_string(),firstname);
    customer.insert("LASTNAME".to_string(),lastname);

    let mut customerNew: HashMap<String, String> = HashMap::new();
    customerNew.insert("FIRSTNAME".to_string(),firstname_opt);
    customerNew.insert("LASTNAME".to_string(),lastname_opt);

    debug!("Operation selected :{:?}", operation);
    let mut req =
        match operation.as_str() {
            "Create" =>
                 Request {
                    op: Op::Create,
                    table:String::from("Customers"),
                    param: customer,
                    param_to_up: Option::from(customerNew),
                 },
            // Read
            "Read" =>
                Request {
                    op: Op::Read,
                    table:String::from("Customers"),
                    param: customer,
                    param_to_up: Option::from(customerNew),
                },
            // Update
            "Update" | _ =>
                Request {
                    op: Op::Update,
                    table:String::from("Customers"),
                    param: customer,
                    param_to_up: Option::from(customerNew),
                },
            // Delete
            "Delete" =>
                 Request {
                    op: Op::Delete,
                    table:String::from("Customers"),
                    param: customer,
                    param_to_up: Option::from(customerNew),
                },
            // _ =>
            //     error!("Invoker | ERROR bad operation")
            // ,
        };
    debug!("Request :{:?}", req);

    let mut req_pack = Vec::new();
    req.serialize(&mut Serializer::new(&mut req_pack)).unwrap();

    println!("{:?}",req_pack);
    debug!("Request serialized sent {:?}", req_pack);

//FIXME problema in lettura o scrittura ?
    // stdin.read_line(&mut buffer);
    // rmp_serde::from_read_ref(&buffer).unwrap();
    // let stdin = io::stdin();
    stdin.read_to_string(&mut result);

    // for line in stdin.lock().lines() {
    //     debug!("{}", line.unwrap());
    // }
    // out.remove(0);
    // out.remove(out.len()-1);
    debug!("Data received: {:?}",result );

    let req_serialized:Vec<u8> = result.split(", ").map(|x| x.parse().unwrap()).collect();
    debug!("Serialized answer {:?}", req_serialized);

//  Deserialize
    if operation == "Read"{
        let req :Vec<(Option<i32>,String,String)> = rmp_serde::from_read_ref(&req_serialized).unwrap();
        debug!("Deserialized answer {:?}", req);
    }
    else{
        let req : String = rmp_serde::from_read_ref(&req_serialized).unwrap();
        debug!("Deserialized answer {:?}", req);
    }
}