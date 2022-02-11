use std::{env, io, time::SystemTime};
use std::io::Write;
use serde;
#[macro_use]
use serde::{Deserialize, Serialize};
use rmps::{Deserializer, Serializer};

//Usage env parameters --OPERATION {CRUD operation} --URL? {url_address}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Customer {
    firstname: String,
    lastname: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
enum Op {
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Request {
    op: Op,
    customer: Option<Customer>,
    optional: Option<Customer>,
}

fn main() {
    env_logger::init();
    let operation = env::var("OPERATION").unwrap();
    let firstname = env::var("FIRSTNAME").unwrap_or("Mario".to_string());
    let lastname = env::var("LASTNAME").unwrap_or("Rossi".to_string());
    let firstname_opt = env::var("LASTNAME").unwrap_or("Luca".to_string());
    let lastname_opt = env::var("LASTNAME").unwrap_or("Rossi".to_string());

    let stdin = std::io::stdin();
    let mut buffer = String::new();

    debug!("Operation selected :{:?}", operation);
    let mut req = Request;
        match operation.as_str() {
            "Create" => {
                req = Request {
                    op: Op::Create,
                    customer: Option::from(customer),
                    optional: None,
                };
            }
            // Read
            "Read" => {
                customer = Customer {
                    firstname: String::from("Mario"),
                    lastname: String::from("Rossi"),
                };
                req = Request {
                    op: Op::Read,
                    customer: Option::from(customer),
                    optional: None,
                };
            }
            // Update
            "Update" => {
                customer = Customer {
                    firstname: String::from("Mario"),
                    lastname: String::from("Rossi"),
                };

                let customer2 = Customer {
                    firstname: String::from("Luca"),
                    lastname: String::from("Rossi"),
                };

                req = Request {
                    op: Op::Update,
                    customer: Option::from(customer),
                    optional: Option::from(customer2),
                };
            }
            // Delete
            "Delete" => {
                customer = Customer {
                    firstname: String::from("Luca"),
                    lastname: String::from("Rossi"),
                };

                req = Request {
                    op: Op::Delete,
                    customer: Option::from(customer),
                    optional: None,
                };
            }
            _ => "err",
        }
    debug!("Request :{:?}", req);
    // let mut req_pack :Vec<u8>= Vec::new();
    let mut req_pack;
    req.serialize(&mut Serializer::new(&mut req_pack)).unwrap();
    io::stdout().write_all(&mut req_pack);
    debug!("Request sent");

    stdin.read_line(&mut buffer);
    rmp_serde::from_read_ref(&buffer).unwrap();
    debug!("Data received :{:?}",buffer );
    ok();
}