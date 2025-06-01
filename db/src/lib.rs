use std::env;

use mongodb::{bson::{doc, Document}, Client, Collection};

pub async fn db_test() -> mongodb::error::Result<()> {
    let uri = env::var("MONGODB_URI").expect("Incorrect database_string");
    println!("uri: {}", uri);
    // Create a new client and connect to the server
    let client = Client::with_uri_str(uri).await?;
    
    // Get a handle on the movies collection
    let database = client.database("sample_mflix");
    let my_coll: Collection<Document> = database.collection("movies");
    // Find a movie based on the title value
    let my_movie = my_coll.find_one(doc! { "title": "The Perils of Pauline" }).await?;
    // Print the document
    println!("Found a movie:\n{:#?}", my_movie);
    Ok(())
}

