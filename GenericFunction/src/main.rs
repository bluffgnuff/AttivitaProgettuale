#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;


use rmps::Serializer;
use serde::{Serialize};
// use rmps::{Deserializer, from_read_ref, Serializer};
// use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::io::Read;
use log::debug;

//Usage env parameters --OPERATION {CRUD operation} --TABLE {TABLE} --FIRSTNAME {FIRSTNAME} --LASTNAME {LASTNAME}  --FIRSTNAME-OP {FIRSTNAME-OP} --LASTNAME-OP {LASTNAME-OP}

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

    let mut customer_new: HashMap<String, String> = HashMap::new();
    customer_new.insert("FIRSTNAME".to_string(),firstname_opt);
    customer_new.insert("LASTNAME".to_string(),lastname_opt);

    debug!("Operation selected :{:?}", operation);
    let mut req =
        match operation.as_str() {
            "Create" =>
                 Request {
                    op: Op::Create,
                    table:String::from("Customers"),
                    param: customer,
                    param_to_up: Option::from(customer_new),
                 },
            // Read
            "Read" =>
                Request {
                    op: Op::Read,
                    table:String::from("Customers"),
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
            // Update
            "Update" =>
                Request {
                    op: Op::Update,
                    table:String::from("Customers"),
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
            // Delete
            "Delete" =>
                 Request {
                    op: Op::Delete,
                    table:String::from("Customers"),
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
            // Update
            _ =>
                Request {
                    op: Op::Update,
                    table:String::from("Customers"),
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
        };
    debug!("Request: {:?}", req);

    let mut req_pack = Vec::new();
    req.serialize(&mut Serializer::new(&mut req_pack)).unwrap();

    println!("{:?}",req_pack);
    debug!("Request serialized sent {:?}", req_pack);

    stdin.read_to_string(&mut result);

    debug!("Data received: {:?}",result );

    let req_serialized:Vec<u8> = result.split(", ").map(|x| x.parse().unwrap()).collect();
    debug!("Serialized answer {:?}", req_serialized);

//  Deserialize
    if operation == "Read"{
        // FIXME: dependence on the type of data to be returned
        let req :Vec<(String)> = rmp_serde::from_read_ref(&req_serialized).unwrap();
        debug!("Deserialized answer: ");
        for el in req {
            debug!("{:?}", el);
        }
    }
    else{
        let req : String = rmp_serde::from_read_ref(&req_serialized).unwrap();
        debug!("Deserialized answer {:?}", req);
    }
}