#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use rmps::Serializer;
use serde::Serialize;
use std::{env, str};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::ptr::null;
use std::time::{ SystemTime};
use log::{debug, info};
use nats::Subscription;
use reqwest::Client;

//Usage env parameters --URL {URL} --DB-NAME {DB-NAME} --COMMAND {COMMAND}

#[tokio::main]
async fn work(command: String, url_base_db: &String, username: &String, password: &String) -> String {
    // Invoking the command
    // In this case, the invoker doesn't need to keep connection to DB so the document can be sent by the function
    let mut child = Command::new("/bin/bash").arg("-c").arg(&command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();
    debug!("Invoker | child launched PID = {}", child.id());

    //Receive the result from the Function
    let mut child_out = BufReader::new(child.stdout.unwrap()).lines();

    // Read function Stdout. It could present a new line char so it's necessary to read all the output until the end
    let mut res= String::new();
    let mut first = true;
    loop{
        match child_out.next(){
            Some(val ) => {
                if first {
                    res = format!("{}", val.unwrap());
                    first = false;
                }else {
                    res = format!("{}, {}",res, val.unwrap());
                }
            },
            None => {
              break;
            },
        }
    }
    debug!("Invoker | child output = {}", res);
    //return the child's output
    res
}

fn server (url_base_db: String, username: String, password: String, sub_command: Subscription){
    let mut n_reqs = 0;
    let mut total_duration = 0;
    let mut max = 0;
    let mut min = 0;

    loop {
        // Stats on time to serve a request
        let start_time = SystemTime::now();
        // Consuming message
        let mex = sub_command.next().unwrap();
        let command =  String::from_utf8_lossy(&mex.data).to_string();
        debug!("Invoker | new req received command: {}",command);

        n_reqs = n_reqs +1;
        // Take child output
        let child_out = work(command, &url_base_db, &username, &password);
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
    let port = env::var("PORT").unwrap_or("5984".to_string());
    let username = env::var("NAME").unwrap_or("root".to_string());
    let password = env::var("PASSWORD").unwrap_or("root".to_string());
    let db = env::var("DB-NAME").unwrap_or("testdb".to_string());
    let nats_server = env::var("NATSSERVER").unwrap_or("127.0.0.1".to_string());
    let trigger_command = env::var("TRIGGER").unwrap_or("trigger-command".to_string());
    // let trigger_args = env::var("TRIGGER-ARGS").unwrap_or("trigger-args".to_string());
    let group = env::var("GROUP").unwrap_or("default".to_string());
    let url_db = format!("http://{}:{}/{}", address, port, db);

    debug!("Invoker | starts");
    debug!("Invoker | URL = {}", url_db);

    // Connection to MOM
    let nc = nats::connect(nats_server.as_str()).unwrap();
    debug!("Invoker | Connected to NATS {:?} ", nc);
    let sub_command = nc.queue_subscribe(&trigger_command, &group).unwrap();
    debug!("Invoker | Sub to command topic {:?}", sub_command);

    server(url_db, username, password, sub_command);
}