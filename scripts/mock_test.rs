/// Mock types and constants to make the snippet immediately compilable and testable.
/// rustc ./scripts/mock_test.rs && ./mock_test
pub const TEST_KEYS: &[char] = &['h', 'j', 'k', 'l', 'w', 'b', 'x'];

pub struct MultiPanelApp;
pub struct MultiPanelState;

impl MultiPanelState {
    pub fn closed(_val: f32) -> Self {
        Self
    }
}

impl MultiPanelApp {
    pub fn from_state(_state: MultiPanelState) -> Self {
        Self
    }
    pub fn handle_key(&mut self, _key: char) {
        // Handle key logic...
    }
}

pub fn set_state_bits(_app: &mut MultiPanelApp, _state: u8) {
    // Set bits logic...
}

pub fn state_bits(_app: &MultiPanelApp) -> u8 {
    // Retrieve actual bits...
    00000000
}

pub fn transition_matrix(state: u8, _key: char) -> u8 {
    // Expected transition result...
    state
}

/// Run the full 256×7 transition matrix and return the first mismatch, if any.
pub fn verify_transition_matrix() -> Option<String> {
    for state in 0..=255u8 {
        println!("verify_transition_matrix: state={state}");
        for &key in TEST_KEYS {
            println!("verify_transition_matrix: key={key}");
            let mut app = MultiPanelApp::from_state(MultiPanelState::closed(100.0));
            set_state_bits(&mut app, state);
            app.handle_key(key);
            
            let expected = transition_matrix(state, key);
            let actual = state_bits(&app);
            
            if actual != expected { 
                return Some(format!(
                    "state={:08b} key={:?} expected={:08b} actual={:08b}",
                    state, key, expected, actual
                )); 
            }       
        }           
    }               
    None                
}

fn main() {
    println!("Starting transition matrix verification...");
    match verify_transition_matrix() {
        Some(mismatch) => println!("Mismatch found: {}", mismatch),
        None => println!("Success! All states and keys verified."),
    }
}
