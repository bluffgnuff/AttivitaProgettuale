#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use rmps::Serializer;
use serde::Serialize;
//use rmps::{Deserializer, from_read_ref, Serializer};
//use serde::{Deserialize, Serialize};
use std::env;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use log::debug;
use mysql::prelude::*;
use mysql::*;

//Usage env parameters --URL {URL} --DB-NAME {DB-NAME} --COMMAND {COMMAND} --COMMAND-PARAMETERS {COMMAND-PARAMETERS}

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

fn from_request_to_query(request: Request) -> String {
    debug!("Invoker | type of operation requested: {:?}", request.op);
    match request.op {
        Op::Create => {
            let mut col:String = String::new();
            let mut val :String = String::new();
            let mut first = true;

//			Split name, val
            for p in request.param {
                if first{
                    col = format!("{}", p.0);
                    val = format!("'{}'", p.1);
                    first = false;
                }else{
                    col = format!("{},{}", col, p.0);
                    val = format!("{},'{}'", val, p.1);
                }
            }
            format!("INSERT INTO {} ({}) VALUES ({});", request.table, col, val)
        },
        Op::Read => {
            let mut to_find:String = String::new();
            let mut first = true;
            for p in request.param {
                if first {
                    to_find = format!("{}='{}' ", p.0, p.1);
                    first = false;
                }else{
                    to_find = format!("{}AND {}='{}' ", to_find, p.0, p.1);
                }
            }
            format!("SELECT * FROM {} WHERE {};", request.table, to_find )
        },
        Op::Update => {
            let mut old_entry:String = String::new();
            let mut new_entry:String = String::new();
            let mut first = true;
            let mut first_new = true;

//		Data to modify
            for p in request.param {
                if first {
                    old_entry = format!("{}='{}' ", p.0, p.1);
                    first = false;
                }else {
                    old_entry = format!("{}{}='{}' ", old_entry, p.0, p.1);
                }
            }
//		New Data
            for p in request.param_to_up.unwrap() {
                if first_new {
                    new_entry = format!("{}='{}' ", p.0, p.1);
                    first_new = false;
                }else {
                    new_entry = format!("{}{}='{}' ", new_entry, p.0, p.1);
                }
            }

            format!("UPDATE {} SET {} WHERE {};",request.table, old_entry,  new_entry)
        },
        Op::Delete => {
            let mut to_delete:String = String::new();
            let mut first = true;
            for p in request.param {
                if first {
                    to_delete = format!("{}='{}' ", p.0, p.1);
                    first = false;
                }
                else {
                    to_delete = format!("{}{}='{}' ", to_delete, p.0, p.1);
                }
            }
            format!("DELETE from {} where {};", request.table, to_delete)
        },
    }
}

fn work(mut conn: PooledConn, mut child: Child) {
    let child_in = child.stdin.as_mut().unwrap();
    let mut child_out = BufReader::new(child.stdout.unwrap()).lines();

// Receive
    let mut out = child_out.next().unwrap().unwrap();
    out.remove(0);
    out.remove(out.len()-1);
    debug!("Invoker | request cleaned {:?}", out);
    drop(child_out);
    let req_serialized:Vec<u8> = out.split(", ").map(|x| x.parse().unwrap()).collect();
    debug!("Invoker | serialized request {:?}", req_serialized);

//  Deserialize
    let req: Request = rmp_serde::from_read_ref(&req_serialized).unwrap();
    debug!("Invoker | deserialized request {:?}", req);

//  Query generation
    let query = from_request_to_query(req.clone());
    debug!("Invoker | query to execute: {}", query);

    let mut result_serialized  = Vec::new();

//  Query execution
    match req.op{
        Op::Read =>{
// FIXME: dependence on the type of data to be returned
            let query_result :Vec<(Option<i32>,String,String)> = conn.query(query).unwrap();
            debug!("Invoker | res: {:?}", query_result);
            query_result.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Invoker | result serialized: {:?}", result_serialized);
        },
        Op::Create | Op::Update | Op::Delete => {
            let query_result =
             match conn.query_drop(query){
                 Ok(_) => String::from("Success"),
                 Err(_) =>String::from("Error")
            };
            query_result.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Invoker | result serialized: {:?}", result_serialized);
        },
    }
//	Send back an answer
    let mut res_string_result_serialized = String::new();
    let mut first = true;

// FIXME: dependence on the type of data to be returned
    for el in result_serialized {
        if first {
            res_string_result_serialized= format!("{}", el);
            first = false;
        }
        else {
            res_string_result_serialized = format!("{}, {}", res_string_result_serialized, el);
        }
    }
    debug!("Invoker | sent the result {}", res_string_result_serialized);
    child_in.write_all(res_string_result_serialized.as_str().as_bytes());
}

fn main() {
    env_logger::init();
    let url = env::var("URL").unwrap_or("mysql://root:root@127.0.0.1:3306".to_string());
    let db = env::var("DB-NAME").unwrap_or("testDB".to_string());
    let command = env::var("COMMAND").unwrap_or("../GenericFunction/target/debug/GenericFunction".to_string());
    let url_db = format!("{}/{}",url,db);

    // Parameters for the command
    // let operation = env::var("OPERATION").unwrap_or("Read".to_string());
    // let firstname = env::var("TABLE").unwrap_or("Customers".to_string());
    // let firstname = env::var("FIRSTNAME").unwrap_or("Mario".to_string());
    // let lastname = env::var("LASTNAME").unwrap_or("Rossi".to_string());
    // let firstname_opt = env::var("FIRSTNAME-OP").unwrap_or("Luca".to_string());
    // let lastname_opt = env::var("LASTNAME-OP").unwrap_or("Rossi".to_string());

    debug!("Invoker | starts");
    debug!("Invoker | URL = {}", url);
    debug!("Invoker | DB = {}", db);
    debug!("Invoker | COMMAND = {}", command);

    let opts = Opts::from_url(url_db.as_str());
    let pool = Pool::new(opts.unwrap()).unwrap();
    let conn = pool.get_conn().unwrap();

    debug!("Invoker | starts the connection on URL= {}", url_db);

    let child = Command::new(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    debug!("Invoker | child launched PID = {}", child.id());
   
    work(conn, child);
}