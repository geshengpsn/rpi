use std::time::Duration;

use tokio::sync::mpsc::{self, error::SendTimeoutError};

pub fn rx_hub2<D: Clone + Send + 'static>(
    mut rx: mpsc::Receiver<D>,
) -> (mpsc::Receiver<D>, mpsc::Receiver<D>) {
    let (tx1, rx1) = mpsc::channel(1);
    let (tx2, rx2) = mpsc::channel(1);
    tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            tx1.try_send(data.clone()).unwrap();
            tx2.try_send(data).unwrap();
        }
    });
    (rx1, rx2)
}

pub fn rx_hub3<D: Clone + Send + 'static>(
    mut rx: mpsc::Receiver<D>,
) -> (mpsc::Receiver<D>, mpsc::Receiver<D>, mpsc::Receiver<D>) {
    let (tx1, rx1) = mpsc::channel(8);
    let (tx2, rx2) = mpsc::channel(8);
    let (tx3, rx3) = mpsc::channel(8);
    tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            tx1.try_send(data.clone()).unwrap();
            tx2.try_send(data.clone()).unwrap();
            tx3.try_send(data).unwrap();
        }
    });
    (rx1, rx2, rx3)
}
