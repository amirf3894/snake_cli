use clap::{self};
use std::{
    sync::{Arc, RwLock},
    thread, vec,
};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    runtime::Runtime,
    sync::mpsc,
};

pub async fn main_host(size: (u16, u16), addr: &str) -> Result<(), Box<dyn (std::error::Error)>> {
    let (tx, rx) = mpsc::channel::<Vec<(u16, u16)>>(300);
    let mut playground = Arc::new(RwLock::new(
        vec![vec![' '; size.1 as usize].into_boxed_slice(); size.0 as usize].into_boxed_slice(),
    ));
    let listener = TcpListener::bind(addr).await?;
    let async_playground = playground.clone();
    let hanler = tokio::spawn(async move {
        loop {
            //println!("inside loop");
            let thread_playground = async_playground.clone();
            let (socket, _) = listener.accept().await.unwrap();
            println!("socket detedted");
            let thread_playground = async_playground.clone();
            thread::spawn(move || {
                let rt = Runtime::new().unwrap();
                rt.block_on(async { clinet_tasks(socket, thread_playground).await })
                    .unwrap();
            });

            //a.join();
        }
    });
    hanler.await?;
    Ok(())
}
//pub async fn wait_for_connect()
pub async fn clinet_tasks(
    mut socket: TcpStream,
    playground: Arc<RwLock<Box<[Box<[char]>]>>>,
) -> Result<(), Box<dyn (std::error::Error)>> {
    println!("a user entered");
    let mut buf = [0_u8; 500];
    loop {
        let len = socket.read(&mut buf).await?;
        println!("{}", String::from_utf8_lossy(&buf[..len]));
    }

    // let duration = 200;
    //let time_handle = tokio::spawn(sleep(Duration::from_millis(duration)));
    // let mut buf = vec![];
    //let listener = TcpListener::bind(addr)
    // loop {
    //     socket.read_to_end(&mut buf).await?;
    //     //time_handle.await?;
    //     println!("{}", String::from_utf8_lossy(&buf));
    // }
    Ok(())
}
