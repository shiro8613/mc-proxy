use std::{collections::VecDeque, io::{Cursor, Read}, sync::{atomic::{AtomicBool, Ordering}, Arc, Condvar, Mutex}};

use tokio_util::sync::CancellationToken;
use varint_rs::VarintReader;



#[derive(PartialEq, Eq, Clone)]
pub enum CheckerState {
    Ok,
    Fail
}

pub struct Checker {
    token :CancellationToken,
    state :Arc<AtomicBool>,
    clear :Arc<AtomicBool>,
    queue :Arc<(Mutex<VecDeque<Vec<u8>>>, Condvar)>
}

impl Checker {
    pub fn new(token :CancellationToken) -> Self {
        Self { 
            token,
            clear: Arc::new(AtomicBool::new(false)),
            state: Arc::new(AtomicBool::new(false)),
            queue: Arc::new((Mutex::new(VecDeque::new()), Condvar::new()))
        }

    }

    pub fn add_packet(&mut self, packet :Vec<u8>) {
        let (lock, cvar) = &*self.queue;
        let mut q = lock.lock().unwrap();
        q.push_back(packet.clone());
        cvar.notify_one();
    }

    pub fn check(&mut self) -> CheckerState {
		if self.clear.load(Ordering::Acquire) {
			CheckerState::Ok
		} else {
			self.state.store(true, Ordering::Release);
			CheckerState::Fail
		}
	}

    pub fn state_counter(&mut self) {
		let token = self.token.clone();
    	let queue_clone = self.queue.clone();
		let clear = self.clear.clone();
		let state = self.state.clone();
		tokio::spawn(async move {
			let (lock, cvar) = &*queue_clone;
			loop {
				let mut queue = lock.lock().unwrap();
				while queue.is_empty() && !token.is_cancelled() && !state.load(Ordering::Acquire) {
                	queue = cvar.wait(queue).unwrap();
            	}

				if token.is_cancelled() {
					break;
				}

				if state.load(Ordering::Acquire) {
					break;
				}

				if let Some(data) = queue.pop_front() {
					let mut buffer = Cursor::new(data);
					let Ok(_protocol_version) = buffer.read_i32_varint() else {
						continue;
					};
					let Ok(size) = buffer.read_i32_varint() else {
						continue;
					};

					let mut buf = vec![0; size as usize];
					if buffer.read_exact(&mut buf).is_err() {
						continue;
					}
					let Ok(_server_addr) = String::from_utf8(buf) else {
						continue;
					};
					
					let mut buf = [0; 4];
					if buffer.read_exact(&mut buf).is_err() {
						continue;
					}
					let _server_port = u32::from_le_bytes(buf);

					let Ok(_intent) = buffer.read_i32_varint() else {
						continue;
					};

					clear.store(true, Ordering::Release);
					break;
				};
			}
       });
    }
	
	pub fn stop(&mut self) {
		self.state.store(true, Ordering::Release);
	}
}