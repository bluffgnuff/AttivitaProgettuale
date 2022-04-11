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
use nats::Subscription;
use reqwest::Client;

//Usage env parameters --URL {URL} --DB-NAME {DB-NAME} --COMMAND {COMMAND}

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
// Convert a Request to a CouchDB query
fn from_request_to_json(request: Request) -> String {
    debug!("Invoker | type of operation requested: {:?}", request.op);
    match request.op {
        Op::Create => {
            format!("{:?}" ,request.param)
        },
        Op::Read => {
            let mut to_find:String = String::new();
            let mut first = true;
            let start_select = "{\"selector\": {";
            let close = "}";
            let eq =  ": {\"$eq\":";
            let mut vec:Vec<String>;

            to_find = format!("{} {}", to_find, start_select);
            if request.clone().param.len() > 1{
                to_find = format!("{}", to_find);
            }
            for p in request.param.clone() {
                if first {
                    to_find = format!("{} \"{}\" {} \"{}\"{}", to_find, p.0, eq, p.1, close);
                    first = false;
                }else{
                    to_find = format!("{}, \"{}\" {} \"{}\"{}", to_find, p.0, eq, p.1, close);
                }
            }
            if request.param.len() > 1{
                to_find = format!("{}", to_find);
            }
            format!("{}{}{}", to_find, close, close)
        },
        Op::Update => {
            let start = "{";
            let close = "}";
            let old_rev = request.param.get("_rev").unwrap();
            let mut res = format!("{} \"_rev\": \"{}\"", start, old_rev);
            for (key, val) in request.param{
                if key!= "_rev".to_string() && key!= "_id".to_string() {
                    res = format!("{}, \"{}\": \"{}\"", res, key, val);
                }
            }
            format!("{} {}", res, close)
        },
        Op::Delete  => format!("")
    }
}

#[tokio::main]
async fn work(client : &mut Client, command: String, url_base_db: &String, username: &String, password: &String) {
    // Invoking the command
    let mut child = Command::new("/bin/bash").arg("-c").arg(&command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    debug!("Invoker | child launched PID = {}", child.id());

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

    //  Query execution
    let mut result_serialized  = Vec::new();
    match req.op{
        Op::Create => {
            //  Query generation
            let data = from_request_to_json(req.clone());
            debug!("Invoker | query to execute: {}", data);

            let query_result = client.post(url_base_db).basic_auth(username, Some(password)).body(data).header("Content-Type", "application/json").send().await.unwrap().text().await.unwrap();
            debug!("Invoker | result: {:?}", query_result);
            query_result.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Invoker | result serialized: {:?}", result_serialized);
        },
        Op::Read =>{
            //  Query generation
            let data = from_request_to_json(req.clone());
            debug!("Invoker | query to execute: {}", data);

            let url= format!("{}/_find",url_base_db);
            let query_result = client.post(url).basic_auth(username, Some(password)).body(data).header("Content-Type", "application/json").send().await.unwrap().text().await.unwrap();
            debug!("Invoker | result: {:?}", query_result);

            query_result.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Invoker | result serialized: {:?}", result_serialized);
        },
        Op::Update => {
            //  Query generation
            let data = from_request_to_json(req.clone());
            let url= format!("{}/{}",url_base_db, req.param.get("_id").unwrap());
            debug!("Invoker | query to execute: {}", data);

            let query_result = client.put(url).basic_auth(username, Some(password)).body(data).header("Content-Type", "application/json").send().await.unwrap().text().await.unwrap();
            debug!("Invoker | result: {:?}", query_result);

            query_result.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Invoker | result serialized: {:?}", result_serialized);
        },
        Op::Delete => {
            //  Query generation
            let data = req.param.get("id").unwrap();
            let url= format!("{}/{}",url_base_db, data);
            debug!("Invoker | query to execute: delete {}", data);

            let query_result = client.delete(url).basic_auth(username, Some(password)).header("Content-Type", "application/json").send().await.unwrap().text().await.unwrap();
            debug!("Invoker | result: {:?}", query_result);

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

    debug!("Invoker | send the result {}", res_string_result_serialized);
    child_in.write_all(res_string_result_serialized.as_str().as_bytes());
}

fn server (mut client :Client, url_base_db: String, username: String, password: String, sub_command: Subscription){
    let mut n_reqs = 0;
    let mut total_duration = 0;
    let mut max = 0;
    let mut min = 0;

    loop {
        // Consuming message
        let command =  String::from_utf8_lossy(&sub_command.next().unwrap().data).to_string();
        debug!("Invoker | new req received command: {}",command);

        n_reqs = n_reqs +1;
        // Stats on time to serve a request
        let start_time = SystemTime::now();
        work(&mut client, command, &url_base_db, &username, &password);
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

    // Client HTTP
    let client = reqwest::Client::builder().build().unwrap();
    debug!("Invoker | Client created to DB: {:?}", client);

    // Connection to MOM
    let nc = nats::connect(nats_server.as_str()).unwrap();
    debug!("Invoker | Connected to NATS {:?} ", nc);
    let sub_command = nc.queue_subscribe(&trigger_command, &group).unwrap();
    debug!("Invoker | Sub to command topic {:?}", sub_command);
    // let sub_args = nc.queue_subscribe(&trigger_args, &group).unwrap();
    // debug!("Invoker | Sub to args topic {:?} ", sub_args);

    // server(client, sub_command, sub_args );
    server(client, url_db, username, password, sub_command);
}