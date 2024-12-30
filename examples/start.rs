use voxidian_editor::*;
use voxidian_database::{ VoxidianDB, DBFilePath };
use std::net::SocketAddr;
use async_std::task::yield_now;
use chrono::TimeDelta;
use uuid::uuid;


#[async_std::main]
async fn main() {

    // Connect to the DB.
    let db = VoxidianDB::connect("sqlite:test.db").await.unwrap();

    // Destroys all subserver and files in the DB. This is just for testing.
    unsafe{ db.prune_subservers(TimeDelta::zero()) }.await.unwrap();

    // Adds a player to the DB.
    let player_uuid = uuid!("bd9e79ad106540458b0887346cff42a7");
    let player_id = if let Some(player_id) = db.get_player_by_uuid(player_uuid).await.unwrap() { player_id.id }
        else { db.create_player(player_uuid, "TotobirdCreation").await.unwrap() };

    // Adds a subserver to the DB.
    let subserver_id = db.create_subserver(player_id, "Totobird's Subserver").await.unwrap();

    // Adds some files to the subserver in the DB.
    db.create_subserver_file(subserver_id, DBFilePath::try_from("src/lib.rs").unwrap(), "fn main() {\n    println!(\"Hello, World!\");\n}\n".as_bytes()).await.unwrap();
    db.create_subserver_file(subserver_id, DBFilePath::try_from("Cargo.toml").unwrap(), "[package]\nname = \"lighthouse-subserver\"\nversion = \"0.1.0\"\nedition = 2024\n".as_bytes()).await.unwrap();
    db.create_subserver_file(subserver_id, DBFilePath::try_from("build.sh").unwrap(), "# Put your build script here!\n".as_bytes()).await.unwrap();


    let mut server = unsafe{ EditorServer::start("127.0.0.1:8080".parse::<SocketAddr>().unwrap()) }.unwrap();
    loop {
        server.update_ws(&db);
        yield_now().await;
    }
}
