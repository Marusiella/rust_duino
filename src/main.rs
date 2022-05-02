use serde::Deserialize;
use std::{sync::mpsc, time::{self, Duration}, vec};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream
};

use clap::Parser;

// Simple multi-threaded duino miner written in Rust.
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // Your miner username
    #[clap(short, long, required = true)]
    username: String,
    // Set number of threads to use
    #[clap(short, long, default_value = "0")]
    threads: u8,
    // Set the difficulty of the mining
    #[clap(short, long, default_value = "EXTRIME", possible_values = &["EXTRIME", "HIGH", "MEDIUM", "LOW"])]
    difficulty: String,
    // type of mining
    #[clap(short, long, default_value = "multithread_one", possible_values = &["multithread_one", "multithread_multi"])]
    mining_type: String,
}
#[derive(Deserialize, Clone)]
struct DuinoPool {
    ip: String,
    name: String,
    port: i64
}

#[derive(Debug)]
struct ToMine {
    from: String,
    to: String,
    difficulty: u128,
}
impl ToMine {
    fn new(msg: String) -> Self {
        let data = msg
            .trim_matches('\u{0}')
            .trim_matches('\n')
            .trim()
            .split(",")
            .collect::<Vec<&str>>();
        println!("{:?} data from msg", data);
        ToMine {
            from: data[0].to_owned(),
            to: data[1].to_owned(),
            difficulty: data[2].parse::<u128>().unwrap(),
        }
    }
}
fn tr(buf: &Vec<u8>) -> String {
    String::from_utf8_lossy(&buf)
        .to_string()
        .trim()
        .trim_matches('\u{0}')
        .trim_matches('\n')
        .trim()
        .to_string()
}
#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("Hello {}!", args.username);
    println!("Getting pool...");
    let resp = reqwest::get("https://server.duinocoin.com/getPool")
        .await
        .expect("Can't get pool")
        .json::<DuinoPool>()
        .await
        .expect("Can't get pool");
    println!("Using pool: {} |  {}:{}", resp.name, resp.ip, resp.port);
    
    if args.mining_type == "multithread_one" {
        let mut tcpstream = match tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(format!("{}:{}", resp.ip, resp.port))
        ).await {
            Ok(socket) => socket.unwrap(),
            Err(_) => {
                println!("Can't connect to the assigned pool");
                TcpStream::connect(format!("server.duinocoin.com:2813")).await.unwrap()
            }
        };
    let mut buf = vec![0; 1024];
    tcpstream.read(&mut buf).await.unwrap();
    println!("Server version is: {}", String::from_utf8_lossy(&buf));
        first_type_mining(&mut tcpstream, &args).await;
    } else {
        let threads: usize = match args.threads {
            0 => num_cpus::get(),
            _ => args.threads as usize,
        };
        let mut threads_vec = vec![];
        for _ in 0..=threads {
            let args = args.clone();
            let resp = resp.clone();
            threads_vec.push(tokio::spawn(async move {
                let mut tcpstream = match tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(format!("{}:{}", resp.ip, resp.port))
        ).await {
            Ok(socket) => socket.unwrap(),
            Err(_) => {
                println!("Can't connect to the assigned pool");
                TcpStream::connect(format!("server.duinocoin.com:2813")).await.unwrap()
            }
        };
            let mut buf = vec![0; 1024];
            tcpstream.read(&mut buf).await.unwrap();
            println!("Server version is: {}", String::from_utf8_lossy(&buf));
            second_type_mining(&mut tcpstream, &args).await ;
            }));
        }
        for thread in threads_vec {
            thread.await.unwrap();
        }
        loop {
            tokio::time::sleep(time::Duration::from_secs(1)).await;
        }
    }
}
async fn first_type_mining(tcpstream: &mut TcpStream, args: &Args) {
    loop {
        let mut buf = vec![0; 1024];
        tcpstream
            .write(format!("JOB,{},{},test", args.username, args.difficulty).as_bytes())
            .await
            .unwrap();
        tcpstream.read(&mut buf).await.unwrap();
        while tr(&buf) == ""
            || tr(&buf) == "GOOD"
            || tr(&buf) == "BLOCK"
            || tr(&buf) == "BAD"
            || tr(&buf) == "INVU"
        {
            if tr(&buf) == "INVU" {
                println!("Invalid user {}", String::from_utf8_lossy(&buf));
                break;
            }
            if tr(&buf) == "GOOD" {
                println!("Good job {}", String::from_utf8_lossy(&buf));
            }
            if tr(&buf) == "BLOCK" {
                println!("Block {}", String::from_utf8_lossy(&buf));
            }
            if tr(&buf) == "BAD" {
                println!("Bad job {}", String::from_utf8_lossy(&buf));
            }
            tcpstream.read(&mut buf).await.unwrap();
        }
        let hash = String::from_utf8_lossy(&buf).to_string().trim().to_string();
        let to_mine = ToMine::new(hash);
        use sha1::{Digest, Sha1};
        let mut hasher = Sha1::new();
        let timel = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        hasher.update(to_mine.from.as_bytes());
        let threads: usize = match args.threads {
            0 => num_cpus::get(),
            _ => args.threads as usize,
        };
        println!("using {} threads", threads);
        let number_for_thread = ((100 * to_mine.difficulty) + 1) / threads as u128;
        let mut found = 0;
        let mut chanel_vec = vec![];
        let mut chanel_vec2 = vec![];
        for x in 1..=threads {
            let (tx, rx) = mpsc::channel::<u128>();
            let (tx2, rx2) = mpsc::channel::<bool>();
            let to_mine = to_mine.to.clone();
            let hasher = hasher.clone();
            println!("spawned thread {}", x);
            std::thread::spawn(move || {
                for result in number_for_thread * (x as u128 - 1)..number_for_thread * x as u128 {
                    if let Ok(resulte) = rx2.try_recv() {
                        if resulte {
                            println!("Stoped at {}", result);
                            break;
                        }
                    }
                    let mut hasher = hasher.clone();
                    hasher.update(result.to_string().as_bytes());
                    if to_mine.clone() == format!("{:x}", hasher.clone().finalize()) {
                        println!("{} thread {} found", result, &x);
                        tx.send(result).unwrap();
                        break;
                    }
                    if result % 100_000_00 == 0 {
                        println!("{} thread id: {}", result, x);
                    }
                }
            });
            chanel_vec.push(rx);
            chanel_vec2.push(tx2);
        }
        loop {
            for x in 0..chanel_vec.len() {
                if let Ok(result) = chanel_vec[x].try_recv() {
                    found = result;
                    break;
                }
            }
            if found != 0 {
                for x in 0..chanel_vec.len() {
                    match chanel_vec2[x].send(true) {
                        Ok(_) => (),
                        Err(_) => (),
                    }
                }
                break;
            }
            std::thread::sleep(time::Duration::from_millis(10));
        }
        let time_of_doing = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            - timel;
        println!("it took {}s", time_of_doing);
        println!(
            "hashrate is {} kH/s",
            (found as f64 / time_of_doing) / 1000.0
        );
        tcpstream
            .write(format!("{},{},RUST", found, found as f64 / time_of_doing).as_bytes())
            .await
            .unwrap();
        buf.clear();
        tcpstream.read(&mut buf).await.unwrap();
    }
}
async fn second_type_mining(tcpstream: &mut TcpStream, args: &Args) {
    loop {
        let mut buf = vec![0; 1024];
        tcpstream
            .write(format!("JOB,{},{},test", args.username, args.difficulty).as_bytes())
            .await
            .unwrap();
        tcpstream.read(&mut buf).await.unwrap();
        while tr(&buf) == ""
            || tr(&buf) == "GOOD"
            || tr(&buf) == "BLOCK"
            || tr(&buf) == "BAD"
            || tr(&buf) == "INVU"
        {
            if tr(&buf) == "INVU" {
                println!("Invalid user {}", String::from_utf8_lossy(&buf));
                break;
            }
            if tr(&buf) == "GOOD" {
                println!("Good job {}", String::from_utf8_lossy(&buf));
            }
            if tr(&buf) == "BLOCK" {
                println!("Block {}", String::from_utf8_lossy(&buf));
            }
            if tr(&buf) == "BAD" {
                println!("Bad job {}", String::from_utf8_lossy(&buf));
            }
            tcpstream.read(&mut buf).await.unwrap();
        }
        let hash = String::from_utf8_lossy(&buf).to_string().trim().to_string();
        let to_mine = ToMine::new(hash);
        use sha1::{Digest, Sha1};
        let mut hasher = Sha1::new();
        let timel = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        hasher.update(to_mine.from.as_bytes());

        let mut this_number = 0;
        for result in 0..((100 * to_mine.difficulty) + 1) {
            let mut hasher = hasher.clone();
            hasher.update(result.to_string().as_bytes());
            if to_mine.to == format!("{:x}", hasher.clone().finalize()) {
                println!("{}", result);
                this_number = result;
                break;
            }
            // println!("{} |  {}", format!("{:x}", hasher.clone().finalize()), to_mine.to);
        }
        let time_of_doing = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            - timel;
        println!("{}", time_of_doing);
        println!(
            "hashrate is {} kH/s",
            (this_number as f64 / time_of_doing) / 1000.0
        );
        // before to unix time
        println!(
            "{}",
            time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
        );
        tcpstream
            .write(format!("{},{},RUST", this_number, this_number as f64 / time_of_doing).as_bytes())
            .await
            .unwrap();
        buf.clear();
        tcpstream.read(&mut buf).await.unwrap();
        println!("z {}", String::from_utf8_lossy(&buf));
    }
}
