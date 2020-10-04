

use mlws_lib::keybind;
use mlws_lib::rdev::Key;

// #[derive(Debug)]
// pub struct DuplexEnd<S, R = S> {
//     s: Sender<S>,
//     r: Receiver<R>,
// }

// impl<S, R> DuplexEnd<S, R> {
//     pub fn send(&self, m: S) -> Result<(), SendError<S>> {
//         self.s.send(m)
//     }

//     pub fn recv(&self) -> Result<R, RecvError> {
//         self.r.recv()
//     }

//     pub fn try_recv(&self) -> Result<R, TryRecvError> {
//         self.r.try_recv()
//     }
// }

// fn duplex<T, U>() -> (DuplexEnd<T, U>, DuplexEnd<U, T>) {
//     let (s1, r1) = unbounded();
//     let (s2, r2) = unbounded();
//     (DuplexEnd { s: s1, r: r2 }, DuplexEnd { s: s2, r: r1 })
// }

// pub struct KeyBinds<M: Send + 'static, F: Fn((String, String)) -> M + Send> {
//     keybinds: keybind::KeyBindings<M, F, (String, String)>,
//     connections: Vec<DuplexEnd<Reply, Request>>,
// }

// impl<M: Send + Clone + 'static, F: Fn((String, String)) -> M + Send> KeyBinds<M, F> {
//     pub fn new(keybinds: keybind::KeyBindings<M, F, (String, String)>) -> Self {
//         Self {
//             keybinds,
//             connections: Vec::new()
//         }
//     }

//     pub fn connection(&mut self) -> KeyBindClient {
//         let (e1, e2) = duplex();
//         self.connections.push(e1);
//         KeyBindClient::new(e2)
//     }

//     pub async fn tick(&mut self) -> bool {
//         let detected = self.keybinds.has_detected();
//         for i in 0..self.connections.len() {
//             if let Some(keys) = detected.clone() {
//                 self.connections[i].send(Reply::Detected(i, keys)).unwrap();
//             }
//             if self.handle(i).await {
//                 self.connections.remove(i);
//                 tokio::task::yield_now().await;
//                 return true;
//             }
//             tokio::task::yield_now().await;
//         }
//         self.keybinds.tick();
//         false
//     }

//     async fn handle(&mut self, i: usize) -> bool {
//         while let Ok(v) = self.connections[i].try_recv() {
//             match v {
//                 Request::NewConnection => {
//                     println!("NEW CONNECTION {}", self.connections.len());
//                     let (e1, e2) = duplex();
//                     self.connections.push(e1);
//                     self.connections[i].send(Reply::NewConnection(e2)).unwrap();
//                 }
//                 Request::CloseConnection => {
//                     println!("DROP CONNECTION {} ({} left)", i, self.connections.len()-1);
//                     return true
//                 },
//                 Request::Keys => self.connections[i].send(Reply::Keys(self.keybinds.keys())).unwrap(),
//                 Request::Add(repo, keys) => {println!("{:?}", repo); self.keybinds.add(repo, keys)},
//                 Request::StartDetect(id) => {
//                     self.keybinds.start_detecting(id)
//                 }
//                 Request::Detecting => {
//                     self.connections[i].send(Reply::Detecting(self.keybinds.is_detecting())).unwrap()
//                 }
//                 Request::StopDetect => {
//                     self.keybinds.stop_detecting();
                    
//                 }
//             }
//             tokio::task::yield_now().await;
//         }
//         false
//     }
// }

// #[derive(Debug)]
// pub enum Request {
//     NewConnection,
//     CloseConnection,
//     Keys,
//     Add((String, String), Vec<Key>),
//     StartDetect(usize),
//     StopDetect,
//     Detecting
// }

// #[derive(Debug)]
// pub enum Reply {
//     NewConnection(DuplexEnd<Request, Reply>),
//     Keys(mlws_lib::utils::IdMap<((String, String), Vec<Key>)>),
//     Detected(usize, Vec<Key>),
//     Detecting(bool)
// }



// pub struct KeyBindClient {
//     channel: DuplexEnd<Request, Reply>
// }

// impl KeyBindClient {
//     fn new(channel: DuplexEnd<Request, Reply>) -> Self {
//         Self {channel}
//     }

//     pub fn keys(&self) -> mlws_lib::utils::IdMap<((String, String), Vec<Key>)> {
//         self.channel.send(Request::Keys).unwrap();
//         if let Ok(Reply::Keys(k)) = self.channel.recv() {
//             k
//         }else{
//             panic!("Keys expected")
//         }
//     }

//     pub fn add(&self, name: (String, String), keys: Vec<Key>) {
//         self.channel.send(Request::Add(name, keys)).unwrap();
//     }

//     pub fn start_detect(&self, i: usize) {
//         self.channel.send(Request::StartDetect(i)).unwrap();
//     }

//     pub fn stop_detect(&self, i: usize) {
//         self.channel.send(Request::StartDetect(i)).unwrap();
//     }
// }

// impl Clone for KeyBindClient {
//     fn clone(&self) -> Self {
//         self.channel.send(Request::NewConnection).unwrap();
//         match self.channel.recv().unwrap() {
//             Reply::NewConnection(conn) => Self {channel: conn},
//             _ => panic!("Connection expected"),
//         }
//     }
// }

// impl Drop for KeyBindClient {
//     fn drop(&mut self) {
//         self.channel.send(Request::CloseConnection).unwrap()
//     }
    
// }

use server_client::server_client;

server_client!(
    pub Keybinds<M: Send + Clone + 'static, F: Fn((String, String)) -> M + Send> {
        let keybinds: keybind::KeyBindings<M, F, (String, String)>

        fn add(repo: (String, String), keys: Vec<Key>) {
            self.keybinds.add(repo, keys);
        }

        fn keys() -> mlws_lib::utils::IdMap<((String, String), Vec<Key>)> {
            self.keybinds.keys()
        }

        fn ids() -> Vec<usize> {
            self.keybinds.keys().ids().map(|x|*x).collect()
        }
    }
);

