use std::time;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
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
    let mut tcpstream = TcpStream::connect("51.159.175.20:6043").await.unwrap();
    let mut buf = vec![0; 1024];
    tcpstream.read(&mut buf).await.unwrap();
    println!("Server version is: {}", String::from_utf8_lossy(&buf));
    loop {
        let mut buf = vec![0; 1024];
        tcpstream
            .write(b"JOB,Mareczekk,EXTRIME,test")
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
        let mut found = 0;
        for result in 0..((100 * to_mine.difficulty) + 1) {
            let mut hasher = hasher.clone();
            hasher.update(result.to_string().as_bytes());
            if to_mine.to == format!("{:x}", hasher.clone().finalize()) {
                println!("{}", result);
                found = result;
                break;
            }
        }
        let time_of_doing = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            - timel;
        println!("{}", time_of_doing);
        println!(
            "hashrate is {} kH/s",
            (found as f64 / time_of_doing) / 1000.0
        );
        println!(
            "{}",
            time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
        );
        tcpstream
            .write(format!("{},{},RUST", found, found as f64 / time_of_doing).as_bytes())
            .await
            .unwrap();
        buf.clear();
        tcpstream.read(&mut buf).await.unwrap();
    }
}
