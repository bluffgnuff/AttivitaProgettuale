//curl 'http://root:root@127.0.0.1:5984/testdb/_find' -X POST -H 'Content-Type: application/json' -d "{\"selector\": {\"lastname\": {\"\$eq\": \"rossi\"}}}
//CouchDB "http://127.0.0.1:5984/_all_dbs"

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = "{\"selector\": {\"lastname\": {\"$eq\": \"rossi\"}}}";
    let client = reqwest::Client::builder().build()?;
    let resp = client.post("http://127.0.0.1:5984/testdb/_find").basic_auth("root", Some("root")).body(data).header("Content-Type", "application/json")
        .send()
        .await?
        .text()
        .await?;
    println!("{:?}", resp);
    Ok(())
}