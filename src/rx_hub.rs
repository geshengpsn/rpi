use std::thread::spawn;

use crossbeam::channel::{unbounded, Receiver, TrySendError};

pub fn rx_hub2<D: Clone + Send + 'static>(rx: Receiver<D>) -> (Receiver<D>, Receiver<D>) {
    let (tx1, rx1) = unbounded();
    let (tx2, rx2) = unbounded();
    spawn(move ||{
        while let Ok(data) = rx.recv() {
            if let Err(TrySendError::Disconnected(_)) = tx1.try_send(data.clone()) {
                panic!("rx_hub2 disconnected")
            }
            if let Err(TrySendError::Disconnected(_)) = tx2.try_send(data) {
                panic!("rx_hub2 disconnected")
            }
        }
    });
    (rx1, rx2)
}

pub fn rx_hub3<D: Clone + Send + 'static>(rx: Receiver<D>) -> (Receiver<D>, Receiver<D>, Receiver<D>) {
    let (tx1, rx1) = unbounded();
    let (tx2, rx2) = unbounded();
    let (tx3, rx3) = unbounded();
    spawn(move ||{
        while let Ok(data) = rx.recv() {
            if let Err(TrySendError::Disconnected(_)) = tx1.try_send(data.clone()) {
                panic!("rx_hub2 disconnected")
            }
            if let Err(TrySendError::Disconnected(_)) = tx2.try_send(data.clone()) {
                panic!("rx_hub2 disconnected")
            }
            if let Err(TrySendError::Disconnected(_)) = tx3.try_send(data) {
                panic!("rx_hub2 disconnected")
            }
        }
    });
    (rx1, rx2, rx3)
}
