#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use std::borrow::Borrow;
use std::ptr::null;
use mysql::prelude::*;
use mysql::*;
use rmps::{Deserializer, from_read_ref, Serializer};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
struct Person {
    firstname: String,
    lastname: String,
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
    customer: Person,
    optional: Option<Person>,
}

fn from_request_to_query(request: Request) -> String {
    match request.op {
        Op::Create => {
            let customer = request.customer;
            format!("INSERT INTO {}(FirstName, LastName) VALUES ( '{}', '{}');", request.table, customer.firstname, customer.lastname )
        },
        Op::Read => {
            let customer = request.customer;
            format!("SELECT FirstName, LastName  FROM {} WHERE LastName='{}';", request.table, customer.lastname )
        },
        Op::Update => {
            let customer = request.customer;
            let customer_new = request.optional.unwrap();
            format!("UPDATE {} SET FirstName='{}' WHERE FirstName='{}';",request.table, customer_new.firstname, customer.firstname)
        },
        Op::Delete => {
            let customer = request.customer;
            format!("DELETE from {} where FirstName='{}' and LastName='{}';", request.table, customer.firstname, customer.lastname)
        },
    }
}

fn server(url: &str, rx: Receiver<Vec<u8>>, tx1: Sender<Vec<u8>>) {
    let opts = Opts::from_url(url);
    let pool = Pool::new(opts.unwrap()).unwrap();
    let mut conn = pool.get_conn().unwrap();

    loop {
        // Receive
        let mut buff = rx.recv().unwrap();
        //  Deserialize
        let req: Request = rmp_serde::from_read_ref(&buff).unwrap();
        //  Query
        let query = from_request_to_query(req.clone());

        match  req.op{
            Op::Read =>{
                let res:Vec<(String,String)> = conn.query(query).unwrap();
                let mut persons :Vec<Person> = Vec::new();
                if !res.is_empty() {
                    for el in res {
                        let p = Person{
                            firstname:el.0,
                            lastname: el.1,
                        };
                        persons.push(p);
                    }
                }
                let mut answ = Vec::new();
                persons.serialize(&mut Serializer::new(&mut answ)).unwrap();
                tx1.send(answ);
            },
            Op::Delete => {
                let res:Result<Vec<String>> = conn.query(query);
                break;
            },
            _ => {
                let res:Result<Vec<String>> = conn.query(query);
            }
        }
    }
}

fn main() {
    let url = "mysql://root:root@127.0.0.1:3306/testDB";
    let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();
    let (tx1, rx1): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));
        // Create
        let mut customer = Person {
            firstname: String::from("Mario"),
            lastname: String::from("Rossi"),
        };

        let mut req = Request {
            op: Op::Create,
            table: String::from("Customers"),
            customer,
            optional: None,
        };

        let mut req_pack = Vec::new();
        req.serialize(&mut Serializer::new(&mut req_pack)).unwrap();
        println!("[C] asks for a {:?}", req.op);

        tx.send(req_pack);
        thread::sleep(Duration::from_millis(100));

        // Read by lastname
        customer = Person {
            firstname: String::from("Mario"),
            lastname: String::from("Rossi"),
        };

        req = Request {
            op: Op::Read,
            table: String::from("Customers"),
            customer,
            optional: None,
        };
        println!("[C] asks for a {:?}", req.op);

        req_pack = Vec::new();
        req.serialize(&mut Serializer::new(&mut req_pack)).unwrap();

        tx.send(req_pack);
        let buff = rx1.recv().unwrap();
        let persons :Vec<Person>= rmp_serde::from_read_ref(&buff).unwrap();
        println!("[C] received {:?}",persons);
        thread::sleep(Duration::from_millis(100));

        // Update
        customer = Person {
            firstname: String::from("Mario"),
            lastname: String::from("Rossi"),
        };

        let customer2 = Person {
            firstname: String::from("Luca"),
            lastname: String::from("Rossi"),
        };

        req = Request {
            op: Op::Update,
            table: String::from("Customers"),
            customer,
            optional: Option::from(customer2),
        };
        req_pack = Vec::new();
        req.serialize(&mut Serializer::new(&mut req_pack)).unwrap();
        println!("[C] asks for an {:?}", req.op);

        tx.send(req_pack);
        thread::sleep(Duration::from_millis(100));

        // Delete
        customer = Person {
            firstname: String::from("Luca"),
            lastname: String::from("Rossi"),
        };

        req = Request {
            op: Op::Delete,
            table: String::from("Customers"),
            customer,
            optional: None,
        };
        req_pack = Vec::new();
        req.serialize(&mut Serializer::new(&mut req_pack)).unwrap();
        println!("[C] asks for a {:?}", req.op);
        tx.send(req_pack);
    });
    server(url, rx, tx1);
}