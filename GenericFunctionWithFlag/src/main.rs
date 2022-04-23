#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use rmps::Serializer;
use serde::{Serialize};
use std::collections::HashMap;
use std::io::{BufRead};
use log::{debug};
use clap::Parser;

//Usage env parameters --operation {CRUD operation} --id {ID} --rev {REV} --table {TABLE} --firstname {FIRSTNAME} --lastname {LASTNAME}  --firstname-op {FIRSTNAME-OP} --lastname-op {LASTNAME-OP}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // Operation name
    #[clap(short, long, default_value = "Read")]
    operation: String,

    // Table name
    #[clap(short, long, default_value = "Customers" )]
    table: String,

    // id of the entry to read/update/delete
    #[clap(long, default_value = "" )]
    id:String,

    // _ref of the entry to update usefull in CouchDB
    #[clap(long, default_value = "")]
    rev:String,

    // Firstname
    #[clap(long, default_value = "" )]
    firstname: String,

    // Lastname
    #[clap(long, default_value = "" )]
    lastname: String,

    // Firstname to Update
    #[clap(long, default_value = "" )]
    firstname_opt: String,

    // Lastname to Update
    #[clap(long, default_value = "" )]
    lastname_opt: String,

    #[clap(long, default_value = "" )]
    db_type: String,
}

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
    let args = Args::parse();
    let stdin = std::io::stdin();
    let result;
    let mut customer: HashMap<String, String> = HashMap::new();
    let mut customer_new: HashMap<String, String> = HashMap::new();

    // added an id column (not auto increment) in the DBs so the client can add it manually
    if args.id != "".to_string() {
            customer.insert("id".to_string(), args.id);
    }

    if args.rev != "".to_string() {
        customer.insert("_rev".to_string(),args.rev);
    }

    if args.firstname != "".to_string() {
        customer.insert("FIRSTNAME".to_string(), args.firstname);
    }

    if args.lastname != "".to_string() {
        customer.insert("LASTNAME".to_string(), args.lastname);
    }

    if args.lastname_opt != "".to_string() {
        customer_new.insert("FIRSTNAME".to_string(), args.firstname_opt);
    }
    if args.lastname_opt != "".to_string() {
        customer_new.insert("LASTNAME".to_string(), args.lastname_opt);
    }

    debug!("Operation selected: {}", args.operation);
    let req =
        match args.operation.as_str() {
            "Create" =>
                Request {
                    op: Op::Create,
                    table: args.table,
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
            // Update
            "Update" =>
                Request {
                    op: Op::Update,
                    table: args.table,
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
            // Delete
            "Delete" =>
                Request {
                    op: Op::Delete,
                    table: args.table,
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
            // Read
            "Read" | _ =>
                Request {
                    op: Op::Read,
                    table: args.table,
                    param: customer,
                    param_to_up: Option::from(customer_new),
                },
        };
    debug!("Request: {:?}", req);

    let mut req_pack = Vec::new();
    req.serialize(&mut Serializer::new(&mut req_pack)).unwrap();

    //  Send req through stdout
    println!("{:?}",req_pack);
    debug!("Request serialized sent {:?}", req_pack);

    //  Receive the answer through stdin
    result = stdin.lock().lines().next().unwrap().unwrap();
    debug!("Data received: {:?}",result );

    //  Deserialize
    if args.operation == "Read"{
        let req_serialized:Vec<u8> = result.split(", ").map(|x| x.parse().unwrap()).collect();
        debug!("Serialized answer {:?}", req_serialized);
        let req :Vec<String> = rmp_serde::from_read_ref(&req_serialized).unwrap();

        if args.db_type != "CouchDB" {
            let mut des_answ= String::new();
            for el in req {
                des_answ = format!("{} {:?}", des_answ, el);
            }
            println!("{}", des_answ);
        }
        else { // case CouchDB
            println!("{:?}", req);
        }
    }
    else{
        println!("{:?}", result);
    }
}