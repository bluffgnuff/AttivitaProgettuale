#[macro_use]
extern crate serde_derive;
extern crate rmpv;
extern crate rmp_serde as rmps;

use rmps::Serializer;
use serde::Serialize;
use std::{env, str};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::iter::Map;
use std::process::{Command, Stdio};
use std::time::{SystemTime};
use log::{debug, info};
use mysql::prelude::*;
use mysql::*;
use nats::{Connection};
use crate::serde_json::Deserializer;

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
    debug!("Type of operation requested: {:?}", request.op);
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

fn work(conn: &mut mysql::PooledConn, command: String) -> String {
    // Invoking the command
    let mut child = Command::new("/bin/bash").arg("-c").arg(&command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    debug!("Child launched PID = {}", child.id());

    let child_in = child.stdin.as_mut().unwrap();
    let mut child_out = BufReader::new(child.stdout.unwrap()).lines();

    //  Receive
    let mut out = child_out.next().unwrap().unwrap();
    debug!("Request {:?}", out);
    out.remove(0);
    out.remove(out.len()-1);
    debug!("Request cleaned {:?}", out);

    let mut req_serialized:Vec<u8> = out.split(", ").map(|x| x.parse().unwrap()).collect();
    debug!("Serialized request {:?}", req_serialized);

    let req: Request = rmp_serde::from_read_ref(&req_serialized).unwrap();
    debug!("Deserialized request {:?}", req);

    //  Query generation
    let query = from_request_to_query(req.clone());
    debug!("Query to execute: {}", query);

    //  Query execution
    let mut result_serialized  = Vec::new();
    let answer = match req.op{
        Op::Read=>{
            // Send back a Vec<Row> to keep the invoker independent from the data type
            let start_time = SystemTime::now();
            let query_result :Vec<Row> = conn.query(query).unwrap();
            let db_latency = SystemTime::now().duration_since(start_time).unwrap();
            info!("[DB_LATENCY] latency {} μs", db_latency.as_micros());

            let mut query_string :Vec<String>= Vec::new();
            for el in query_result{
                query_string.push(format!("{:?}", el));
            }
            debug!("Result: {:?}", query_string);
            query_string.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Result serialized: {:?}", result_serialized);

            // Conversion to string
            let mut string_result_serialized = String::new();
            let mut first = true;

            for el in result_serialized {
                if first {
                    string_result_serialized= format!("{}", el);
                    first = false;
                }
                else {
                    string_result_serialized = format!("{}, {}", string_result_serialized, el);
                }
            }
            string_result_serialized
        },
        Op::Create| Op::Update | Op::Delete => {
            let start_time = SystemTime::now();
            let query_result =
             match conn.query_drop(query){
                 Ok(_) => String::from("Success"),
                 Err(_) =>String::from("Error")
            };
            let db_latency = SystemTime::now().duration_since(start_time).unwrap();
            info!("[DB_LATENCY] latency {} μs", db_latency.as_micros());

            query_result
        },
    };
    //	Send back an answer
    debug!("Sent the result {}", answer);

    child_in.write_all(answer.as_str().as_bytes());
    child_in.write("\n".as_bytes());

    //  Return the child's output
    let res = child_out.next().unwrap().unwrap();
    debug!("Child output: {}", res);
    return  res;
}

fn server (mut conn : PooledConn, nc: Connection, trigger_command: String, trigger_answer: String, group: String){
    let mut n_reqs = 0;
    let mut total_latency = 0;
    let mut max = 0;
    let mut min = 0;

    let sub_command = nc.queue_subscribe(trigger_command.as_str(), group.as_str()).unwrap();
    debug!("Sub to command topic {:?}", sub_command);

    loop {
        // Consuming message
        let mex = sub_command.next().unwrap();
        let received_header = mex.headers.unwrap();
        let req_id = received_header.get("id").unwrap();
        let command =  String::from_utf8_lossy(&mex.data).to_string();
        debug!("New request: {} received command: {}", req_id, command);

        // Launch operation
        n_reqs = n_reqs +1;
        let start_time = SystemTime::now();
        let child_out = work(&mut conn, command);
        let work_latency = SystemTime::now().duration_since(start_time).unwrap();
        debug!("Child ouput: {}",child_out);

        // Answer to stresser
        let mut headers = nats::HeaderMap::new();
        headers.append("id", req_id);
        nc.publish_with_reply_or_headers(&trigger_answer, None, Option::Some(&headers), child_out).unwrap();
        debug!("Answer to {} sent", req_id);

        // Update general stats work
        total_latency = total_latency + work_latency.as_micros();
        if work_latency.as_micros() > max{
            max = work_latency.as_micros();
        }
        if work_latency.as_micros() < min || min == 0{
            min = work_latency.as_micros();
        }
        let average = total_latency/(n_reqs as u128);

        // Print Stats
        info!("[WORK_LATENCY] request number {}: latency {} μs", n_reqs, work_latency.as_micros());
        info!("[WORK_AVERAGE_LATENCY] request number {}: average latency {} μs", n_reqs, average);
        info!("[WORK_MIN_LATENCY] request number {}: {} μs", n_reqs, min);
        info!("[WORK_MAX_LATENCY] request number {}: max latency {} μs", n_reqs, max);
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
    let trigger_answer = env::var("TRIGGER_ANSWER").unwrap_or("trigger-answer".to_string());
    let group = env::var("GROUP").unwrap_or("default".to_string());
    let url_db = format!("mysql://{}:{}@{}:{}/{}", username, password, address, port, db);

    debug!("Starts");
    debug!("URL = {}", url_db);

    // Connection to DB
    let opts = Opts::from_url(url_db.as_str());
    let pool = Pool::new(opts.unwrap()).unwrap();
    let conn = pool.get_conn().unwrap();
    debug!("Connected to DB: {:?}", conn);

    // Connection to MOM
    let nc = nats::connect(nats_server.as_str()).unwrap();
    debug!("Connected to NATS {:?} ", nc);
    debug!("Start publishing to topic:{}", trigger_answer);

    server(conn, nc, trigger_command, trigger_answer, group);
}