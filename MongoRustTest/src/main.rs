//Mongo Sync
use mongodb::{bson::doc, sync::Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]

struct Customer {
    firstname: String,
    lastname: String,
}

fn main() -> mongodb::error::Result<()> {
    // Get a handle to the cluster
    let client = Client::with_uri_str("mongodb://localhost:27017")?;
    // Ping the server to see if you can connect to the cluster
    let db = client.database("testDB");
    println!("Connected successfully.");

    println!("Databases:");
    for name in db.list_collection_names(None)? {
        println!("- {}", name);
    }

    let customers = db.collection::<Customer>("customers");

    let cursor = customers.find(doc! {"lastname":"Rossi"}, None).unwrap();

    for result in cursor {
        println!("- {}", result?.firstname);
    }
    Ok(())
}