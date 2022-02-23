#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use mysql::prelude::*;
use mysql::*;
use rmps::{Deserializer, from_read_ref, Serializer};
use serde::{Deserialize, Serialize};
use std::{env, thread};
use std::io::{BufRead, BufReader, Lines};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::time::Duration;

// #[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
// struct Person {
//     firstname: String,
//     lastname: String,
// }

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

fn server(url: String, rx: &mut ChildStdin, tx1: Lines<BufReader<ChildStdout>>) {
    let pool = Pool::new(opts.unwrap()).unwrap();
    let mut conn = pool.get_conn().unwrap();
    debug!("Invoker starts the connection on URL", url);

    loop {
        // Receive
        // FIXME update con stdin
        let mut buff = rx.recv().unwrap();
        //  Deserialize
        let req: Request = rmp_serde::from_read_ref(&buff).unwrap();
        //  Query
        let query = from_request_to_query(req.clone());

        match  req.op{
            Op::Read =>{
                let res:Vec<(String,String)> = conn.query(query).unwrap();
                //TODO modificare usando una map<String,String9/>?
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
                // FIXME update con stdout
                tx1.send(answ);
            },
            Op::Delete => {
                let res:Result<Vec<String>> = conn.query(query);
                break;
            }
            _ => {
                break
            }
        }
    }
}

fn main() {
    env_logger::init();
    let url = env::var("URL").unwrap();//  "mysql://root:root@127.0.0.1:3306/";
    debug!("Invoker | starts");
    debug!("Invoker | URL = {:?}", );
    let mut child = Command::new(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    debug!("Invoker | child launch");
    //FIXME remove the channel dependencies
    //TODO add comunication stdin/out
    //TODO remove the person/customer dependency
    let child_stdin = child.stdin.as_mut().unwrap();
    //let child_stdout = child.stdout.as_mut().unwrap();
    let mut child_stdout = BufReader::new(child.stdout.unwrap()).lines();
    // drop(child_stdin);
//TODO valuta le opzioni
//1
    //     .stdout(Stdio::piped())
    //     .spawn()?;
    // let output = child.wait_with_output()?;
// 2
    // let output = Command::new("/bin/bash")
    //     .arg("-c")
    //     .arg(&cm)
    //     .output()
    //     .expect("failed to execute process");
    // let stdout = String::from_utf8_lossy(&output.stdout).to_string();


    server(url, child_stdin, child_stdout);
}