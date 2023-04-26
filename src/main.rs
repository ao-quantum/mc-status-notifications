use std::io::{BufRead, stdin};
use std::time::Duration;
use notify_rust::Notification;
use serde::Deserialize;

#[derive(Deserialize)]
struct ServerStatusResponse {
    online: bool,
    host: String,
    port: u16,
    players: Option<ServerStatusPlayers>,
}

#[derive(Deserialize)]
struct ServerStatusPlayers {
    online: u32,
}

async fn check(server_ip: &String, last_status: &bool, force_notif: bool) -> bool {
    let resp = match reqwest::get(format!("https://api.mcstatus.io/v2/status/java/{}", server_ip)).await {
        Ok(resp) => resp,
        Err(_) => {
            send_notification("Failed to get server status", "Please check your internet connection");
            return false;
        },
    };

    let json = resp.json::<ServerStatusResponse>().await.unwrap();

    if &json.online != last_status || force_notif {
        if json.online == true {
            let players = json.players.unwrap();
            Notification::new()
                .appname("Minecraft Server Status")
                .summary("Server is online")
                .body(&*format!("{}:{} is online with {} players", json.host, json.port, players.online))
                .show()
                .expect("Failed to send notification");
        } else {
            Notification::new()
                .appname("Minecraft Server Status")
                .summary("Server is offline")
                .body(&*format!("{}:{} is offline", json.host, json.port))
                .show()
                .expect("Failed to send notification");
        }

        return json.online;
    }

    return last_status.to_owned();
}

fn send_notification(title: &str, body: &str) {
    Notification::new()
        .summary(title)
        .body(body)
        .show()
        .unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Please enter the server you'd like to track");

    let mut server_ip = String::new();
    stdin().lock().read_line(&mut server_ip).unwrap();

    let mut last_status: bool = false;

    let mut force_notif = true;
    let mut timer = tokio::time::interval(Duration::from_secs(60));
    loop {
        timer.tick().await;
        last_status = check(&server_ip, &last_status, force_notif).await;
        force_notif = false;
    }
}
