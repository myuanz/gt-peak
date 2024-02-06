use socket2::{Domain, SockAddr, Socket, Type};
use std::io::{self, Write};
use std::net::{SocketAddr, TcpStream};
use native_tls::TlsConnector;
use std::io::Read;

fn main() -> io::Result<()> {
    // 创建一个socket
    let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;

    // 连接至 github.com 的 443 端口
    let address: SocketAddr = "140.82.112.4:443".parse().unwrap();
    let address = SockAddr::from(address);
    
    socket.connect(&address)?;

    // 转换socket2的Socket为std::net::TcpStream
    let stream = TcpStream::from(socket);

    // 创建TlsConnector来处理SSL连接
    let connector = TlsConnector::new().unwrap();
    let mut stream = connector.connect("github.com", stream).unwrap();

    // 发送HTTP请求
    let request = "GET / HTTP/1.1\r\nHost: github.com\r\nConnection: close\r\n\r\n";
    let _ = stream.write_all(request.as_bytes())?;

    // 读取响应（这里只是一个简单示例）
    let mut buffer = [0; 64];
    let mut len = 0;
    loop {
        let bytes_read = stream.read(&mut buffer)?;
        len += bytes_read;
        if bytes_read == 0 {
            break;
        }
    }

    println!("Response: {}", len);

    Ok(())
}
