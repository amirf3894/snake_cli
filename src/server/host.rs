use crate::game::model;
use clap::{self, ArgMatches};
use std::{
    net::Ipv4Addr,
    sync::{Arc, RwLock},
    thread, vec,
};
use tokio::{
    net::{self, TcpListener, TcpStream},
    sync::mpsc,
};
pub async fn main_host(size: (u16, u16), addr: &str) -> Result<(), Box<dyn (std::error::Error)>> {
    let (tx, rx) = mpsc::channel::<Vec<(u16, u16)>>(300);
    let mut playground = Arc::new(RwLock::new(
        vec![vec![' '; size.1 as usize].into_boxed_slice(); size.0 as usize].into_boxed_slice(),
    ));
    let listener = TcpListener::bind(addr).await?;
    let async_playground = playground.clone();
    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let thread_playground = async_playground.clone();
            thread::spawn(move || clinet_tasks(socket, thread_playground.clone()));
        }
    });
    Ok(())
}
//pub async fn wait_for_connect()
pub async fn clinet_tasks(socket: TcpStream, playground: Arc<RwLock<Box<[Box<[char]>]>>>) {
    //let listener = TcpListener::bind(addr)
}
