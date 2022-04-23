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
        let data = msg.trim_matches('\u{0}')
        .trim_matches('\n')
        .trim().split(",").collect::<Vec<&str>>();
        println!("{:?} data from msg", data);
        ToMine {
            from: data[0].to_owned(),
            to: data[1].to_owned(),
            difficulty: data[2]
                .parse::<u128>()
                .unwrap(),
        }
    }
}
#[tokio::main]
async fn main() {
    let mut tcpstream = TcpStream::connect("51.159.175.20:6043").await.unwrap();
    let mut buf = vec![0; 1024];
    tcpstream.read(&mut buf).await.unwrap();
    println!("Server version is: {}", String::from_utf8_lossy(&buf));
   loop {
        let mut buf = vec![0; 1024];
        tcpstream.write(b"JOB,Mareczekk,EXTRIME,test").await.unwrap();
        tcpstream.read(&mut buf).await.unwrap();
        while String::from_utf8_lossy(&buf)
            .to_string()
            .trim()
            .trim_matches('\u{0}')
            .trim_matches('\n')
            .trim()
            == ""
            || String::from_utf8_lossy(&buf)
                .to_string()
                .trim()
                .trim_matches('\u{0}')
                .trim_matches('\n')
                .trim()
                == "GOOD"
        {
            tcpstream.read(&mut buf).await.unwrap();
        }

        let hash = String::from_utf8_lossy(&buf).to_string().trim().to_string();
        let to_mine = ToMine::new(hash);
        // print!("{:?}", to_mine);
        use sha1::{Digest, Sha1};
        let mut hasher = Sha1::new();
        // use std::time::Instant;

        let timel = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        hasher.update(to_mine.from.as_bytes());
        let mut thisNumber = 0;
        for result in 0..((100 * to_mine.difficulty) + 1) {
            let mut hasher = hasher.clone();
            hasher.update(result.to_string().as_bytes());
            if to_mine.to == format!("{:x}", hasher.clone().finalize()) {
                println!("{}", result);
                thisNumber = result;
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
            (thisNumber as f64 / time_of_doing) / 1000.0
        );
        println!(
            "{}",
            time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
        );
        tcpstream
            .write(format!("{},{},RUST", thisNumber, thisNumber as f64 / time_of_doing).as_bytes())
            .await
            .unwrap();
        buf.clear();
        tcpstream.read(&mut buf).await.unwrap();
    }
}