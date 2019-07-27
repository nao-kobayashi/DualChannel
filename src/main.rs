use std::thread;
use std::time::Duration;
use std::sync::mpsc::{ channel, Sender, Receiver };
use std::collections::VecDeque;

struct ChannelParameter<T> {
    pub id: usize,
    pub value: T,
}

struct MultiChannel<T> {
    to_child: Vec<Sender<T>>,
    sender_from_child: Sender<ChannelParameter<T>>,
    receiver: Receiver<ChannelParameter<T>>,
    free_list: Vec<usize>,
    ret_values: VecDeque<T>,
    timeout: Duration,
}

impl<T> MultiChannel<T> where T: Clone {
    pub fn new(timeout: Duration) -> Self {
        let (sender, recv) = channel::<ChannelParameter<T>>();

        MultiChannel {
            to_child: Vec::new(),
            sender_from_child: sender,
            receiver: recv,
            free_list: Vec::new(),
            ret_values: VecDeque::new(),
            timeout,
        }
    }

    pub fn get_child_channel(&mut self) -> (usize, Receiver<T>) {
        let (send, recv) = channel::<T>();
        self.to_child.push(send);
        let id = self.free_list.len();
        self.free_list.push(id);
        (id, recv)
    }

    pub fn send_child(&mut self, value: T) {
        loop {
            if self.free_list.len() > 0 {
                let id = self.free_list.pop().unwrap();
                self.to_child[id].send(value).unwrap();
                break;
            } else {
                if let Some(val) = self.recv() {
                    self.ret_values.push_back(val);
                }
            }
        }
    }

    pub fn send_all_child(&self, value: T) {
        for ch in self.to_child.iter() {
            ch.send(value.clone()).unwrap();
        }
    }

    pub fn recv(&mut self) -> Option<T> {
        if let Ok(val) = self.receiver.recv_timeout(self.timeout) {
            self.free_list.push(val.id);
            self.ret_values.push_back(val.value);
        }
        
        self.ret_values.pop_front()
    }
}


fn main() {

    let timeout = Duration::from_millis(10);
    let mut channel_ex = MultiChannel::new(timeout);

    let mut handles = Vec::new();
    for _ in 1..4 {
        let sender1 = channel_ex.sender_from_child.clone(); 
        let (id, recv_ch) = channel_ex.get_child_channel(); 
        handles.push(thread::spawn(move || {
            loop {
                let num = recv_ch.recv().unwrap();
                // println!("id:{} num:{}", id, num);
                if num == -1 { break; }


                std::thread::sleep(std::time::Duration::from_millis(10));

                sender1.send(
                    ChannelParameter{
                        id,
                        value:  num 
                    }).unwrap();
            }
        }));
    }

    for i in 1..100 {
        channel_ex.send_child(i);
        if let Some(ans) = channel_ex.recv() {
            println!("ans:{}", ans);
        }
    }
    channel_ex.send_all_child(-1);

    for h in handles {
        h.join().unwrap();
    }
}
