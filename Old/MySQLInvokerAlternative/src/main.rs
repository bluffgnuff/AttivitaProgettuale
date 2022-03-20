#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use rmps::Serializer;
use serde::Serialize;
use std::{env, str};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::{ SystemTime};
use log::{debug, info};
use mysql::prelude::*;
use mysql::*;
use nats::Subscription;
//  Usage env parameters --URL {URL} --DB-NAME {DB-NAME} --COMMAND {COMMAND}

fn work(conn: &mut mysql::PooledConn, command: String) {
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
    drop(child_out);

    let req_serialized:Vec<u8> = out.split(", ").map(|x| x.parse().unwrap()).collect();
    debug!("Invoker | serialized request {:?}", req_serialized);

    //  Deserialize
    let query : String = rmp_serde::from_read_ref(&req_serialized).unwrap();
    debug!("Invoker | deserialized request {:?}", query);

    //  Query execution
    let mut result_serialized  = Vec::new();
    if query.to_uppercase().contains("SELECT"){
            // Send back a Vec<Row> to keep the invoker independent from the data type
            let query_result :Vec<Row> = conn.query(query).unwrap();
            let mut query_string :Vec<String>= Vec::new();
            for el in query_result{
                query_string.push(format!("{:?}", el));
            }
            debug!("Invoker | res: {:?}", query_string);
            query_string.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Invoker | result serialized: {:?}", result_serialized);
        }
    else {
            let query_result =
                match conn.query_drop(query){
                    Ok(_) => String::from("Success"),
                    Err(_) =>String::from("Error")
                };
            query_result.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Invoker | result serialized: {:?}", result_serialized);
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
}

// fn server (mut conn : PooledConn, sub_command: Subscription, sub_args: Subscription){
fn server (mut conn : PooledConn, sub_command: Subscription){
    let mut n_reqs = 0;
    let mut total_duration = 0;
    let mut max = 0;
    let mut min = 0;
    loop {
        // Consuming message
        // TODO waiting for a "close" message to break the loop ?
        let command =  String::from_utf8_lossy(&sub_command.next().unwrap().data).to_string();
        debug!("Invoker | new req received command: {}",command);

        // Stats on time to serve a request
        let start_time = SystemTime::now();
        n_reqs = n_reqs +1;
        work(&mut conn, command);
        let duration = SystemTime::now().duration_since(start_time).unwrap();
        total_duration = total_duration + duration.as_millis();

        if duration.as_millis() > max{
            max = duration.as_millis();
        }

        if duration.as_millis() < min || min == 0{
            min = duration.as_millis();
        }

        info!("Invoker | served the request number {}, in {} ms", n_reqs, duration.as_millis());
        info!("Invoker | average latency {} ms", total_duration/n_reqs);
        info!("Invoker | min latency {} ms", min);
        info!("Invoker | max latency {} ms", max);
    }
}

fn main() {
    env_logger::init();
    let address = env::var("ADDRESS").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or("3306".to_string());
    let username = env::var("NAME").unwrap_or("root".to_string());
    let password = env::var("PASSWORD").unwrap_or("root".to_string());
    let db = env::var("DB-NAME").unwrap_or("testDB".to_string());
    let nats_server = env::var("NATSSERVER").unwrap_or("127.0.0.1".to_string());
    let trigger_command = env::var("TRIGGER").unwrap_or("trigger-command".to_string());
    // let trigger_args = env::var("TRIGGER-ARGS").unwrap_or("trigger-args".to_string());
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
