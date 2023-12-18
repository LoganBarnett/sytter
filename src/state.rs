use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

#[derive(Clone, std::fmt::Debug, serde::Deserialize, serde::Serialize)]
pub struct SytterVariable {
  pub key: String,
  pub value: String,
}

#[derive(Clone, std::fmt::Debug, serde::Deserialize, serde::Serialize)]
pub struct State {
  pub variables: Vec<SytterVariable>,
}

lazy_static! {
  pub static ref STATE: Arc<Mutex<State>> = Arc::new(Mutex::new(State::new()));
}

impl State {

  pub fn new() -> Self {
    State {
      variables: vec!(),
    }
  }

  pub fn get_variables() -> Vec<SytterVariable> {
    let state = STATE
      .lock()
      .unwrap() // If this got poisoned, there's no limping by, just panic.
      ;
    state.variables.clone()
  }

  pub fn get_variable(key: &String) -> Option<String> {
    let state = STATE
      .lock()
      .unwrap() // If this got poisoned, there's no limping by, just panic.
      ;
    state
      .variables
      .iter()
      .find(|v| *key == v.key)
      .map(|v| v.value.clone())
  }

  pub fn set_variable(variable: SytterVariable) {
    let mut state = STATE
      .lock()
      .unwrap() // If this got poisoned, there's no limping by, just panic.
      ;
    match state
      .variables
      .iter_mut()
      .find(|v| variable.key == v.key)
    {
      Some(v) => v.value = variable.value.clone(),
      None => state.variables.push(variable.clone()),
    };
  }

}
