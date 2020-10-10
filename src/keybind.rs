use mlws_lib::keybind;
use mlws_lib::rdev::Key;

use server_client::server_client;

server_client!(
    pub Keybinds<M: Send + Clone + 'static, F: Fn((String, String)) -> M + Send> {
        let keybinds: keybind::KeyBindings<M, F, (String, String)>

        fn add(repo: (String, String)) {
            self.keybinds.add(repo, Vec::new());
        }

        fn remove(i: usize) {
            self.keybinds.remove(i);
        }

        fn set(id: usize, sound: (String, String)) {
            self.keybinds.set_keybind(id, sound);
        }

        fn detect(i: usize) {
            self.keybinds.start_detecting(i);
        }

        fn stop_detect() {
            self.keybinds.stop_detecting();
        }

        fn has_detected() -> Option<Vec<Key>> {
            self.keybinds.has_detected()
        }

        fn keys() -> mlws_lib::utils::IdMap<((String, String), Vec<Key>)> {
            self.keybinds.keys()
        }

        fn ids() -> Vec<usize> {
            self.keybinds.keys().ids().map(|x|*x).collect()
        }
    }
);

