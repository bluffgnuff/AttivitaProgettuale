#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use rmps::Serializer;
use serde::Serialize;
use std::{env, str};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::{ SystemTime};
use log::{debug, info};
use mysql::prelude::*;
use mysql::*;
use nats::{Subscription};
//  Usage env parameters --URL {URL} --DB-NAME {DB-NAME} --COMMAND {COMMAND}

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
    table : String,
    param: HashMap<String, String>,
    param_to_up: Option<HashMap<String, String>>,
}
// Convert a Request to a MySQL query
fn from_request_to_query(request: Request) -> String {
    debug!("Invoker | type of operation requested: {:?}", request.op);
    match request.op {
        Op::Create => {
            let mut col:String = String::new();
            let mut val :String = String::new();
            let mut first = true;

            //  Split name, val
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
                    to_find = format!("{}AND {}='{}'", to_find, p.0, p.1);
                }
            }
            format!("SELECT * FROM {} WHERE {};", request.table, to_find )
        },
        Op::Update => {
            let mut old_entry:String = String::new();
            let mut new_entry:String = String::new();
            let mut first = true;
            let mut first_new = true;

            //	Data to modify
            for p in request.param {
                if first {
                    old_entry = format!("{}='{}'", p.0, p.1);
                    first = false;
                }else {
                    old_entry = format!("{} AND {}='{}'", old_entry, p.0, p.1);
                }
            }
            //  New Data
            for p in request.param_to_up.unwrap() {
                if first_new {
                    new_entry = format!("{}='{}'", p.0, p.1);
                    first_new = false;
                }else {
                    new_entry = format!("{},{}='{}'", new_entry, p.0, p.1);
                }
            }

            format!("UPDATE {} SET {} WHERE {};",request.table, new_entry, old_entry)
        },
        Op::Delete => {
            let mut to_delete:String = String::new();
            let mut first = true;
            for p in request.param {
                if first {
                    to_delete = format!("{}='{}'", p.0, p.1);
                    first = false;
                }
                else {
                    to_delete = format!("{} AND {}='{}'", to_delete, p.0, p.1);
                }
            }
            format!("DELETE from {} where {};", request.table, to_delete)
        },
    }
}

fn work(conn: &mut mysql::PooledConn, command: String, n_reqs:i32) -> String {
    // Invoking the command
    let mut child = Command::new("/bin/bash").arg("-c").arg(&command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    debug!("Invoker | child launched PID = {}", child.id());

    let child_in = child.stdin.as_mut().unwrap();
    let mut child_out = BufReader::new(child.stdout.unwrap()).lines();

    //  Receive
    let mut out = child_out.next().unwrap().unwrap();
    out.remove(0);
    out.remove(out.len()-1);
    debug!("Invoker | request cleaned {:?}", out);

    let req_serialized:Vec<u8> = out.split(", ").map(|x| x.parse().unwrap()).collect();
    debug!("Invoker | serialized request {:?}", req_serialized);

    //  Deserialize
    let req: Request = rmp_serde::from_read_ref(&req_serialized).unwrap();
    debug!("Invoker | deserialized request {:?}", req);

    //  Query generation
    let query = from_request_to_query(req.clone());
    debug!("Invoker | query to execute: {}", query);

    //  Query execution
    let mut result_serialized  = Vec::new();
    match req.op{
        Op::Read =>{
            // Send back a Vec<Row> to keep the invoker independent from the data type
            let start_time = SystemTime::now();
            let query_result :Vec<Row> = conn.query(query).unwrap();
            let db_duration = SystemTime::now().duration_since(start_time).unwrap();
            info!("[DB_LATENCY] request number {}: latency {} μs", n_reqs, db_duration.as_micros());

            let mut query_string :Vec<String>= Vec::new();
            for el in query_result{
                query_string.push(format!("{:?}", el));
            }
            debug!("Invoker | res: {:?}", query_string);
            query_string.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Invoker | result serialized: {:?}", result_serialized);
        },
        Op::Create | Op::Update | Op::Delete => {
            let start_time = SystemTime::now();
            let query_result =
             match conn.query_drop(query){
                 Ok(_) => String::from("Success"),
                 Err(_) =>String::from("Error")
            };
            let duration = SystemTime::now().duration_since(start_time).unwrap();
            info!("[DB_LATENCY] request number {}: latency {} μs", n_reqs, duration.as_micros());

            query_result.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Invoker | result serialized: {:?}", result_serialized);
        },
    }
    //	Send back an answer
    let mut res_string_result_serialized = String::new();
    let mut first = true;

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
    child_in.write("\n".as_bytes());

    //return the child's output
    let res = child_out.next().unwrap().unwrap();
    debug!("Invoker | child output: {}", res);
    return  res;
}

fn server (mut conn : PooledConn, sub_command: Subscription){
    let mut n_reqs = 0;
    let mut total_duration = 0;
    let mut max = 0;
    let mut min = 0;
    loop {
        // Consuming message
        let mex = sub_command.next().unwrap();
        let command =  String::from_utf8_lossy(&mex.data).to_string();
        debug!("Invoker | new req received command: {}",command);

        n_reqs = n_reqs +1;

        // Stats on time to serve a request
        let start_time = SystemTime::now();
        let child_out = work(&mut conn, command, n_reqs.clone());
        let duration = SystemTime::now().duration_since(start_time).unwrap();

        total_duration = total_duration + duration.as_micros();

        if duration.as_micros() > max{
            max = duration.as_micros();
        }

        if duration.as_micros() < min || min == 0{
            min = duration.as_micros();
        }
        let average =  total_duration/(n_reqs as u128);
        info!("[WORK_LATENCY] request number {}: latency {} μs", n_reqs, duration.as_micros());
        info!("[WORK_AVERAGE_LATENCY] request number {}: average latency {} μs", n_reqs, average);
        info!("[WORK_MIN_LATENCY] request number {}: {} μs", n_reqs, min);
        info!("[WORK_MAX_LATENCY] request number {}: max latency {} μs", n_reqs, max);
        // answer to stresser
        let res_answer = match mex.respond(child_out){
            Ok(_) => String::from("Success"),
            Err(_) =>String::from("Error")
        };
        debug!("Respond: {}", res_answer);
    }
}

fn main() {
    env_logger::init();
    let address = env::var("ADDRESS").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or("3306".to_string());
    let username = env::var("NAME").unwrap_or("root".to_string());
    let password = env::var("PASSWORD").unwrap_or("root".to_string());
    let db = env::var("DB_NAME").unwrap_or("testDB".to_string());
    let nats_server = env::var("NATSSERVER").unwrap_or("127.0.0.1".to_string());
    let trigger_command = env::var("TRIGGER").unwrap_or("trigger-command".to_string());
    let group = env::var("GROUP").unwrap_or("default".to_string());
    let url_db = format!("mysql://{}:{}@{}:{}/{}", username, password, address, port, db);

    debug!("Invoker | starts");
    debug!("Invoker | URL = {}", url_db);

    // Connection to DB
    let opts = Opts::from_url(url_db.as_str());
    let pool = Pool::new(opts.unwrap()).unwrap();
    let conn = pool.get_conn().unwrap();
    debug!("Invoker | Connected to DB: {:?}", conn);

    // Connection to MOM
    let nc = nats::connect(nats_server.as_str()).unwrap();
    debug!("Invoker | Connected to NATS {:?} ", nc);
    let sub_command = nc.queue_subscribe(&trigger_command, &group).unwrap();
    debug!("Invoker | Sub to command topic {:?}", sub_command);

    server(conn, sub_command);
}