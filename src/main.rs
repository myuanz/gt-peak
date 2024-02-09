use io::Result;
use native_tls::TlsConnector;
use native_tls::TlsStream;
use socket2::{Domain, SockAddr, Socket, Type};
use std::io::Read;
use std::io::{self, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, SystemTime};

#[derive(Debug)]
struct SpeedTestResult {
    start_time: SystemTime,
    connect_time: SystemTime,
    send_time: SystemTime,
    recv_len: u64,
    end_time: SystemTime,
}

// 定义一个新的trait来统一Read和Write操作
pub trait ReadWrite: Read + Write {}
impl ReadWrite for TcpStream {}
impl ReadWrite for TlsStream<TcpStream> {}

impl SpeedTestResult {
    fn latency(&self) -> Duration {
        self.connect_time.duration_since(self.start_time).unwrap()
    }
    /// 计算下载速度
    ///
    /// 单位：字节/秒
    fn download_speed(&self) -> f64 {
        let time = self.end_time.duration_since(self.send_time).unwrap();
        self.recv_len as f64 / time.as_secs_f64()
    }
    fn from_http(ip: &str, port: u16, host: &str, path: &str) -> Result<SpeedTestResult> {
        let start_time = SystemTime::now();

        let socket = Socket::new(Domain::IPV4, Type::STREAM, None).unwrap();
        // let address: SocketAddr = format!("{}:{}", ip, port).parse()?;
        let address = format!("{}:{}", ip, port)
            .parse::<SocketAddr>()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let address = SockAddr::from(address);
        socket.connect(&address)?;
        let connect_time = SystemTime::now();

        let stream = TcpStream::from(socket);
        let mut stream: Box<dyn ReadWrite> = if port == 443 {
            let connector = TlsConnector::new().unwrap();
            let tls_stream = connector
                .connect(host, stream)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            Box::new(tls_stream)
        } else {
            Box::new(stream)
        };

        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, host
        );
        stream.write_all(request.as_bytes())?;
        let send_time = SystemTime::now();

        let mut buffer = [0; 1024];
        let mut recv_len: u64 = 0;
        loop {
            let bytes_read = stream.read(&mut buffer)?;
            recv_len += bytes_read as u64;
            if bytes_read == 0 {
                break;
            }
        }
        let end_time = SystemTime::now();
        Ok(SpeedTestResult {
            start_time,
            connect_time,
            send_time,
            recv_len,
            end_time,
        })
    }
}

fn main() -> io::Result<()> {
    let res = SpeedTestResult::from_http("140.82.112.4", 443, "github.com", "/");
    println!("{:?}", res);

    match res {
        Ok(res) => {
            println!("latency: {:?}", res.latency());
            println!("download speed: {:?} Byte/s", res.download_speed());
        }
        Err(e) => {
            println!("error: {:?}", e);
        }
    }

    Ok(())
}
