#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use rmps::Serializer;
use serde::Serialize;
use std::{env, str};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::{ SystemTime};
use log::{debug, info};
use mongodb::bson::{doc, Document};
use mongodb::bson::oid::ObjectId;
use mongodb::sync::{Client, Database};
use nats::{Connection};
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
    table: String,
    param: HashMap<String, String>,
    param_to_up: Option<HashMap<String, String>>,
}
// Convert the parameter of the the request to a Bson Document;
// Set update=true if the Document is used in an Update operation
fn from_param_to_doc(param : HashMap<String, String>, update: bool) -> Document {
    let mut doc: Document = Default::default();
    //	Split name, val
    for p in param{
         if p.0 == "_id" {
             doc.insert(p.0.as_str(), ObjectId::parse_str(p.1.as_str()).unwrap());
         }
         else {
             doc.insert(p.0.as_str(),p.1.as_str());
        }
    }
    if update{
       doc = doc!( "$set" : doc);
    }
    debug!("Document Created {}", doc);
    doc
}

fn work(conn: &mut Database, command: String) -> String {
    // Invoking the command
    let mut child = Command::new("/bin/bash").arg("-c").arg(&command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    debug!("Child launched PID = {}", child.id());

    let child_in = child.stdin.as_mut().unwrap();
    let mut child_out = BufReader::new(child.stdout.unwrap()).lines();

    // Receive
    let mut out = child_out.next().unwrap().unwrap();
    out.remove(0);
    out.remove(out.len()-1);
    debug!("Request cleaned {:?}", out);

    let req_serialized:Vec<u8> = out.split(", ").map(|x| x.parse().unwrap()).collect();
    debug!("Serialized request {:?}", req_serialized);

    //  Deserialize Req
    let req: Request = rmp_serde::from_read_ref(&req_serialized).unwrap();
    debug!("Deserialized request {:?}", req);

    //  Document generation
    let document = from_param_to_doc(req.param, false);
    debug!("Document to execute: {}", document);

    //  Query execution
    let mut result_serialized  = Vec::new();
    let answer = match req.op{
        Op::Create => {
            let start_time = SystemTime::now();
            let query_result =
                match conn.collection(req.table.as_str()).insert_one(document,None){
                    Ok(_) => String::from("Success"),
                    Err(_) =>String::from("Error")
                };
            let db_latency = SystemTime::now().duration_since(start_time).unwrap();
            info!("[DB_LATENCY] latency {} μs", db_latency.as_micros());

            query_result
        },
        Op::Read => {
            // Send back a Vec<Row> to keep the invoker independent from the data type
            let start_time = SystemTime::now();
            let query_result = conn.collection::<Document>(req.table.as_str()).find(document,None).unwrap();
            let db_latency = SystemTime::now().duration_since(start_time).unwrap();
            info!("[DB_LATENCY] latency {} μs", db_latency.as_micros());

            let mut query_string :Vec<String>= Vec::new();
            for el in query_result{
                query_string.push(format!("{:?}", el.unwrap()));
            }
            debug!("Res: {:?}", query_string);

            query_string.serialize(&mut Serializer::new(&mut result_serialized)).unwrap();
            debug!("Result serialized: {:?}", result_serialized);

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
            res_string_result_serialized
        },
        Op::Update => {
            let update_doc = from_param_to_doc(req.param_to_up.unwrap(), true);

            let start_time = SystemTime::now();
            let query_result =
                match conn.collection::<HashMap<String, String>>(req.table.as_str()).update_one(document,update_doc, None){
                Ok(_) => String::from("Success"),
                Err(_) =>String::from("Error")
            };
            let db_latency = SystemTime::now().duration_since(start_time).unwrap();
            info!("[DB_LATENCY] latency {} μs", db_latency.as_micros());

            query_result
        },
        Op::Delete => {
            let start_time = SystemTime::now();
            let query_result =
                match conn.collection::<HashMap<String, String>>(req.table.as_str()).delete_one(document,None){
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

    //return the child's output
    let res = child_out.next().unwrap().unwrap();
    debug!("Child output: {}", res);
    return  res;
}

fn server (mut conn: Database, nc: Connection, trigger_command: String, trigger_answer: String, group: String){
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
    let port = env::var("PORT").unwrap_or("27017".to_string());
    let db = env::var("DB_NAME").unwrap_or("testDB".to_string());
    let nats_server = env::var("NATSSERVER").unwrap_or("127.0.0.1".to_string());
    let trigger_command = env::var("TRIGGER").unwrap_or("trigger-command".to_string());
    let trigger_answer = env::var("TRIGGER_ANSWER").unwrap_or("trigger-answer".to_string());
    let group = env::var("GROUP").unwrap_or("default".to_string());
    let url_db = format!("mongodb://{}:{}", address, port);

    debug!("Starts");
    debug!("URL = {}", url_db);

    // Connection to DB
    let client = Client::with_uri_str(url_db).unwrap();
    let conn = client.database(db.as_str());
    debug!("Connected to DB: {:?}", conn);

    // Connection to MOM
    let nc = nats::connect(nats_server.as_str()).unwrap();
    debug!("Connected to NATS {:?} ", nc);
    debug!("Start publishing to topic:{}", trigger_answer);

    server(conn, nc, trigger_command, trigger_answer, group);
}
